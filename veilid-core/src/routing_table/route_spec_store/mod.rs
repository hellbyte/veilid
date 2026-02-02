use super::*;
use crate::veilid_api::*;

mod remote_private_route_info;
mod route_allocate;
mod route_assemble;
mod route_compile;
mod route_remote;
mod route_select;
mod route_set_spec_detail;
mod route_spec_store_cache;
mod route_spec_store_content;
mod route_stats;
mod route_test;
mod route_validate;

use remote_private_route_info::*;
use route_set_spec_detail::*;
use route_spec_store_cache::*;
use route_spec_store_content::*;

pub(crate) use route_allocate::AllocateRouteParams;
pub(crate) use route_select::{RouteIdAndPublicKeys, RouteSelectParams};
pub(crate) use route_spec_store_cache::CompiledRoute;
pub use route_stats::*;

impl_veilid_log_facility!("rtab");

/// The size of the remote private route cache
const REMOTE_PRIVATE_ROUTE_CACHE_SIZE: usize = 1024;
/// Remote private route cache entries expire in 5 minutes if they haven't been used
const REMOTE_PRIVATE_ROUTE_CACHE_EXPIRY: TimestampDuration = TimestampDuration::new(300_000_000u64);
/// Amount of time a route can remain idle before it gets tested
const ROUTE_MIN_IDLE_TIME_MS: u32 = 30_000;
/// The size of the compiled route cache
const COMPILED_ROUTE_CACHE_SIZE: usize = 256;

#[derive(Debug)]
struct RouteSpecStoreInner {
    /// Serialize RouteSpecStore content
    content: RouteSpecStoreContent,
    /// RouteSpecStore cache
    cache: RouteSpecStoreCache,
}

/// Key for the compile lock table
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct CompileLockKey {
    pr_pubkey: PublicKey,
}

/// The routing table's storage for private/safety routes
#[derive(Debug)]
#[must_use]
pub(crate) struct RouteSpecStore {
    registry: VeilidComponentRegistry,
    inner: RwLock<RouteSpecStoreInner>,
    /// Ensure we don't compile the same route more than once at a time
    compile_lock_table: AsyncTagLockTable<CompileLockKey>,
    /// Ensure we don't try to select the first available route for more the same parameters than once at a time
    first_available_route_lock: AsyncMutex<()>,
    /// Maximum number of hops in a route
    max_route_hop_count: usize,
    /// Default number of hops in a safe route
    default_route_hop_count_safe: usize,
    /// Default number of hops in an unsafe route
    default_route_hop_count_unsafe: usize,
}

impl_veilid_component_accessors!(RouteSpecStore);

impl RouteSpecStore {
    pub fn new(registry: VeilidComponentRegistry) -> Self {
        let config = registry.config();

        let max_route_hop_count = config.network.rpc.max_route_hop_count as usize;
        let default_route_hop_count_safe = config.network.rpc.default_route_hop_count as usize;
        let default_route_hop_count_unsafe =
            max_route_hop_count.min(default_route_hop_count_safe * 2);

        Self {
            registry: registry.clone(),
            inner: RwLock::new(RouteSpecStoreInner {
                content: RouteSpecStoreContent::default(),
                cache: RouteSpecStoreCache::new(registry.clone()),
            }),
            compile_lock_table: AsyncTagLockTable::new(),
            first_available_route_lock: AsyncMutex::new(()),
            max_route_hop_count,
            default_route_hop_count_safe,
            default_route_hop_count_unsafe,
        }
    }

    pub fn get_max_route_hop_count(&self) -> usize {
        self.max_route_hop_count
    }

    pub fn get_default_route_hop_count_safe(&self) -> usize {
        self.default_route_hop_count_safe
    }

