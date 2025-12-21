use crate::dart_isolate_wrapper::*;
use allo_isolate::*;
use cfg_if::*;
use data_encoding::BASE64URL_NOPAD;
use ffi_support::*;
use lazy_static::*;
use opentelemetry::sdk::*;
use opentelemetry::*;
use opentelemetry_otlp::WithExportConfig;
use parking_lot::Mutex;
use serde::*;
use std::io::Write;
use std::os::raw::c_char;
use std::sync::Arc;
use tracing::*;
use tracing_flame::FlameLayer;
use tracing_subscriber::prelude::*;
use veilid_core::tools::*;
use veilid_core::*;

// Detect flutter load/unload
cfg_if! {
    if #[cfg(feature="debug-load")] {
        #[ctor::ctor]
        fn onload() {
            cfg_if! {
                if #[cfg(target_os="android")] {
                    use android_log_sys::*;
                    use std::ffi::{CString, c_int, c_char};
                    unsafe {
                        let tag = CString::new("veilid").unwrap();
                        let text = CString::new(">>> VEILID-FLUTTER LOADED <<<").unwrap();
                        __android_log_write(LogPriority::INFO as c_int, tag.as_ptr() as *const c_char, text.as_ptr() as *const c_char);
                    }
                } else {
                    libc_print::libc_println!(">>> VEILID-FLUTTER LOADED <<<");
                }
            }
        }
        #[ctor::dtor]
        fn onunload() {
            cfg_if! {
                if #[cfg(target_os="android")] {
                    use android_log_sys::*;
                    use std::ffi::{CString, c_int, c_char};
                    unsafe {
                        let tag = CString::new("veilid").unwrap();
                        let text = CString::new(">>> VEILID-FLUTTER UNLOADED <<<").unwrap();
                        __android_log_write(LogPriority::INFO as c_int, tag.as_ptr() as *const c_char, text.as_ptr() as *const c_char);
                    }
                } else {
                    libc_print::libc_println!(">>> VEILID-FLUTTER UNLOADED <<<");
                }
            }
        }
    }
}

// Globals
lazy_static! {
    static ref CORE_INITIALIZED: Mutex<bool> = Mutex::new(false);
    static ref VEILID_API: AsyncMutex<Option<VeilidAPI>> = AsyncMutex::new(None);
    static ref FILTERS: Mutex<BTreeMap<&'static str, VeilidLayerFilter>> =
        Mutex::new(BTreeMap::new());
    static ref ROUTING_CONTEXTS: Mutex<BTreeMap<u32, RoutingContext>> = Mutex::new(BTreeMap::new());
    static ref TABLE_DBS: Mutex<BTreeMap<u32, TableDB>> = Mutex::new(BTreeMap::new());
    static ref TABLE_DB_TRANSACTIONS: Mutex<BTreeMap<u32, TableDBTransaction>> =
        Mutex::new(BTreeMap::new());
    static ref DHT_TRANSACTIONS: Mutex<BTreeMap<u32, DHTTransaction>> = Mutex::new(BTreeMap::new());
    static ref FLAME_GUARD: Mutex<Option<tracing_flame::FlushGuard<std::io::BufWriter<std::fs::File>>>> =
        Mutex::new(None);
}

async fn get_veilid_api() -> VeilidAPIResult<VeilidAPI> {
    let api_lock = VEILID_API.lock().await;
    api_lock
        .as_ref()
        .cloned()
        .ok_or(VeilidAPIError::NotInitialized)
}

/////////////////////////////////////////
// FFI Helpers

// Declare external routine to release ffi strings
define_string_destructor!(free_string);

