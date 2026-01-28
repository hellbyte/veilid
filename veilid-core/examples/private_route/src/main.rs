use clap::Parser;
use std::{future::Future, io::Write as _, sync::Arc};
use veilid_core::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Route blob to connect to
    #[arg(long)]
    connect: Option<String>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Veilid Private Routing Example");

    // Parse the command line
    let cli = Cli::parse();

    // Set up exit handler
    let (done_send, done_recv) = tokio::sync::mpsc::channel(1);
    ctrlc::set_handler(move || {
        let _ = done_send.try_send(());
    })
    .expect("Error setting Ctrl-C handler");

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Set up some basic Veilid terminal logging
    let logs = VeilidTracing::stderr();

    // Use RUST_LOG environment variable to set up logging
    logs.try_apply_default_env()?;

    // If the '-d' flag is specified, determine which log level to override
    let debug_level = match cli.debug {
        1 => VeilidConfigLogLevel::Info,
        2 => VeilidConfigLogLevel::Debug,
        3.. => VeilidConfigLogLevel::Trace,
        _ => VeilidConfigLogLevel::Warn,
    };

    // Override the veilid 'common' facility tag from the command line
    logs.try_apply_facility_level("#common", debug_level)?;

    // Set up a config for this application
    let exe_dir = std::env::current_exe()
        .map(|x| x.parent().map(|p| p.to_owned()))
        .ok()
        .flatten()
        .unwrap_or(".".into());

    let config = VeilidConfig {
        program_name: "Veilid Private Routing Example".into(),
        protected_store: VeilidConfigProtectedStore {
            // IMPORTANT: don't do this in production
            // This avoids prompting for a password and is insecure
            always_use_insecure_storage: true,
            directory: exe_dir
                .join(".veilid/protected_store")
                .to_string_lossy()
                .to_string(),
            ..Default::default()
        },
        table_store: VeilidConfigTableStore {
            directory: exe_dir
                .join(".veilid/table_store")
                .to_string_lossy()
                .to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    // Handle 'connect' option
    if let Some(connect) = cli.connect {
        // Parse the blob from base64
        let blob: Vec<u8> = data_encoding::BASE64.decode(connect.as_bytes())?;

        // Open the route
        return open_route(blob, done_recv, config).await;
    }

    // Create a new route and listen on it
    create_route(done_recv, config).await
}

async fn try_again_loop<R, F: Future<Output = VeilidAPIResult<R>>>(
    f: impl Fn() -> F,
) -> VeilidAPIResult<R> {
    let mut waiting = false;
    loop {
        let res = f().await;
        match res {
            Ok(v) => {
                if waiting {
                    println!("ready.");
                }
                return Ok(v);
            }
            Err(VeilidAPIError::TryAgain { message: _ }) => {
                if !waiting {
                    print!("Waiting for network...");
                    waiting = true;
                } else {
                    print!(".");
                }
                let _ = std::io::stdout().flush();
                tools::sleep(1000).await;
            }
            Err(e) => {
                if waiting {
                    println!();
                }
                return Err(e);
            }
        }
    }
}

async fn veilid_api_scope<'a, F: Future<Output = Result<T, Box<dyn std::error::Error>>>, T>(
    update_callback: impl Fn(VeilidUpdate) + Send + Sync + 'static,
    veilid_config: VeilidConfig,
    scope: impl FnOnce(VeilidAPI) -> F + Send + Sync + 'a,
) -> Result<T, Box<dyn std::error::Error>> {
    // Startup Veilid node
    // Note: future is boxed due to its size and our aggressive clippy lints
    let veilid_api = Box::pin(api_startup(Arc::new(update_callback), veilid_config)).await?;

    // Attach to the network
    veilid_api.attach().await?;

    // Operate the Veilid node inside a scope
    let res = scope(veilid_api.clone()).await;

    // Clean shutdown
    veilid_api.shutdown().await;

    // Return result
    res
}

async fn create_route(
    mut done_recv: tokio::sync::mpsc::Receiver<()>,
    mut config: VeilidConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use a namespace for the receiving side of the private route
    config.namespace = "recv".to_owned();

    // Run veilid node
    veilid_api_scope(update_callback, config, |veilid_api| async move {

        // Create a new private route endpoint
        let RouteBlob { route_id, blob }  =
            try_again_loop(|| async { veilid_api.new_private_route().await }).await?;

        // Print the blob
        println!(
            "Route id created: {route_id}\nConnect with this private route blob:\ncargo run --example private-route-example -- --connect {}",
            data_encoding::BASE64.encode(&blob)
        );

        // Wait for enter key to exit the application
        // The VeilidUpdate for AppMessages will print received messages in the background
        println!("Press ctrl-c when you are finished.");
        let _ = done_recv.recv().await;

        Ok(())

    }).await
}

async fn open_route(
    route_blob: Vec<u8>,
    mut done_recv: tokio::sync::mpsc::Receiver<()>,
    mut config: VeilidConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use a namespace for the sending side of the private route
    config.namespace = "send".to_owned();

    // Run veilid node
    veilid_api_scope(update_callback, config, |veilid_api| async move {
        // Import the private route blob
        let route_id =
            try_again_loop(|| async { veilid_api.import_remote_private_route(route_blob.clone()) })
                .await?;

        // Create a routing context to send with
        let rc = veilid_api.routing_context()?;

        // Get some strings from stdin to send
        println!("Enter some lines to send. Send an empty line when you're finished.");

        let mut rx = async_stdin::recv_from_stdin(1);

        loop {
            tokio::select! {
                val = rx.recv() => {
                    if let Some(val) = val {
                        if val.is_empty() {
                            break;
                        }
                        try_again_loop(|| async { rc.app_message(Target::RouteId(route_id.clone()), val.as_bytes().to_vec()).await })
                            .await?;
                    } else {
                        break;
                    }
                }
                _ = done_recv.recv() => {
                    break;
                }
            }
        }

        Ok(())
    }).await
}

fn update_callback(update: VeilidUpdate) {
    match update {
        VeilidUpdate::Log(_veilid_log) => {}
        VeilidUpdate::AppMessage(veilid_app_message) => {
            let msg = String::from_utf8_lossy(veilid_app_message.message());
            println!("AppMessage received: {msg}");
        }
        VeilidUpdate::AppCall(_veilid_app_call) => {}
        VeilidUpdate::Attachment(_veilid_state_attachment) => {}
        VeilidUpdate::Network(_veilid_state_network) => {}
        VeilidUpdate::Config(_veilid_state_config) => {}
        VeilidUpdate::RouteChange(veilid_route_change) => {
            // XXX: If this happens, the route is dead, and a new one should be generated and
            // exchanged. This will no longer be necessary after DHT Route Autopublish is implemented in veilid-core v0.6.0
            println!("{veilid_route_change:?}");
        }
        VeilidUpdate::ValueChange(_veilid_value_change) => {}
        VeilidUpdate::Shutdown => {}
    }
}