    pub fn get_default_route_hop_count_unsafe(&self) -> usize {
        self.default_route_hop_count_unsafe
    }

    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip_all, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub fn reset(&self) {
        *self.inner.write() = RouteSpecStoreInner {
            content: RouteSpecStoreContent::default(),
            cache: RouteSpecStoreCache::new(self.registry()),
        };
    }

    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub async fn load(&self) -> EyreResult<()> {
        let inner = {
            let table_store = self.table_store();
            let routing_table = self.routing_table();

            // Get frozen blob from table store
            let content = RouteSpecStoreContent::load(&table_store, &routing_table).await?;

            let mut inner = RouteSpecStoreInner {
                content,
                cache: RouteSpecStoreCache::new(self.registry()),
            };

            // Rebuild the routespecstore cache
            for (_, rssd) in inner.content.iter_details() {
                inner.cache.add_to_cache(rssd);
            }

            inner
        };

        // Return the loaded RouteSpecStore
        *self.inner.write() = inner;

        Ok(())
    }

    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip(self), err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub async fn save(&self) -> EyreResult<()> {
        let content = {
            let inner = self.inner.read();
            inner.content.clone()
        };

        // Save our content
        let table_store = self.table_store();
        content.save(&table_store).await?;

        Ok(())
    }

    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip(self), fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub fn send_route_update(&self) {
        let (dead_routes, dead_remote_routes) = {
            let mut inner = self.inner.write();
            let Some(dr) = inner.cache.take_dead_routes() else {
                // Nothing to do
                return;
            };
            dr
        };

        let update = VeilidUpdate::RouteChange(Box::new(VeilidRouteChange {
            dead_routes,
            dead_remote_routes,
        }));

        let update_callback = self.registry.update_callback();
        update_callback(update);
    }

    /// Purge the route spec store
    pub async fn purge(&self) -> VeilidAPIResult<()> {
        // Briefly pause routing table ticker while changes are made
        let routing_table = self.routing_table();

        let _tick_guard = routing_table.pause_tasks().await;
        routing_table.cancel_tasks().await;
        {
            let inner = &mut *self.inner.write();
            inner.content = Default::default();
            inner.cache = RouteSpecStoreCache::new(self.registry());
        }
        self.save().await.map_err(VeilidAPIError::internal)
    }

    /// Release an allocated or remote route that is no longer in use
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip(self), ret, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub fn release_route(&self, id: RouteId) -> bool {
        let is_remote = self.is_route_id_remote(&id);
        if is_remote {
            self.release_remote_route_id(id)
        } else {
            self.release_allocated_route(id)
        }
    }

    /// List all allocated routes
    pub fn list_allocated_routes<F, R>(&self, mut filter: F) -> Vec<R>
    where
        F: FnMut(&RouteId, &RouteSetSpecDetail) -> Option<R>,
    {
        let inner = self.inner.read();
        let mut out = Vec::with_capacity(inner.content.get_detail_count());
        let mut details = inner.content.iter_details().collect::<Vec<_>>();
        details.sort_by(|a, b| {
            let cmp_hop_count = a.1.hop_count().cmp(&b.1.hop_count());
            if cmp_hop_count != cmp::Ordering::Equal {
                return cmp_hop_count;
            }
            let cmp_avg_latency =
                a.1.get_stats()
                    .latency_stats()
                    .average
                    .cmp(&b.1.get_stats().latency_stats().average);
            if cmp_avg_latency != cmp::Ordering::Equal {
                return cmp_avg_latency;
            }
            a.0.cmp(b.0)
        });

        for detail in details {
            if let Some(x) = filter(detail.0, detail.1) {
                out.push(x);
            }
        }
        out
    }

    /// List all allocated routes
    pub fn list_remote_routes<F, R>(&self, mut filter: F) -> Vec<R>
    where
        F: FnMut(&RouteId, &RemotePrivateRouteInfo) -> Option<R>,
    {
        let inner = self.inner.read();
        let cur_ts = Timestamp::now();
        let remote_route_ids = inner.cache.get_remote_private_route_ids(cur_ts);

        let remote_routes = remote_route_ids
            .iter()
            .filter_map(|id| {
                inner
                    .cache
                    .peek_remote_private_route(cur_ts, id)
                    .map(|x| (id, x))
            })
            .collect::<Vec<_>>();
        let mut out = Vec::with_capacity(remote_routes.len());

        for (id, rpri) in remote_routes {
            if let Some(x) = filter(id, rpri) {
                out.push(x);
            }
        }
        out
    }