/////////////////////////////////////////
// FFI-specific

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VeilidFFIConfigLoggingTerminal {
    pub enabled: bool,
    pub level: VeilidConfigLogLevel,
    pub ignore_log_targets: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VeilidFFIConfigLoggingOtlp {
    pub enabled: bool,
    pub level: VeilidConfigLogLevel,
    pub grpc_endpoint: String,
    pub service_name: String,
    pub ignore_log_targets: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VeilidFFIConfigLoggingApi {
    pub enabled: bool,
    pub level: VeilidConfigLogLevel,
    pub ignore_log_targets: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VeilidFFIConfigLoggingFlame {
    pub enabled: bool,
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VeilidFFIConfigLogging {
    pub terminal: VeilidFFIConfigLoggingTerminal,
    pub otlp: VeilidFFIConfigLoggingOtlp,
    pub api: VeilidFFIConfigLoggingApi,
    pub flame: VeilidFFIConfigLoggingFlame,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VeilidFFIConfig {
    pub logging: VeilidFFIConfigLogging,
}

/////////////////////////////////////////
// Initializer
#[no_mangle]
#[instrument]
pub extern "C" fn initialize_veilid_flutter(
    dart_post_c_object_ptr: ffi::DartPostCObjectFnType,
    crash_path: FfiStr,
) {
    unsafe {
        store_dart_post_cobject(dart_post_c_object_ptr);
    }
    let crash_path = crash_path.into_opt_string().unwrap_or_default();

    use std::sync::Once;
    static INIT_BACKTRACE: Once = Once::new();
    INIT_BACKTRACE.call_once(move || {
        // unsafe {
        //     std::env::set_var("RUST_BACKTRACE", "1");
        // }
        std::panic::set_hook(Box::new(move |panic_info| {
            let crash_file = if crash_path.is_empty() {
                None
            } else {
                Some(std::fs::File::create(&crash_path).unwrap())
            };

            let (file, line) = if let Some(loc) = panic_info.location() {
                (loc.file(), loc.line())
            } else {
                ("<unknown>", 0)
            };

            let mut out = String::new();

            out += &format!("### Rust `panic!` hit at file '{}', line {}\n", file, line);
            if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                out += &format!("panic payload: {:?}\n", s);
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                out += &format!("panic payload: {:?}\n", s);
            } else if let Some(a) = panic_info.payload().downcast_ref::<std::fmt::Arguments>() {
                out += &format!("panic payload: {:?}\n", a);
            } else {
                out += "no panic payload\n";
            }
            out += &format!(
                "  Complete stack trace:\n{:?}\n",
                backtrace::Backtrace::new()
            );

            if let Some(mut crash_file) = crash_file {
                write!(crash_file, "{}", out).unwrap();
                crash_file.flush().unwrap();
            } else {
                eprintln!("{}", out);
            }

            // And stop the process, no recovery is going to be possible here
            eprintln!("aborting!");
            std::process::abort();
        }));
    });
}

//////////////////////////////////////////////////////////////////////////////////
// C-compatible FFI Functions

#[no_mangle]
#[instrument]
pub extern "C" fn initialize_veilid_core(platform_config: FfiStr) {
    // Only do this once, ever
    // Until we have Dart native finalizers running on hot-restart, this will cause a crash if run more than once
    {
        let mut core_init = CORE_INITIALIZED.lock();
        if *core_init {
            return;
        }
        *core_init = true;
    }

    let platform_config = platform_config.into_opt_string();
    let platform_config: VeilidFFIConfig =
        deserialize_opt_json(platform_config).expect("failed to deserialize plaform config json");

    // Set up subscriber and layers
    let subscriber = tracing_subscriber::Registry::default();
    let mut layers = Vec::new();
    let mut filters = (*FILTERS).lock();

    let mut fields_to_strip = HashSet::<&'static str>::new();
    fields_to_strip.insert(VEILID_LOG_KEY_FIELD);

    // Terminal logger
    if platform_config.logging.terminal.enabled {
        cfg_if! {
            if #[cfg(target_os = "android")] {
                let filter =
                    VeilidLayerFilter::new(platform_config.logging.terminal.level, &platform_config.logging.terminal.ignore_log_targets, None);
                let layer = paranoid_android::layer("veilid-flutter")
                    .map_fmt_fields(|f| FmtStripFields::new(f, fields_to_strip.clone()))
                    .with_ansi(false)
                    .with_filter(filter.clone());
                filters.insert("terminal", filter);
                layers.push(layer.boxed());
            } else if #[cfg(target_os = "ios")] {
                let filter =
                    VeilidLayerFilter::new(platform_config.logging.terminal.level, &platform_config.logging.terminal.ignore_log_targets, None);
                let layer = tracing_subscriber::fmt::Layer::new()
                    .compact()
                    .map_fmt_fields(|f| FmtStripFields::new(f, fields_to_strip.clone()))
                    .with_ansi(false)
                    .with_writer(std::io::stdout)
                    .with_filter(filter.clone());
                filters.insert("terminal", filter);
                layers.push(layer.boxed());
            } else {
                let filter =
                    VeilidLayerFilter::new(platform_config.logging.terminal.level, &platform_config.logging.terminal.ignore_log_targets, None);
                let layer = tracing_subscriber::fmt::Layer::new()
                    .compact()
                    .map_fmt_fields(|f| FmtStripFields::new(f, fields_to_strip.clone()))
                    .with_writer(std::io::stdout)
                    .with_filter(filter.clone());
                filters.insert("terminal", filter);
                layers.push(layer.boxed());
            }
        }
    };

    // OpenTelemetry logger
    if platform_config.logging.otlp.enabled {
        let grpc_endpoint = platform_config.logging.otlp.grpc_endpoint.clone();

        cfg_if! {
            if #[cfg(feature="rt-async-std")] {
                let exporter = opentelemetry_otlp::new_exporter()
                    .grpcio()
                    .with_endpoint(grpc_endpoint);
                let batch = opentelemetry::runtime::AsyncStd;
            } else if #[cfg(feature="rt-tokio")] {
                let exporter = opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(format!("http://{}", grpc_endpoint));
                let batch = opentelemetry::runtime::Tokio;
            } else {
                compile_error!("needs executor implementation");
            }
        }

        let tracer =
            opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
                    Resource::new(vec![KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                        format!(
                        "{}:{}",
                        platform_config.logging.otlp.service_name,
                        hostname::get()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_else(|_| "unknown".to_owned())),
                    )]),
                ))
                .install_batch(batch)
                .map_err(|e| format!("failed to install OpenTelemetry tracer: {}", e))
                .unwrap();

        let filter = VeilidLayerFilter::new(
            platform_config.logging.otlp.level,
            &platform_config.logging.otlp.ignore_log_targets,
            None,
        );
        let layer = tracing_opentelemetry::layer()
            .with_tracer(tracer)
            .with_filter(filter.clone());
        filters.insert("otlp", filter);
        layers.push(layer.boxed());
    }

    // Flamegraph logger
    if platform_config.logging.flame.enabled {
        let filter = VeilidLayerFilter::new_no_default(
            VeilidConfigLogLevel::Trace,
            &FLAME_LOG_FACILITIES_IGNORE_LIST
                .iter()
                .map(|&x| x.to_string())
                .collect::<Vec<_>>(),
            None,
        );
        let (flame_layer, guard) =
            FlameLayer::with_file(&platform_config.logging.flame.path).unwrap();
        *FLAME_GUARD.lock() = Some(guard);

        // Do not include this in change_log_level changes, so we keep trace level
        // filters.insert("flame", filter.clone());

        layers.push(
            flame_layer
                .with_threads_collapsed(true)
                .with_empty_samples(false)
                .with_filter(filter)
                .boxed(),
        );
    }

    // API logger
    if platform_config.logging.api.enabled {
        let filter = VeilidLayerFilter::new(
            platform_config.logging.api.level,
            &platform_config.logging.api.ignore_log_targets,
            None,
        );
        let layer = ApiTracingLayer::init().with_filter(filter.clone());
        filters.insert("api", filter);
        layers.push(layer.boxed());
    }

    let subscriber = subscriber.with(layers);

    subscriber
        .try_init()
        .map_err(|e| format!("failed to initialize logging: {}", e))
        .expect("failed to initalize ffi platform");
}

#[no_mangle]
pub extern "C" fn change_log_level(layer: FfiStr, log_level: FfiStr) {
    // get layer to change level on
    let layer = layer.into_opt_string().unwrap_or("all".to_owned());
    let layer = if layer == "all" { "".to_owned() } else { layer };

    // get log level to change layer to
    let log_level = log_level.into_opt_string();
    let log_level: VeilidConfigLogLevel = deserialize_opt_json(log_level).unwrap();

    // change log level on appropriate layer
    let filters = (*FILTERS).lock();
    if layer.is_empty() {
        // Change all layers
        for f in filters.values() {
            f.set_max_level(log_level);
        }
    } else {
        // Change a specific layer
        let f = filters.get(layer.as_str()).unwrap();
        f.set_max_level(log_level);
    }
}

