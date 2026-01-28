use super::*;

cfg_if! {
    if #[cfg(feature="rt-async-std")] {
        mod async_std_providers;
        use async_std_providers::{*, AsyncStdResolver as AsyncResolver};

        use hickory_resolver::{config, system_conf::read_system_conf};

        fn resolver(
            config: config::ResolverConfig,
            options: config::ResolverOpts,
        ) -> AsyncResolver {
            AsyncResolver::builder_with_config(config, AsyncStdConnectionProvider::default()).with_options(options).build()
        }

    } else if #[cfg(feature="rt-tokio")] {
        use hickory_resolver::{config, name_server::TokioConnectionProvider, TokioResolver as AsyncResolver, system_conf::read_system_conf};

        fn resolver(
            config: config::ResolverConfig,
            options: config::ResolverOpts,
        ) -> AsyncResolver {
            AsyncResolver::builder_with_config(config, TokioConnectionProvider::default()).with_options(options).build()
        }
    } else {
        compile_error!("needs executor implementation");
    }
}

struct Resolvers {
    system: Option<Arc<AsyncResolver>>,
    default: Arc<AsyncResolver>,
}

static RESOLVERS: LazyLock<AsyncMutex<Option<Arc<Resolvers>>>> =
    LazyLock::new(|| AsyncMutex::new(None));

async fn with_resolvers<R, F: FnOnce(Arc<Resolvers>) -> PinBoxFutureStatic<R>>(closure: F) -> R {
    let mut resolvers_lock = RESOLVERS.lock().await;
    if let Some(r) = &*resolvers_lock {
        return closure(r.clone()).await;
    }

    let (config, mut options) = (
        config::ResolverConfig::default(),
        config::ResolverOpts::default(),
    );
    options.try_tcp_on_error = true;
    let default = Arc::new(resolver(config, options));

    let system = if let Ok((config, options)) = read_system_conf() {
        Some(Arc::new(resolver(config, options)))
    } else {
        None
    };
    let resolvers = Arc::new(Resolvers { system, default });
    *resolvers_lock = Some(resolvers.clone());
    closure(resolvers).await
}

pub struct Resolver {}

impl Resolver {
    pub async fn txt_lookup<S: AsRef<str>>(host: S) -> Result<Vec<String>, ResolverError> {
        let host = host.as_ref().to_string();
        let txt_result = with_resolvers(|resolvers| {
            Box::pin(async move {
                // Try the system resolver config
                if let Some(system_resolver) = &resolvers.system {
                    match system_resolver.txt_lookup(&host).await {
                        Ok(v) => {
                            return Ok(v);
                        }
                        Err(e) => {
                            debug!("system resolver txt_lookup error: {}", e);
                        }
                    }
                }

                match resolvers.default.txt_lookup(&host).await {
                    Ok(v) => Ok(v),
                    Err(e) => Err(ResolverError::Generic(format!(
                        "default resolver txt_lookup error: {}",
                        e
                    ))),
                }
            })
        })
        .await?;

        let mut out = Vec::new();
        for x in txt_result.iter() {
            let mut record_out = Vec::<u8>::new();
            for txtd in x.txt_data() {
                record_out.extend_from_slice(txtd);
            }
            if let Ok(s) = String::from_utf8(record_out) {
                out.push(s);
            }
        }
        Ok(out)
    }

    pub async fn ptr_lookup(ip_addr: IpAddr) -> Result<String, ResolverError> {
        let ptr_result = with_resolvers(|resolvers| {
            Box::pin(async move {
                // Try the system resolver config
                if let Some(system_resolver) = &resolvers.system {
                    match system_resolver.reverse_lookup(ip_addr).await {
                        Ok(v) => {
                            return Ok(v);
                        }
                        Err(e) => {
                            debug!("system resolver ptr_lookup error: {}", e);
                        }
                    }
                }

                match resolvers.default.reverse_lookup(ip_addr).await {
                    Ok(v) => Ok(v),
                    Err(e) => Err(ResolverError::Generic(format!(
                        "default resolver ptr_lookup error: {}",
                        e
                    ))),
                }
            })
        })
        .await?;

        if let Some(r) = ptr_result.iter().next() {
            Ok(r.to_string().trim_end_matches('.').to_string())
        } else {
            Err(ResolverError::Generic(
                "PTR lookup returned an empty string".to_string(),
            ))
        }
    }
}