    /// Clear caches when our local node info changes
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip(self), fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub fn reset_cache(&self) {
        veilid_log!(self debug "resetting route cache");

        let mut inner = self.inner.write();

        // Clean up allocated routes (does not delete allocated routes, set republication flag)
        inner.content.reset_details();

        // Reset private route cache (does not delete imported routes)
        inner.cache.reset_remote_private_routes();
    }

    /// Mark route as published
    /// When first deserialized, routes must be re-published in order to ensure they remain
    /// in the RouteSpecStore.
    pub fn mark_route_published(&self, id: &RouteId, published: bool) -> VeilidAPIResult<()> {
        let mut inner = self.inner.write();
        let Some(rssd) = inner.content.get_detail_mut(id) else {
            apibail_invalid_target!("route does not exist");
        };
        rssd.set_published(published);
        Ok(())
    }

    /// Convert private route list to binary blob
    pub fn private_routes_to_blob(private_routes: &[PrivateRoute]) -> VeilidAPIResult<Vec<u8>> {
        let mut buffer = vec![];

        // Serialize count
        let pr_count = private_routes.len();
        if pr_count > MAX_CRYPTO_KINDS {
            apibail_internal!("too many crypto kinds to encode blob");
        }
        let pr_count = pr_count as u8;
        buffer.push(pr_count);

        // Serialize stream of private routes
        for private_route in private_routes {
            let mut pr_message = ::capnp::message::Builder::new_default();
            let mut pr_builder = pr_message.init_root::<veilid_capnp::private_route::Builder>();

            encode_private_route(private_route, &mut pr_builder)
                .map_err(VeilidAPIError::internal)?;

            canonical_message_builder_to_write_packed(&mut buffer, pr_message)
                .map_err(VeilidAPIError::internal)?;
        }
        Ok(buffer)
    }

    /// Display debugging for routes by their public key
    pub fn display_route_by_key(&self, key: &PublicKey) -> String {
        if let Some(id) = self.get_route_id_for_key(key) {
            if let Some(s) = self.display_route(&id) {
                format!("{{key={}, id={}, {}}}", key, id, s)
            } else {
                format!("{{key={}, id={}, (route missing)}}", key, id)
            }
        } else {
            format!("{{key={}, id=(missing)", key)
        }
    }

    /// Display debugging for routes by their route id
    #[expect(dead_code)]
    pub fn display_route_by_id(&self, id: &RouteId) -> String {
        if let Some(s) = self.display_route(id) {
            format!("{{id={}, {}}}", id, s)
        } else {
            format!("{{id={}, (route missing)}}", id)
        }
    }

    /// Get the display description of a route
    fn display_route(&self, id: &RouteId) -> Option<String> {
        let inner = self.inner.read();
        let cur_ts = Timestamp::now();
        if let Some(rpri) = inner.cache.peek_remote_private_route(cur_ts, id) {
            return Some(format!("remote: {}", rpri));
        }
        if let Some(rssd) = inner.content.get_detail(id) {
            return Some(format!("allocated: {}", rssd));
        }
        None
    }

    /// Debug debugging for routes by their public key
    pub fn debug_route_by_key(&self, key: &PublicKey) -> String {
        if let Some(id) = self.get_route_id_for_key(key) {
            let s = if let Some(s) = self.debug_route(&id) {
                s
            } else {
                "(route missing)".to_string()
            };
            format!(
                "{{\n    key={},\n    id={},\n{}\n}}",
                key,
                id,
                indent_all_string(&s)
            )
        } else {
            format!("{{\n    key={},\n    id=(missing)\n}}", key)
        }
    }

    /// Debug debugging for routes by their route id
    pub fn debug_route_by_id(&self, id: &RouteId) -> String {
        let s = if let Some(s) = self.debug_route(id) {
            s
        } else {
            "(route missing)".to_string()
        };
        format!("{{\n    id={},\n{}\n}}", id, indent_all_string(&s))
    }

    /// Get the debug description of a route
    fn debug_route(&self, id: &RouteId) -> Option<String> {
        let inner = self.inner.read();
        let cur_ts = Timestamp::now();
        if let Some(rpri) = inner.cache.peek_remote_private_route(cur_ts, id) {
            return Some(format!("remote: {:#?}", rpri));
        }
        if let Some(rssd) = inner.content.get_detail(id) {
            return Some(format!("allocated: {:#?}", rssd));
        }
        None
    }
}