#[no_mangle]
pub extern "C" fn change_log_ignore(layer: FfiStr, log_ignore: FfiStr) {
    // get layer to change level on
    let layer = layer.into_opt_string().unwrap_or("all".to_owned());
    let layer = if layer == "all" { "".to_owned() } else { layer };

    // get changes to make
    let log_ignore = log_ignore.into_opt_string().unwrap_or_default();

    // change log level on appropriate layer
    let filters = (*FILTERS).lock();
    if layer.is_empty() {
        // Change all layers
        for f in filters.values() {
            f.set_ignore_list(Some(VeilidLayerFilter::apply_ignore_change(
                &f.ignore_list(),
                log_ignore.clone(),
            )));
        }
    } else {
        // Change a specific layer
        let f = filters.get(layer.as_str()).unwrap();
        f.set_ignore_list(Some(VeilidLayerFilter::apply_ignore_change(
            &f.ignore_list(),
            log_ignore.clone(),
        )));
    }
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn startup_veilid_core(port: i64, stream_port: i64, config: FfiStr) {
    let config = config.into_opt_string();
    let stream = DartIsolateStream::new(stream_port);
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let config_json = match config {
                Some(v) => v,
                None => {
                    let err = VeilidAPIError::MissingArgument {
                        context: "startup_veilid_core".to_owned(),
                        argument: "config".to_owned(),
                    };
                    return VeilidAPIResult::Err(err);
                }
            };

            let mut api_lock = VEILID_API.lock().await;
            if api_lock.is_some() {
                return VeilidAPIResult::Err(VeilidAPIError::AlreadyInitialized);
            }

            let sink = stream.clone();
            let update_callback = Arc::new(move |update: VeilidUpdate| {
                let sink = sink.clone();
                match update {
                    VeilidUpdate::Shutdown => {
                        sink.close();
                    }
                    _ => {
                        sink.item_json(update);
                    }
                }
            });

            let veilid_api = api_startup_json(update_callback, config_json).await?;
            *api_lock = Some(veilid_api);

            Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn get_veilid_state(port: i64) {
    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;
            let core_state = veilid_api.get_state().await?;
            VeilidAPIResult::Ok(core_state)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn is_shutdown(port: i64) {
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await;
            if let Err(VeilidAPIError::NotInitialized) = veilid_api {
                return VeilidAPIResult::Ok(true);
            }
            let veilid_api = veilid_api.unwrap();
            let is_shutdown = veilid_api.is_shutdown();
            VeilidAPIResult::Ok(is_shutdown)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn attach(port: i64) {
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            veilid_api.attach().await?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn detach(port: i64) {
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            veilid_api.detach().await?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn shutdown_veilid_core(port: i64) {
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let mut api_lock = VEILID_API.lock().await;
            let veilid_api = api_lock.take().ok_or(VeilidAPIError::NotInitialized)?;
            veilid_api.shutdown().await;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

fn add_routing_context(
    rc: &mut BTreeMap<u32, RoutingContext>,
    routing_context: RoutingContext,
) -> u32 {
    let mut next_id: u32 = 1;
    while rc.contains_key(&next_id) {
        next_id += 1;
    }
    rc.insert(next_id, routing_context);
    next_id
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context(port: i64) {
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let routing_context = veilid_api.routing_context()?;
            let mut rc = ROUTING_CONTEXTS.lock();
            let new_id = add_routing_context(&mut rc, routing_context);
            VeilidAPIResult::Ok(new_id)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn release_routing_context(id: u32) -> i32 {
    let mut rc = ROUTING_CONTEXTS.lock();
    if rc.remove(&id).is_none() {
        return 0;
    }
    1
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_with_default_safety(id: u32) -> u32 {
    let mut rc = ROUTING_CONTEXTS.lock();
    let Some(routing_context) = rc.get(&id) else {
        return 0;
    };
    let Ok(routing_context) = routing_context.clone().with_default_safety() else {
        return 0;
    };

    add_routing_context(&mut rc, routing_context)
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_with_safety(id: u32, safety_selection: FfiStr) -> u32 {
    let safety_selection: SafetySelection =
        deserialize_opt_json(safety_selection.into_opt_string()).unwrap();

    let mut rc = ROUTING_CONTEXTS.lock();
    let Some(routing_context) = rc.get(&id) else {
        return 0;
    };
    let Ok(routing_context) = routing_context.clone().with_safety(safety_selection) else {
        return 0;
    };

    add_routing_context(&mut rc, routing_context)
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_with_sequencing(id: u32, sequencing: FfiStr) -> u32 {
    let sequencing: Sequencing = deserialize_opt_json(sequencing.into_opt_string()).unwrap();

    let mut rc = ROUTING_CONTEXTS.lock();
    let Some(routing_context) = rc.get(&id) else {
        return 0;
    };
    let routing_context = routing_context.clone().with_sequencing(sequencing);

    add_routing_context(&mut rc, routing_context)
}

fn get_routing_context(id: u32, func_name: &str) -> VeilidAPIResult<RoutingContext> {
    let rc = ROUTING_CONTEXTS.lock();
    let Some(routing_context) = rc.get(&id) else {
        return VeilidAPIResult::Err(VeilidAPIError::invalid_argument(func_name, "id", id));
    };
    Ok(routing_context.clone())
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_safety(port: i64, id: u32) {
    DartIsolateWrapper::new(port).spawn_result_json(async move {
        let routing_context = get_routing_context(id, "routing_context_safety")?;
        VeilidAPIResult::Ok(routing_context.safety())
    });
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_app_call(port: i64, id: u32, target: FfiStr, request: FfiStr) {
    let target: Target = deserialize_opt_json(target.into_opt_string()).unwrap();
    let request: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(request.into_opt_string().unwrap().as_bytes())
        .unwrap();
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let routing_context = get_routing_context(id, "routing_context_app_call")?;

            let answer = routing_context.app_call(target, request).await?;
            let answer = data_encoding::BASE64URL_NOPAD.encode(&answer);
            VeilidAPIResult::Ok(answer)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_app_message(port: i64, id: u32, target: FfiStr, message: FfiStr) {
    let target: Target = deserialize_opt_json(target.into_opt_string()).unwrap();
    let message: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(message.into_opt_string().unwrap().as_bytes())
        .unwrap();
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let routing_context = get_routing_context(id, "routing_context_app_message")?;

            routing_context.app_message(target, message).await?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_create_dht_record(
    port: i64,
    id: u32,
    kind: u32,
    schema: FfiStr,
    owner: FfiStr,
) {
    let crypto_kind = CryptoKind::from(kind);
    let schema: DHTSchema = deserialize_opt_json(schema.into_opt_string()).unwrap();
    let owner: Option<KeyPair> = owner
        .into_opt_string()
        .map(|s| deserialize_json(&s).unwrap());

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let routing_context = get_routing_context(id, "routing_context_create_dht_record")?;

            let dht_record_descriptor = routing_context
                .create_dht_record(crypto_kind, schema, owner)
                .await?;
            VeilidAPIResult::Ok(dht_record_descriptor)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_open_dht_record(port: i64, id: u32, key: FfiStr, writer: FfiStr) {
    let key: RecordKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    let writer: Option<KeyPair> = writer
        .into_opt_string()
        .map(|s| deserialize_json(&s).unwrap());
    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let routing_context = get_routing_context(id, "routing_context_open_dht_record")?;

            let dht_record_descriptor = routing_context.open_dht_record(key, writer).await?;
            VeilidAPIResult::Ok(dht_record_descriptor)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_close_dht_record(port: i64, id: u32, key: FfiStr) {
    let key: RecordKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let routing_context = get_routing_context(id, "routing_context_close_dht_record")?;

            routing_context.close_dht_record(key).await?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_delete_dht_record(port: i64, id: u32, key: FfiStr) {
    let key: RecordKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let routing_context = get_routing_context(id, "routing_context_delete_dht_record")?;

            routing_context.delete_dht_record(key).await?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_get_dht_value(
    port: i64,
    id: u32,
    key: FfiStr,
    subkey: u32,
    force_refresh: bool,
) {
    let key: RecordKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let routing_context = get_routing_context(id, "routing_context_get_dht_value")?;

            let res = routing_context
                .get_dht_value(key, subkey, force_refresh)
                .await?;
            VeilidAPIResult::Ok(res)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_set_dht_value(
    port: i64,
    id: u32,
    key: FfiStr,
    subkey: u32,
    data: FfiStr,
    options: FfiStr,
) {
    let key: RecordKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    let data: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(data.into_opt_string().unwrap().as_bytes())
        .unwrap();
    let options: Option<SetDHTValueOptions> = options
        .into_opt_string()
        .map(|s| deserialize_json(&s).unwrap());

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let routing_context = get_routing_context(id, "routing_context_set_dht_value")?;

            let res = routing_context
                .set_dht_value(key, subkey, data, options)
                .await?;
            VeilidAPIResult::Ok(res)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_watch_dht_values(
    port: i64,
    id: u32,
    key: FfiStr,
    subkeys: FfiStr,
    expiration: u64,
    count: u32,
) {
    let key: RecordKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    let subkeys: Option<ValueSubkeyRangeSet> = subkeys
        .into_opt_string()
        .map(|s| deserialize_json(&s).unwrap());
    let expiration = Timestamp::from(expiration);

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let routing_context = get_routing_context(id, "routing_context_watch_dht_values")?;

            let res = routing_context
                .watch_dht_values(key, subkeys, Some(expiration), Some(count))
                .await?;
            VeilidAPIResult::Ok(res)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_cancel_dht_watch(
    port: i64,
    id: u32,
    key: FfiStr,
    subkeys: FfiStr,
) {
    let key: RecordKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    let subkeys: Option<ValueSubkeyRangeSet> = subkeys
        .into_opt_string()
        .map(|s| deserialize_json(&s).unwrap());

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let routing_context = get_routing_context(id, "routing_context_cancel_dht_watch")?;

            let res = routing_context.cancel_dht_watch(key, subkeys).await?;
            VeilidAPIResult::Ok(res)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn routing_context_inspect_dht_record(
    port: i64,
    id: u32,
    key: FfiStr,
    subkeys: FfiStr,
    scope: FfiStr,
) {
    let key: RecordKey = deserialize_opt_json(key.into_opt_string()).unwrap();

    let subkeys: Option<ValueSubkeyRangeSet> = subkeys
        .into_opt_string()
        .map(|s| deserialize_json(&s).unwrap());

    let scope: DHTReportScope = deserialize_opt_json(scope.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let routing_context = get_routing_context(id, "routing_context_inspect_dht_record")?;

            let res = routing_context
                .inspect_dht_record(key, subkeys, scope)
                .await?;
            VeilidAPIResult::Ok(res)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn generate_member_id(port: i64, writer_key: FfiStr) {
    let writer_key: PublicKey = deserialize_opt_json(writer_key.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;

            let member_id = veilid_api.generate_member_id(&writer_key)?;

            VeilidAPIResult::Ok(member_id)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn get_dht_record_key(
    port: i64,
    schema: FfiStr,
    owner: FfiStr,
    encryption_key: FfiStr,
) {
    let schema: DHTSchema = deserialize_opt_json(schema.into_opt_string()).unwrap();
    let owner: PublicKey = deserialize_opt_json(owner.into_opt_string()).unwrap();
    let encryption_key: Option<SharedSecret> = encryption_key
        .into_opt_string()
        .map(|s| deserialize_json(&s).unwrap());

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;

            let record_key = veilid_api.get_dht_record_key(schema, owner, encryption_key)?;
            VeilidAPIResult::Ok(record_key)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn new_private_route(port: i64) {
    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;

            let route_blob = veilid_api.new_private_route().await?;

            VeilidAPIResult::Ok(route_blob)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn new_custom_private_route(port: i64, stability: FfiStr, sequencing: FfiStr) {
    let stability: Stability = deserialize_opt_json(stability.into_opt_string()).unwrap();
    let sequencing: Sequencing = deserialize_opt_json(sequencing.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;

            let route_blob = veilid_api
                .new_custom_private_route(&VALID_CRYPTO_KINDS, stability, sequencing)
                .await?;

            VeilidAPIResult::Ok(route_blob)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn import_remote_private_route(port: i64, blob: FfiStr) {
    let blob: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(blob.into_opt_string().unwrap().as_bytes())
        .unwrap();
    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;

            let route_id = veilid_api.import_remote_private_route(blob)?;
            VeilidAPIResult::Ok(route_id)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn release_private_route(port: i64, route_id: FfiStr) {
    let route_id: RouteId = deserialize_opt_json(route_id.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            veilid_api.release_private_route(route_id)?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn app_call_reply(port: i64, call_id: FfiStr, message: FfiStr) {
    let call_id = call_id.into_opt_string().unwrap_or_default();
    let message = data_encoding::BASE64URL_NOPAD
        .decode(message.into_opt_string().unwrap().as_bytes())
        .unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let call_id = match call_id.parse() {
                Ok(v) => v,
                Err(e) => {
                    return VeilidAPIResult::Err(VeilidAPIError::invalid_argument(
                        e, "call_id", call_id,
                    ))
                }
            };

            let veilid_api = get_veilid_api().await?;
            veilid_api.app_call_reply(call_id, message).await?;
            Ok(())
        }
        .in_current_span(),
    );
}

fn add_dht_transaction(dht_tx: DHTTransaction) -> u32 {
    let mut next_id: u32 = 1;
    let mut rc = DHT_TRANSACTIONS.lock();
    while rc.contains_key(&next_id) {
        next_id += 1;
    }
    rc.insert(next_id, dht_tx);
    next_id
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn transact_dht_records(port: i64, record_keys: FfiStr, options: FfiStr) {
    let record_keys: Vec<RecordKey> = deserialize_opt_json(record_keys.into_opt_string()).unwrap();
    let options: Option<TransactDHTRecordsOptions> = options
        .into_opt_string()
        .map(|s| deserialize_json(&s).unwrap());

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let dht_tx = veilid_api
                .transact_dht_records(record_keys, options)
                .await?;
            let new_id = add_dht_transaction(dht_tx);
            VeilidAPIResult::Ok(new_id)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn release_dht_transaction(id: u32) -> i32 {
    let mut rc = DHT_TRANSACTIONS.lock();
    if rc.remove(&id).is_none() {
        return 0;
    }
    1
}

fn get_dht_transaction(id: u32, func_name: &str) -> VeilidAPIResult<DHTTransaction> {
    let dht_transactions = DHT_TRANSACTIONS.lock();
    let Some(dht_tx) = dht_transactions.get(&id) else {
        return VeilidAPIResult::Err(VeilidAPIError::invalid_argument(func_name, "id", id));
    };
    Ok(dht_tx.clone())
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn dht_transaction_commit(port: i64, id: u32) {
    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let dht_tx = get_dht_transaction(id, "dht_transaction_commit")?;

            dht_tx.commit().await?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn dht_transaction_rollback(port: i64, id: u32) {
    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let dht_tx = get_dht_transaction(id, "dht_transaction_rollback")?;

            dht_tx.rollback().await?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn dht_transaction_get(port: i64, id: u32, key: FfiStr, subkey: u32) {
    let key: RecordKey = deserialize_opt_json(key.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let dht_tx = get_dht_transaction(id, "dht_transaction_get")?;

            let out = dht_tx.get(key, subkey).await?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn dht_transaction_set(
    port: i64,
    id: u32,
    key: FfiStr,
    subkey: u32,
    data: FfiStr,
    options: FfiStr,
) {
    let key: RecordKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    let data: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(data.into_opt_string().unwrap().as_bytes())
        .unwrap();
    let options: Option<DHTTransactionSetValueOptions> = options
        .into_opt_string()
        .map(|s| deserialize_json(&s).unwrap());

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let dht_tx = get_dht_transaction(id, "dht_transaction_set")?;

            let out = dht_tx.set(key, subkey, data, options).await?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn dht_transaction_inspect(
    port: i64,
    id: u32,
    key: FfiStr,
    subkeys: FfiStr,
    scope: FfiStr,
) {
    let key: RecordKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    let subkeys: Option<ValueSubkeyRangeSet> = subkeys
        .into_opt_string()
        .map(|s| deserialize_json(&s).unwrap());
    let scope: DHTReportScope = deserialize_opt_json(scope.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let dht_tx = get_dht_transaction(id, "dht_transaction_inspect")?;

            let out = dht_tx.inspect(key, subkeys, scope).await?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

fn add_table_db(table_db: TableDB) -> u32 {
    let mut next_id: u32 = 1;
    let mut rc = TABLE_DBS.lock();
    while rc.contains_key(&next_id) {
        next_id += 1;
    }
    rc.insert(next_id, table_db);
    next_id
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn open_table_db(port: i64, name: FfiStr, column_count: u32) {
    let name = name.into_opt_string().unwrap_or_default();
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let tstore = veilid_api.table_store()?;
            let table_db = tstore.open(&name, column_count).await?;
            let new_id = add_table_db(table_db);
            VeilidAPIResult::Ok(new_id)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn release_table_db(id: u32) -> i32 {
    let mut rc = TABLE_DBS.lock();
    if rc.remove(&id).is_none() {
        return 0;
    }
    1
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn delete_table_db(port: i64, name: FfiStr) {
    let name = name.into_opt_string().unwrap_or_default();
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let tstore = veilid_api.table_store()?;
            let deleted = tstore.delete(&name).await?;
            VeilidAPIResult::Ok(deleted)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn table_db_get_column_count(id: u32) -> u32 {
    let table_dbs = TABLE_DBS.lock();
    let Some(table_db) = table_dbs.get(&id) else {
        return 0;
    };
    let Ok(cc) = table_db.clone().get_column_count() else {
        return 0;
    };
    cc
}

fn get_table_db(id: u32, func_name: &str) -> VeilidAPIResult<TableDB> {
    let table_dbs = TABLE_DBS.lock();
    let Some(table_db) = table_dbs.get(&id) else {
        return VeilidAPIResult::Err(VeilidAPIError::invalid_argument(func_name, "id", id));
    };
    Ok(table_db.clone())
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn table_db_get_keys(port: i64, id: u32, col: u32) {
    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let table_db = get_table_db(id, "table_db_get_keys")?;

            let keys = table_db.get_keys(col).await?;
            let out: Vec<String> = keys
                .into_iter()
                .map(|k| BASE64URL_NOPAD.encode(&k))
                .collect();
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

fn add_table_db_transaction(tdbt: TableDBTransaction) -> u32 {
    let mut next_id: u32 = 1;
    let mut tdbts = TABLE_DB_TRANSACTIONS.lock();
    while tdbts.contains_key(&next_id) {
        next_id += 1;
    }
    tdbts.insert(next_id, tdbt);
    next_id
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn table_db_transact(id: u32) -> u32 {
    let table_dbs = TABLE_DBS.lock();
    let Some(table_db) = table_dbs.get(&id) else {
        return 0;
    };
    let tdbt = table_db.clone().transact();

    add_table_db_transaction(tdbt)
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn release_table_db_transaction(id: u32) -> i32 {
    let mut tdbts = TABLE_DB_TRANSACTIONS.lock();
    if tdbts.remove(&id).is_none() {
        return 0;
    }
    1
}

fn get_table_db_transaction(id: u32, func_name: &str) -> VeilidAPIResult<TableDBTransaction> {
    let tdbts = TABLE_DB_TRANSACTIONS.lock();
    let Some(tdbt) = tdbts.get(&id) else {
        return VeilidAPIResult::Err(VeilidAPIError::invalid_argument(func_name, "id", id));
    };
    Ok(tdbt.clone())
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn table_db_transaction_commit(port: i64, id: u32) {
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let tdbt = get_table_db_transaction(id, "table_db_transaction_commit")?;

            tdbt.commit().await?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}
#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn table_db_transaction_rollback(port: i64, id: u32) {
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let tdbt = get_table_db_transaction(id, "table_db_transaction_rollback")?;

            tdbt.rollback();
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn table_db_transaction_store(
    port: i64,
    id: u32,
    col: u32,
    key: FfiStr,
    value: FfiStr,
) {
    let key: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(key.into_opt_string().unwrap().as_bytes())
        .unwrap();
    let value: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(value.into_opt_string().unwrap().as_bytes())
        .unwrap();
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let tdbt = get_table_db_transaction(id, "table_db_transaction_store")?;

            tdbt.store(col, &key, &value).await?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn table_db_transaction_delete(port: i64, id: u32, col: u32, key: FfiStr) {
    let key: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(key.into_opt_string().unwrap().as_bytes())
        .unwrap();
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let tdbt = get_table_db_transaction(id, "table_db_transaction_delete")?;

            tdbt.delete(col, &key).await?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn table_db_store(port: i64, id: u32, col: u32, key: FfiStr, value: FfiStr) {
    let key: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(key.into_opt_string().unwrap().as_bytes())
        .unwrap();
    let value: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(value.into_opt_string().unwrap().as_bytes())
        .unwrap();
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let table_db = get_table_db(id, "table_db_store")?;

            table_db.store(col, &key, &value).await?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn table_db_load(port: i64, id: u32, col: u32, key: FfiStr) {
    let key: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(key.into_opt_string().unwrap().as_bytes())
        .unwrap();
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let table_db = get_table_db(id, "table_db_load")?;

            let out = table_db.load(col, &key).await?;
            let out = out.map(|x| data_encoding::BASE64URL_NOPAD.encode(&x));
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn table_db_delete(port: i64, id: u32, col: u32, key: FfiStr) {
    let key: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(key.into_opt_string().unwrap().as_bytes())
        .unwrap();
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let table_db = get_table_db(id, "table_db_delete")?;

            let out = table_db.delete(col, &key).await?;
            let out = out.map(|x| data_encoding::BASE64URL_NOPAD.encode(&x));
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn valid_crypto_kinds() -> *mut c_char {
    serialize_json(
        VALID_CRYPTO_KINDS
            .iter()
            .map(|k| (*k).into())
            .collect::<Vec<u32>>(),
    )
    .into_ffi_value()
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn verify_signatures(port: i64, node_ids: FfiStr, data: FfiStr, signatures: FfiStr) {
    let node_ids: Vec<PublicKey> = deserialize_opt_json(node_ids.into_opt_string()).unwrap();

    let data: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(data.into_opt_string().unwrap().as_bytes())
        .unwrap();

    let typed_signatures: Vec<Signature> =
        deserialize_opt_json(signatures.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let out = crypto.verify_signatures(&node_ids, &data, &typed_signatures)?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn generate_signatures(port: i64, data: FfiStr, key_pairs: FfiStr) {
    let data: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(data.into_opt_string().unwrap().as_bytes())
        .unwrap();

    let key_pairs: Vec<KeyPair> = deserialize_opt_json(key_pairs.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let out = crypto.generate_signatures(&data, &key_pairs, |_k, s| s)?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn generate_key_pair(port: i64, kind: u32) {
    let kind: CryptoKind = CryptoKind::from(kind);

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let out = Crypto::generate_keypair(kind)?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
pub extern "C" fn crypto_cached_dh(port: i64, kind: u32, key: FfiStr, secret: FfiStr) {
    let kind: CryptoKind = CryptoKind::from(kind);

    let key: PublicKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    let secret: SecretKey = deserialize_opt_json(secret.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_cached_dh", "kind", kind.to_string())
            })?;
            let out = csv.cached_dh(&key, &secret)?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_compute_dh(port: i64, kind: u32, key: FfiStr, secret: FfiStr) {
    let kind: CryptoKind = CryptoKind::from(kind);

    let key: PublicKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    let secret: SecretKey = deserialize_opt_json(secret.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_compute_dh", "kind", kind.to_string())
            })?;
            let out = csv.compute_dh(&key, &secret)?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_generate_shared_secret(
    port: i64,
    kind: u32,
    key: FfiStr,
    secret: FfiStr,
    domain: FfiStr,
) {
    let kind: CryptoKind = CryptoKind::from(kind);

    let key: PublicKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    let secret: SecretKey = deserialize_opt_json(secret.into_opt_string()).unwrap();
    let domain: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(domain.into_opt_string().unwrap().as_bytes())
        .unwrap();

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_generate_shared_secret",
                    "kind",
                    kind.to_string(),
                )
            })?;
            let out = csv.generate_shared_secret(&key, &secret, &domain)?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_random_bytes(port: i64, kind: u32, len: u32) {
    let kind: CryptoKind = CryptoKind::from(kind);

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_random_bytes", "kind", kind.to_string())
            })?;
            let out = csv.random_bytes(len);
            let out = data_encoding::BASE64URL_NOPAD.encode(&out);
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_shared_secret_length(port: i64, kind: u32) {
    let kind: CryptoKind = CryptoKind::from(kind);

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_shared_secret_length",
                    "kind",
                    kind.to_string(),
                )
            })?;
            let out = csv.shared_secret_length();
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_nonce_length(port: i64, kind: u32) {
    let kind: CryptoKind = CryptoKind::from(kind);

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_nonce_length", "kind", kind.to_string())
            })?;
            let out = csv.nonce_length();
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_hash_digest_length(port: i64, kind: u32) {
    let kind: CryptoKind = CryptoKind::from(kind);

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_hash_digest_length",
                    "kind",
                    kind.to_string(),
                )
            })?;
            let out = csv.hash_digest_length();
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_public_key_length(port: i64, kind: u32) {
    let kind: CryptoKind = CryptoKind::from(kind);

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_public_key_length",
                    "kind",
                    kind.to_string(),
                )
            })?;
            let out = csv.public_key_length();
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_secret_key_length(port: i64, kind: u32) {
    let kind: CryptoKind = CryptoKind::from(kind);

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_secret_key_length",
                    "kind",
                    kind.to_string(),
                )
            })?;
            let out = csv.secret_key_length();
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_signature_length(port: i64, kind: u32) {
    let kind: CryptoKind = CryptoKind::from(kind);

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_signature_length",
                    "kind",
                    kind.to_string(),
                )
            })?;
            let out = csv.signature_length();
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_default_salt_length(port: i64, kind: u32) {
    let kind: CryptoKind = CryptoKind::from(kind);

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_default_salt_length",
                    "kind",
                    kind.to_string(),
                )
            })?;
            let out = csv.default_salt_length();
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_aead_overhead(port: i64, kind: u32) {
    let kind: CryptoKind = CryptoKind::from(kind);

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_aead_overhead", "kind", kind.to_string())
            })?;
            let out = csv.aead_overhead();
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_check_shared_secret(port: i64, kind: u32, secret: FfiStr) {
    let kind: CryptoKind = CryptoKind::from(kind);
    let secret: SharedSecret = deserialize_opt_json(secret.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_check_shared_secret",
                    "kind",
                    kind.to_string(),
                )
            })?;
            csv.check_shared_secret(&secret)?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_check_nonce(port: i64, kind: u32, nonce: FfiStr) {
    let kind: CryptoKind = CryptoKind::from(kind);
    let nonce: Nonce = deserialize_opt_json(nonce.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_check_nonce", "kind", kind.to_string())
            })?;
            csv.check_nonce(&nonce)?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_check_hash_digest(port: i64, kind: u32, digest: FfiStr) {
    let kind: CryptoKind = CryptoKind::from(kind);
    let digest: HashDigest = deserialize_opt_json(digest.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_check_hash_digest",
                    "kind",
                    kind.to_string(),
                )
            })?;
            csv.check_hash_digest(&digest)?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_check_public_key(port: i64, kind: u32, key: FfiStr) {
    let kind: CryptoKind = CryptoKind::from(kind);
    let key: PublicKey = deserialize_opt_json(key.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_check_public_key",
                    "kind",
                    kind.to_string(),
                )
            })?;
            csv.check_public_key(&key)?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_check_secret_key(port: i64, kind: u32, key: FfiStr) {
    let kind: CryptoKind = CryptoKind::from(kind);
    let key: SecretKey = deserialize_opt_json(key.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_check_secret_key",
                    "kind",
                    kind.to_string(),
                )
            })?;
            csv.check_secret_key(&key)?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_check_signature(port: i64, kind: u32, signature: FfiStr) {
    let kind: CryptoKind = CryptoKind::from(kind);
    let signature: Signature = deserialize_opt_json(signature.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_check_signature", "kind", kind.to_string())
            })?;
            csv.check_signature(&signature)?;
            VeilidAPIResult::Ok(())
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_hash_password(port: i64, kind: u32, password: FfiStr, salt: FfiStr) {
    let kind: CryptoKind = CryptoKind::from(kind);
    let password: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(password.into_opt_string().unwrap().as_bytes())
        .unwrap();
    let salt: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(salt.into_opt_string().unwrap().as_bytes())
        .unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_hash_password", "kind", kind.to_string())
            })?;
            let out = csv.hash_password(&password, &salt)?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_verify_password(
    port: i64,
    kind: u32,
    password: FfiStr,
    password_hash: FfiStr,
) {
    let kind: CryptoKind = CryptoKind::from(kind);
    let password: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(password.into_opt_string().unwrap().as_bytes())
        .unwrap();
    let password_hash = password_hash.into_opt_string().unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_verify_password", "kind", kind.to_string())
            })?;
            let out = csv.verify_password(&password, &password_hash)?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_derive_shared_secret(
    port: i64,
    kind: u32,
    password: FfiStr,
    salt: FfiStr,
) {
    let kind: CryptoKind = CryptoKind::from(kind);
    let password: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(password.into_opt_string().unwrap().as_bytes())
        .unwrap();
    let salt: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(salt.into_opt_string().unwrap().as_bytes())
        .unwrap();

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_derive_shared_secret",
                    "kind",
                    kind.to_string(),
                )
            })?;
            let out = csv.derive_shared_secret(&password, &salt)?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_random_nonce(port: i64, kind: u32) {
    let kind: CryptoKind = CryptoKind::from(kind);

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_random_nonce", "kind", kind.to_string())
            })?;
            let out = csv.random_nonce();
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_random_shared_secret(port: i64, kind: u32) {
    let kind: CryptoKind = CryptoKind::from(kind);

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_random_shared_secret",
                    "kind",
                    kind.to_string(),
                )
            })?;
            let out = csv.random_shared_secret();
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_generate_key_pair(port: i64, kind: u32) {
    let kind: CryptoKind = CryptoKind::from(kind);

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_generate_key_pair",
                    "kind",
                    kind.to_string(),
                )
            })?;
            let out = csv.generate_keypair();
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_generate_hash(port: i64, kind: u32, data: FfiStr) {
    let kind: CryptoKind = CryptoKind::from(kind);

    let data: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(data.into_opt_string().unwrap().as_bytes())
        .unwrap();

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_generate_hash", "kind", kind.to_string())
            })?;
            let out = csv.generate_hash(&data);
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_validate_key_pair(port: i64, kind: u32, key: FfiStr, secret: FfiStr) {
    let kind: CryptoKind = CryptoKind::from(kind);

    let key: PublicKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    let secret: SecretKey = deserialize_opt_json(secret.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument(
                    "crypto_validate_key_pair",
                    "kind",
                    kind.to_string(),
                )
            })?;
            let out = csv.validate_keypair(&key, &secret);
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_validate_hash(port: i64, kind: u32, data: FfiStr, hash: FfiStr) {
    let kind: CryptoKind = CryptoKind::from(kind);

    let data: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(data.into_opt_string().unwrap().as_bytes())
        .unwrap();

    let hash: HashDigest = deserialize_opt_json(hash.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_validate_hash", "kind", kind.to_string())
            })?;
            let out = csv.validate_hash(&data, &hash);
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_sign(port: i64, kind: u32, key: FfiStr, secret: FfiStr, data: FfiStr) {
    let kind: CryptoKind = CryptoKind::from(kind);

    let key: PublicKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    let secret: SecretKey = deserialize_opt_json(secret.into_opt_string()).unwrap();
    let data: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(data.into_opt_string().unwrap().as_bytes())
        .unwrap();

    DartIsolateWrapper::new(port).spawn_result_json(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_sign", "kind", kind.to_string())
            })?;
            let out = csv.sign(&key, &secret, &data)?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_verify(
    port: i64,
    kind: u32,
    key: FfiStr,
    data: FfiStr,
    signature: FfiStr,
) {
    let kind: CryptoKind = CryptoKind::from(kind);

    let key: PublicKey = deserialize_opt_json(key.into_opt_string()).unwrap();
    let data: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(data.into_opt_string().unwrap().as_bytes())
        .unwrap();
    let signature: Signature = deserialize_opt_json(signature.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_verify", "kind", kind.to_string())
            })?;
            let out = csv.verify(&key, &data, &signature)?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_decrypt_aead(
    port: i64,
    kind: u32,
    body: FfiStr,
    nonce: FfiStr,
    shared_secret: FfiStr,
    associated_data: FfiStr,
) {
    let kind: CryptoKind = CryptoKind::from(kind);

    let body: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(body.into_opt_string().unwrap().as_bytes())
        .unwrap();

    let nonce: Nonce = deserialize_opt_json(nonce.into_opt_string()).unwrap();

    let shared_secret: SharedSecret =
        deserialize_opt_json(shared_secret.into_opt_string()).unwrap();

    let associated_data: Option<Vec<u8>> = associated_data
        .into_opt_string()
        .map(|s| data_encoding::BASE64URL_NOPAD.decode(s.as_bytes()).unwrap());

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_decrypt_aead", "kind", kind.to_string())
            })?;
            let out = csv.decrypt_aead(
                &body,
                &nonce,
                &shared_secret,
                match &associated_data {
                    Some(ad) => Some(ad.as_slice()),
                    None => None,
                },
            )?;
            let out = data_encoding::BASE64URL_NOPAD.encode(&out);
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_encrypt_aead(
    port: i64,
    kind: u32,
    body: FfiStr,
    nonce: FfiStr,
    shared_secret: FfiStr,
    associated_data: FfiStr,
) {
    let kind: CryptoKind = CryptoKind::from(kind);

    let body: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(body.into_opt_string().unwrap().as_bytes())
        .unwrap();

    let nonce: Nonce = deserialize_opt_json(nonce.into_opt_string()).unwrap();

    let shared_secret: SharedSecret =
        deserialize_opt_json(shared_secret.into_opt_string()).unwrap();

    let associated_data: Option<Vec<u8>> = associated_data
        .into_opt_string()
        .map(|s| data_encoding::BASE64URL_NOPAD.decode(s.as_bytes()).unwrap());

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_encrypt_aead", "kind", kind.to_string())
            })?;
            let out = csv.encrypt_aead(
                &body,
                &nonce,
                &shared_secret,
                match &associated_data {
                    Some(ad) => Some(ad.as_slice()),
                    None => None,
                },
            )?;
            let out = data_encoding::BASE64URL_NOPAD.encode(&out);
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn crypto_crypt_no_auth(
    port: i64,
    kind: u32,
    body: FfiStr,
    nonce: FfiStr,
    shared_secret: FfiStr,
) {
    let kind: CryptoKind = CryptoKind::from(kind);

    let mut body: Vec<u8> = data_encoding::BASE64URL_NOPAD
        .decode(body.into_opt_string().unwrap().as_bytes())
        .unwrap();

    let nonce: Nonce = deserialize_opt_json(nonce.into_opt_string()).unwrap();

    let shared_secret: SharedSecret =
        deserialize_opt_json(shared_secret.into_opt_string()).unwrap();

    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let crypto = veilid_api.crypto()?;
            let csv = crypto.get(kind).ok_or_else(|| {
                VeilidAPIError::invalid_argument("crypto_crypt_no_auth", "kind", kind.to_string())
            })?;
            csv.crypt_in_place_no_auth(&mut body, &nonce, &shared_secret)?;
            let body = data_encoding::BASE64URL_NOPAD.encode(&body);
            VeilidAPIResult::Ok(body)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn now() -> u64 {
    Timestamp::now().as_u64()
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn debug(port: i64, command: FfiStr) {
    let command = command.into_opt_string().unwrap_or_default();
    DartIsolateWrapper::new(port).spawn_result(
        async move {
            let veilid_api = get_veilid_api().await?;
            let out = veilid_api.debug(command).await?;
            VeilidAPIResult::Ok(out)
        }
        .in_current_span(),
    );
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn veilid_version_string() -> *mut c_char {
    veilid_core::veilid_version_string().into_ffi_value()
}

#[repr(C)]
pub struct VeilidVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn veilid_version() -> VeilidVersion {
    let (major, minor, patch) = veilid_core::veilid_version();
    VeilidVersion {
        major,
        minor,
        patch,
    }
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn default_veilid_config() -> *mut c_char {
    veilid_core::default_veilid_config().into_ffi_value()
}

#[no_mangle]
#[instrument(level = "trace", target = "ffi", skip_all)]
pub extern "C" fn veilid_features() -> *mut c_char {
    serde_json::to_string(&veilid_core::veilid_features())
        .expect("Failed to serialize features")
        .into_ffi_value()
}
