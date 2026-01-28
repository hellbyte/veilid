use std::sync::Arc;
use veilid_core::VeilidUpdate::{AppMessage, Network};
use veilid_core::{
    VeilidConfig, VeilidConfigProtectedStore, VeilidConfigTableStore, VeilidTracing, VeilidUpdate,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a basic update callback to display Veilid's update events
    let update_callback = Arc::new(move |update: VeilidUpdate| {
        match update {
            AppMessage(msg) => {
                println!("Message: {}", String::from_utf8_lossy(msg.message()));
            }
            Network(msg) => {
                println!(
                    "Network: Node Ids: {}, Peers {}, bytes/sec [{} up] [{} down]",
                    if msg.node_ids.is_empty() {
                        "(none assigned)".to_string()
                    } else {
                        msg.node_ids
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    },
                    msg.peers.len(),
                    msg.bps_up,
                    msg.bps_down
                )
            }
            _ => {
                println!("{:#?}", update)
            }
        };
    });

    // Set up a config for this application
    let exe_dir = std::env::current_exe()
        .map(|x| x.parent().map(|p| p.to_owned()))
        .ok()
        .flatten()
        .unwrap_or(".".into());
    let config = VeilidConfig {
        program_name: "Example Veilid".into(),
        namespace: "veilid-example".into(),

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

    // Create simple veilid logger using the default RUST_LOG environment variable
    VeilidTracing::stderr().try_apply_default_env()?;

    // Startup Veilid node
    let veilid = veilid_core::api_startup(update_callback, config).await?;

    // Attach to the network
    veilid.attach().await?;

    // Until CTRL+C is pressed, keep running
    tokio::signal::ctrl_c().await?;

    veilid.shutdown().await;

    Ok(())
}
