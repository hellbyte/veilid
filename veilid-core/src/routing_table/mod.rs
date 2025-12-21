mod bucket;
mod bucket_entry;
mod debug;
mod find_peers;
#[cfg(feature = "geolocation")]
mod geolocation;
mod node_ref;
mod privacy;
mod route_spec_store;
mod routing_table_inner;
mod stats_accounting;
mod tasks;
mod types;

pub mod tests;

pub(crate) use bucket_entry::*;
pub(crate) use node_ref::*;
pub(crate) use privacy::*;
pub(crate) use route_spec_store::*;
pub(crate) use routing_table_inner::*;
pub(crate) use stats_accounting::*;
pub use types::*;

use super::*;

use crate::crypto::*;
use crate::network_manager::*;
use crate::rpc_processor::*;

use bucket::*;
use hashlink::LruCache;

impl_veilid_log_facility!("rtab");

//////////////////////////////////////////////////////////////////////////

/// Routing table bucket count (one per bit per 32 byte node id)
pub const BUCKET_COUNT: usize = HASH_COORDINATE_LENGTH * 8;

/// Minimum number of nodes we need, per crypto kind, per routing domain, or we trigger a bootstrap
pub const MIN_BOOTSTRAP_CONNECTIVITY_PEERS: usize = 4;
/// Set of routing domains that use the bootstrap mechanism
pub const BOOTSTRAP_ROUTING_DOMAINS: [RoutingDomain; 1] = [RoutingDomain::PublicInternet];

/// How frequently we tick the relay management routine
pub const RELAY_MANAGEMENT_INTERVAL_SECS: u32 = 1;
/// How frequently we optimize relays
pub const RELAY_OPTIMIZATION_INTERVAL: TimestampDuration = TimestampDuration::new_secs(10);
/// What percentile to keep our relays optimized to
pub const RELAY_OPTIMIZATION_PERCENTILE: f32 = 66.0;
/// What percentile to choose our relays from (must be greater than RELAY_OPTIMIZATION_PERCENTILE)
pub const RELAY_SELECTION_PERCENTILE: f32 = 85.0;

/// How frequently we tick the private route management routine
pub const PRIVATE_ROUTE_MANAGEMENT_INTERVAL_SECS: u32 = 1;

/// How frequently we flush the routing table and route spec store to storage
pub const ROUTING_TABLE_FLUSH_INTERVAL_SECS: u32 = 30;

// Connectionless protocols like UDP are dependent on a NAT translation timeout
// We ping relays to maintain our UDP NAT state with a RELAY_KEEPALIVE_PING_INTERVAL_SECS=10 frequency
// since 30 seconds is a typical UDP NAT state timeout  .
// Non-relay flows are assumed to be alive for half the typical timeout and we regenerate the hole punch
// if it the flow hasn't had any activity in this amount of time.
pub const CONNECTIONLESS_TIMEOUT: TimestampDuration = TimestampDuration::new_secs(15);

// Table store keys
const ALL_ENTRY_BYTES: &[u8] = b"all_entry_bytes";
const ROUTING_TABLE: &str = "routing_table";
const SERIALIZED_BUCKET_MAP: &[u8] = b"serialized_bucket_map";
const CACHE_VALIDITY_KEY: &[u8] = b"cache_validity_key";

pub type RoutingTableEntryFilter<'t> =
    Box<dyn FnMut(&RoutingTableInner, Option<Arc<BucketEntry>>, Timestamp) -> bool + Send + 't>;
pub type RoutingTableEntryPreSortFilter<'t> =
    Box<dyn FnMut(&RoutingTableInner, &mut Vec<Option<Arc<BucketEntry>>>, Timestamp) + Send + 't>;
pub type RoutingTableEntrySort<'t> = Box<
    dyn FnMut(
            &RoutingTableInner,
            Option<Arc<BucketEntry>>,
            Option<Arc<BucketEntry>>,
            Timestamp,
        ) -> core::cmp::Ordering
        + Send
        + 't,
>;

type SerializedBuckets = Vec<Vec<u8>>;
type SerializedBucketMap = BTreeMap<CryptoKind, SerializedBuckets>;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[must_use]
pub struct RoutingTableHealth {
    /// Number of reliable (long-term responsive) entries in the routing table
    pub reliable_entry_count: usize,
    /// Number of unreliable (occasionally unresponsive) entries in the routing table
    pub unreliable_entry_count: usize,
    /// Number of dead (always unresponsive) entries in the routing table
    pub dead_entry_count: usize,
    /// Number of live (responsive) entries in the routing table per RoutingDomain and CryptoKind
    pub live_entry_counts: LiveEntryCounts,
    /// If PublicInternet network class is valid yet
    pub public_internet_ready: bool,
    /// If LocalNetwork network class is valid yet
    pub local_network_ready: bool,
}

pub type BucketIndex = (CryptoKind, usize);

#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct RecentPeersEntry {
    pub last_connection: Flow,
}

#[derive(Debug, Clone)]
pub struct RoutingTableStartupContext {
    pub startup_lock: Arc<StartupLock>,
}
impl RoutingTableStartupContext {
    pub fn new() -> Self {
        Self {
            startup_lock: Arc::new(StartupLock::new()),
        }
    }
}
impl Default for RoutingTableStartupContext {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub(crate) struct RoutingTable {
    registry: VeilidComponentRegistry,
    inner: RwLock<RoutingTableInner>,

    // Startup context
    startup_context: RoutingTableStartupContext,

    /// Node Ids
    node_ids: RwLock<NodeIdGroup>,
    /// Public Keys
    public_keys: RwLock<PublicKeyGroup>,
    /// Secret Keys
    secret_keys: RwLock<SecretKeyGroup>,
    /// Route spec store
    route_spec_store: RouteSpecStore,
    /// Buckets to kick on our next kick task
    kick_queue: Mutex<BTreeSet<BucketIndex>>,
    /// Background process for flushing the table to disk
    flush_task: TickTask<EyreReport>,
    /// Background process for computing statistics
    rolling_transfers_task: TickTask<EyreReport>,
    /// Background process for computing statistics
    update_state_stats_task: TickTask<EyreReport>,
    /// Background process for computing statistics
    rolling_answers_task: TickTask<EyreReport>,
    /// Background process to purge dead routing table entries when necessary
    kick_buckets_task: TickTask<EyreReport>,
    /// Background process to get our initial routing table
    bootstrap_task: TickTask<EyreReport>,
    /// Background process to ensure we have enough nodes in our routing table
    peer_minimum_refresh_task: TickTask<EyreReport>,
    /// Background process to ensure we have enough nodes close to our own in our routing table
    closest_peers_refresh_task: TickTask<EyreReport>,
    /// Background process to check PublicInternet nodes to see if they are still alive and for reliability
    ping_validator_public_internet_task: TickTask<EyreReport>,
    /// Background process to check LocalNetwork nodes to see if they are still alive and for reliability
    ping_validator_local_network_task: TickTask<EyreReport>,
    /// Background process to check PublicInternet relay nodes to see if they are still alive and for reliability
    ping_validator_public_internet_relay_task: TickTask<EyreReport>,
    /// Background process to check Active Watch nodes to see if they are still alive and for reliability
    ping_validator_active_watch_task: TickTask<EyreReport>,
    /// Background process to keep relays up
    relay_management_task: TickTask<EyreReport>,
    /// Background process to keep private routes up
    private_route_management_task: TickTask<EyreReport>,
    /// Tick subscription
    tick_subscription: Mutex<Option<EventBusSubscription>>,
}

impl fmt::Debug for RoutingTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RoutingTable")
            // .field("inner", &self.inner)
            // .field("unlocked_inner", &self.unlocked_inner)
            .finish()
    }
}

impl_veilid_component!(RoutingTable);

impl RoutingTable {
    pub fn new(
        registry: VeilidComponentRegistry,
        startup_context: RoutingTableStartupContext,
    ) -> Self {
        let config = registry.config();
        let inner = RwLock::new(RoutingTableInner::new(registry.clone()));
        let route_spec_store = RouteSpecStore::new(registry.clone());
        let this = Self {
            registry,
            inner,
            startup_context,
            node_ids: RwLock::new(NodeIdGroup::new()),
            public_keys: RwLock::new(PublicKeyGroup::new()),
            secret_keys: RwLock::new(SecretKeyGroup::new()),
            route_spec_store,
            kick_queue: Mutex::new(BTreeSet::default()),
            flush_task: TickTask::new("flush_task", ROUTING_TABLE_FLUSH_INTERVAL_SECS),
            rolling_transfers_task: TickTask::new(
                "rolling_transfers_task",
                ROLLING_TRANSFERS_INTERVAL_SECS,
            ),
            update_state_stats_task: TickTask::new(
                "update_state_stats_task",
                UPDATE_STATE_STATS_INTERVAL_SECS,
            ),
            rolling_answers_task: TickTask::new(
                "rolling_answers_task",
                ROLLING_ANSWER_INTERVAL_SECS,
            ),
            kick_buckets_task: TickTask::new("kick_buckets_task", 1),
            bootstrap_task: TickTask::new("bootstrap_task", 1),
            peer_minimum_refresh_task: TickTask::new("peer_minimum_refresh_task", 1),
            closest_peers_refresh_task: TickTask::new_ms(
                "closest_peers_refresh_task",
                config.network.dht.min_peer_refresh_time_ms,
            ),
            ping_validator_public_internet_task: TickTask::new(
                "ping_validator_public_internet_task",
                1,
            ),
            ping_validator_local_network_task: TickTask::new(
                "ping_validator_local_network_task",
                1,
            ),
            ping_validator_public_internet_relay_task: TickTask::new(
                "ping_validator_public_internet_relay_task",
                1,
            ),
            ping_validator_active_watch_task: TickTask::new("ping_validator_active_watch_task", 1),
            relay_management_task: TickTask::new(
                "relay_management_task",
                RELAY_MANAGEMENT_INTERVAL_SECS,
            ),
            private_route_management_task: TickTask::new(
                "private_route_management_task",
                PRIVATE_ROUTE_MANAGEMENT_INTERVAL_SECS,
            ),
            tick_subscription: Mutex::new(None),
        };

        this.setup_tasks();

        this
    }

    /////////////////////////////////////
    // Initialization

    /// Called to initialize the routing table after it is created
    async fn init_async(&self) -> EyreResult<()> {
        veilid_log!(self debug "starting routing table init");

        // Set up initial keys and node ids
        self.setup_public_keys().await?;

        // Set up routing buckets
        {
            let mut inner = self.inner.write();

            inner.init_buckets();
        }

        // Load bucket entries from table db if possible
        veilid_log!(self debug "loading routing table entries");
        if let Err(e) = self.load_buckets().await {
            veilid_log!(self debug "Error loading buckets from storage: {:#?}. Resetting.", e);
            let mut inner = self.inner.write();
            inner.init_buckets();
        }

        // Set up routespecstore
        veilid_log!(self debug "starting route spec store init");
        if let Err(e) = self.route_spec_store().load().await {
            veilid_log!(self debug "Error loading route spec store: {:#?}. Resetting.", e);
            self.route_spec_store().reset();
        };
        veilid_log!(self debug "finished route spec store init");

        veilid_log!(self debug "finished routing table init");
        Ok(())
    }

    #[expect(clippy::unused_async)]
    async fn post_init_async(&self) -> EyreResult<()> {
        Ok(())
    }

    #[expect(clippy::unused_async)]
    pub(crate) async fn startup(&self) -> EyreResult<()> {
        let guard = self.startup_context.startup_lock.startup()?;

        // Register event handlers
        let tick_subscription = impl_subscribe_event_bus_async!(self, Self, tick_event_handler);

        *self.tick_subscription.lock() = Some(tick_subscription);

        guard.success();
        Ok(())
    }

    pub(crate) async fn shutdown(&self) {
        // Stop tasks
        veilid_log!(self debug "stopping routing table tasks");

        if let Some(sub) = self.tick_subscription.lock().take() {
            self.event_bus().unsubscribe(sub);
        }

        self.cancel_tasks().await;

        let guard = self
            .startup_context
            .startup_lock
            .shutdown()
            .await
            .expect("should be started up");

        // Unpublish peer info
        veilid_log!(self debug "unpublishing peer info");
        {
            let mut inner = self.inner.write();
            for routing_domain in RoutingDomainSet::all() {
                inner.unpublish_peer_info(routing_domain);
            }
        }

        guard.success();
    }

    #[expect(clippy::unused_async)]
    async fn pre_terminate_async(&self) {
        // Ensure things have shut down
        assert!(
            self.startup_context.startup_lock.is_shut_down(),
            "should have shut down by now"
        );
    }

    /// Called to shut down the routing table
    async fn terminate_async(&self) {
        veilid_log!(self debug "starting routing table terminate");

        veilid_log!(self debug "routing table termination flush");
        self.flush().await;

        veilid_log!(self debug "shutting down routing table");

        let mut inner = self.inner.write();
        *inner = RoutingTableInner::new(self.registry());

        self.node_ids.write().clear();

        veilid_log!(self debug "finished routing table terminate");
    }

    pub async fn flush(&self) {
        if let Err(e) = self.save_buckets().await {
            error!("failed to save routing table entries: {}", e);
        }

        if let Err(e) = self.route_spec_store().save().await {
            error!("couldn't save route spec store: {}", e);
        }
    }

    ///////////////////////////////////////////////////////////////////

    pub fn node_id(&self, kind: CryptoKind) -> NodeId {
        self.node_ids.read().get(kind).unwrap()
    }

    pub fn public_key(&self, kind: CryptoKind) -> PublicKey {
        self.public_keys.read().get(kind).unwrap()
    }

    pub fn secret_key(&self, kind: CryptoKind) -> SecretKey {
        self.secret_keys.read().get(kind).unwrap()
    }

    pub fn node_ids(&self) -> NodeIdGroup {
        self.node_ids.read().clone()
    }

    pub fn public_keys(&self) -> PublicKeyGroup {
        self.public_keys.read().clone()
    }

    pub fn signing_key_pairs(&self) -> KeyPairGroup {
        let mut tkps = KeyPairGroup::new();
        for ck in VALID_CRYPTO_KINDS {
            tkps.add(KeyPair::new(
                ck,
                BareKeyPair::new(self.public_key(ck).value(), self.secret_key(ck).value()),
            ));
        }
        tkps
    }

    pub fn matches_own_node_id(&self, node_ids: &[NodeId]) -> bool {
        for ni in node_ids {
            if let Some(v) = self.node_ids().get(ni.kind()) {
                if v.ref_value() == ni.ref_value() {
                    return true;
                }
            }
        }
        false
    }

    pub fn matches_own_public_key(&self, public_keys: &[PublicKey]) -> bool {
        for pk in public_keys {
            if let Some(v) = self.public_keys().get(pk.kind()) {
                if v.ref_value() == pk.ref_value() {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(not(test))]
    async fn setup_public_key(
        &self,
        vcrypto: AsyncCryptoSystemGuard<'_>,
    ) -> VeilidAPIResult<(PublicKey, SecretKey)> {
        let config = self.config();
        let table_store = self.table_store();
        let ck = vcrypto.kind();
        let mut public_key = config.network.routing_table.public_keys.get(ck);
        let mut secret_key = config.network.routing_table.secret_keys.get(ck);

        let config_table = table_store.open("__veilid_config", 1).await?;

        // Old pre-0.5.0 locations
        let table_key_node_id = format!("node_id_{}", ck);
        let table_key_node_id_secret = format!("node_id_secret_{}", ck);
        // Post-0.5.0 locations
        let table_key_public_key = format!("public_key_{}", ck);
        let table_key_secret_key = format!("secret_key_{}", ck);

        // See if public key was previously stored in the table store
        if public_key.is_none() {
            veilid_log!(self debug "pulling {} from storage", table_key_public_key);
            if let Ok(Some(stored_public_key)) = config_table
                .load_json::<PublicKey>(0, table_key_public_key.as_bytes())
                .await
            {
                veilid_log!(self debug "{} found in storage", table_key_public_key);
                public_key = Some(stored_public_key);
            } else {
                veilid_log!(self debug "{} not found in storage", table_key_public_key);
            }
        }
        if public_key.is_none() {
            veilid_log!(self debug "pulling {} from storage", table_key_node_id);
            if let Ok(Some(stored_public_key)) = config_table
                .load_json::<PublicKey>(0, table_key_node_id.as_bytes())
                .await
            {
                veilid_log!(self debug "{} found in storage", table_key_node_id);
                public_key = Some(stored_public_key);
            } else {
                veilid_log!(self debug "{} not found in storage", table_key_node_id);
            }
        }

        // See if secret key was previously stored in the table store
        if secret_key.is_none() {
            veilid_log!(self debug "pulling {} from storage", table_key_secret_key);
            if let Ok(Some(stored_secret_key)) = config_table
                .load_json::<SecretKey>(0, table_key_secret_key.as_bytes())
                .await
            {
                veilid_log!(self debug "{} found in storage", table_key_secret_key);
                secret_key = Some(stored_secret_key);
            } else {
                veilid_log!(self debug "{} not found in storage", table_key_secret_key);
            }
        }
        if secret_key.is_none() {
            veilid_log!(self debug "pulling {} from storage", table_key_node_id_secret);
            if let Ok(Some(stored_secret_key)) = config_table
                .load_json::<SecretKey>(0, table_key_node_id_secret.as_bytes())
                .await
            {
                veilid_log!(self debug "{} found in storage", table_key_node_id_secret);
                secret_key = Some(stored_secret_key);
            } else {
                veilid_log!(self debug "{} not found in storage", table_key_node_id_secret);
            }
        }

        // If we have a public key from storage, check it
        let (public_key, secret_key) =
            if let (Some(public_key), Some(secret_key)) = (public_key, secret_key) {
                // Validate node id
                if !vcrypto.validate_keypair(&public_key, &secret_key).await? {
                    apibail_generic!(
                        "secret_key and public_key don't match:\npublic_key: {}\nsecret_key: {}",
                        public_key,
                        secret_key
                    );
                }
                (public_key, secret_key)
            } else {
                // If we still don't have a valid keypair, generate one
                veilid_log!(self debug "generating new node {} keypair", ck);
                vcrypto.generate_keypair().await.into_split()
            };

        // Save the public key + secret in storage
        config_table
            .store_json(0, table_key_public_key.as_bytes(), &public_key)
            .await?;
        config_table
            .store_json(0, table_key_secret_key.as_bytes(), &secret_key)
            .await?;

        Ok((public_key, secret_key))
    }

    /// Get the public keys from config if one is specified
    #[cfg_attr(test, allow(unused_variables))]
    async fn setup_public_keys(&self) -> VeilidAPIResult<()> {
        let crypto = self.crypto();

        let mut out_public_keys = PublicKeyGroup::new();
        let mut out_secret_keys = SecretKeyGroup::new();

        for ck in VALID_CRYPTO_KINDS {
            let vcrypto = crypto
                .get_async(ck)
                .expect("Valid crypto kind is not actually valid.");

            #[cfg(test)]
            let (public_key, secret_key) = vcrypto.generate_keypair().await.into_split();
            #[cfg(not(test))]
            let (public_key, secret_key) = self.setup_public_key(vcrypto).await?;

            // Save for config
            out_public_keys.add(public_key);
            out_secret_keys.add(secret_key);
        }

        veilid_log!(self info  "Public Keys: {}", out_public_keys);

        *self.public_keys.write() = out_public_keys;
        *self.secret_keys.write() = out_secret_keys;

        // Set up node ids
        let mut node_ids = NodeIdGroup::new();
        for pk in self.public_keys().iter() {
            let node_id = self.generate_node_id(pk)?;
            node_ids.add(node_id);
        }

        veilid_log!(self info "Node Ids: {}", node_ids);

        *self.node_ids.write() = node_ids;

        Ok(())
    }

    // Convenience validators
    pub fn check_route_id(&self, route_id: &RouteId) -> VeilidAPIResult<()> {
        let crypto = self.crypto();
        let Some(vcrypto) = crypto.get(route_id.kind()) else {
            apibail_generic!("unsupported crypto kind");
        };
        if route_id.ref_value().len() != vcrypto.hash_digest_length() {
            apibail_generic!("invalid route id length");
        }
        Ok(())
    }
    pub fn check_node_id(&self, node_id: &NodeId) -> VeilidAPIResult<()> {
        let crypto = self.crypto();
        let Some(_) = crypto.get(node_id.kind()) else {
            apibail_generic!("unsupported crypto kind");
        };
        if node_id.ref_value().len() != HASH_COORDINATE_LENGTH {
            apibail_generic!("invalid node id length");
        }
        Ok(())
    }

    /// Produce node id from public key
    pub fn generate_node_id(&self, public_key: &PublicKey) -> VeilidAPIResult<NodeId> {
        if public_key.ref_value().len() == HASH_COORDINATE_LENGTH {
            return Ok(NodeId::new(
                public_key.kind(),
                BareNodeId::new(public_key.ref_value()),
            ));
        }
        let crypto = self.crypto();
        let Some(vcrypto) = crypto.get(public_key.kind()) else {
            apibail_generic!("unsupported cryptosystem");
        };

        let idhash = vcrypto.generate_hash(public_key.ref_value());
        assert!(
            idhash.ref_value().len() >= HASH_COORDINATE_LENGTH,
            "generate_hash needs to produce at least {} bytes",
            HASH_COORDINATE_LENGTH
        );
        Ok(NodeId::new(
            public_key.kind(),
            BareNodeId::new(&idhash.ref_value()[0..HASH_COORDINATE_LENGTH]),
        ))
    }

    pub fn calculate_bucket_index(&self, node_id: &NodeId) -> EyreResult<BucketIndex> {
        if node_id.ref_value().len() * 8 != BUCKET_COUNT {
            bail!("NodeId should be hashed down to BUCKET_COUNT bits");
        }
        let self_hash_coordinate = self.node_id(node_id.kind()).to_hash_coordinate();
        Ok((
            node_id.kind(),
            node_id
                .to_hash_coordinate()
                .distance(&self_hash_coordinate)
                .first_nonzero_bit()
                .unwrap(),
        ))
    }

    /// Serialize the routing table.
    fn serialized_buckets(&self) -> (SerializedBucketMap, SerializedBuckets) {
        // Since entries are shared by multiple buckets per cryptokind
        // we need to get the list of all unique entries when serializing
        let mut all_entries: Vec<Arc<BucketEntry>> = Vec::new();

        // Serialize all buckets and get map of entries
        let mut serialized_bucket_map: SerializedBucketMap = BTreeMap::new();
        {
            let mut entry_map: HashMap<*const BucketEntry, u32> = HashMap::new();
            let inner = &*self.inner.read();
            for ck in VALID_CRYPTO_KINDS {
                let buckets = inner.buckets.get(&ck).unwrap();
                let mut serialized_buckets = Vec::new();
                for bucket in buckets.iter() {
                    serialized_buckets.push(bucket.save_bucket(&mut all_entries, &mut entry_map))
                }
                serialized_bucket_map.insert(ck, serialized_buckets);
            }
        }

        // Serialize all the entries
        let mut all_entry_bytes = Vec::with_capacity(all_entries.len());
        for entry in all_entries {
            // Serialize entry
            let entry_bytes = entry.with_inner(|e| serialize_json_bytes(e));
            all_entry_bytes.push(entry_bytes);
        }

        (serialized_bucket_map, all_entry_bytes)
    }

    /// Write the serialized routing table to the table store.
    async fn save_buckets(&self) -> EyreResult<()> {
        let (serialized_bucket_map, all_entry_bytes) = self.serialized_buckets();

        let table_store = self.table_store();
        let tdb = table_store.open(ROUTING_TABLE, 1).await?;
        let dbx = tdb.transact();
        if let Err(e) = dbx
            .store_json(0, SERIALIZED_BUCKET_MAP, &serialized_bucket_map)
            .await
        {
            dbx.rollback();
            return Err(e.into());
        }
        if let Err(e) = dbx.store_json(0, ALL_ENTRY_BYTES, &all_entry_bytes).await {
            dbx.rollback();
            return Err(e.into());
        }
        dbx.commit().await?;
        Ok(())
    }

    /// Deserialize routing table from table store
    async fn load_buckets(&self) -> EyreResult<()> {
        // Make a cache validity key of all our node ids and our bootstrap choice
        let mut cache_validity_key: Vec<u8> = Vec::new();
        {
            let config = self.config();
            for ck in VALID_CRYPTO_KINDS {
                if let Some(nid) = config.network.routing_table.public_keys.get(ck) {
                    cache_validity_key.extend_from_slice(nid.ref_value());
                }
            }
            for b in &config.network.routing_table.bootstrap {
                cache_validity_key.extend_from_slice(b.as_bytes());
            }
            cache_validity_key.extend_from_slice(
                config
                    .network
                    .network_key_password
                    .clone()
                    .unwrap_or_default()
                    .as_bytes(),
            );
        };

        // Deserialize bucket map and all entries from the table store
        let table_store = self.table_store();
        let db = table_store.open(ROUTING_TABLE, 1).await?;

        let caches_valid = match db.load(0, CACHE_VALIDITY_KEY).await? {
            Some(v) => v == cache_validity_key,
            None => false,
        };
        if !caches_valid {
            // Caches not valid, start over
            veilid_log!(self debug "cache validity key changed, emptying routing table");
            drop(db);
            table_store.delete(ROUTING_TABLE).await?;
            let db = table_store.open(ROUTING_TABLE, 1).await?;
            db.store(0, CACHE_VALIDITY_KEY, &cache_validity_key).await?;
            return Ok(());
        }

        // Caches valid, load saved routing table
        let Some(serialized_bucket_map): Option<SerializedBucketMap> =
            db.load_json(0, SERIALIZED_BUCKET_MAP).await?
        else {
            veilid_log!(self debug "no bucket map in saved routing table");
            return Ok(());
        };
        let Some(all_entry_bytes): Option<SerializedBuckets> =
            db.load_json(0, ALL_ENTRY_BYTES).await?
        else {
            veilid_log!(self debug "no all_entry_bytes in saved routing table");
            return Ok(());
        };

        // Reconstruct all entries
        let inner = &mut *self.inner.write();
        Self::populate_routing_table_inner(inner, serialized_bucket_map, all_entry_bytes)?;

        Ok(())
    }

    /// Write the deserialized table store data to the routing table.
    pub fn populate_routing_table_inner(
        inner: &mut RoutingTableInner,
        serialized_bucket_map: SerializedBucketMap,
        all_entry_bytes: SerializedBuckets,
    ) -> EyreResult<()> {
        let mut all_entries: Vec<Arc<BucketEntry>> = Vec::with_capacity(all_entry_bytes.len());
        for entry_bytes in all_entry_bytes {
            #[allow(unused_mut)]
            let mut entryinner: BucketEntryInner = deserialize_json_bytes(&entry_bytes)
                .wrap_err("failed to deserialize bucket entry")?;

            #[cfg(feature = "geolocation")]
            {
                entryinner.update_geolocation_info();
            }

            let entry = Arc::new(BucketEntry::new_with_inner(entryinner));

            // Keep strong reference in table
            all_entries.push(entry.clone());

            // Keep all entries in weak table too
            inner.all_entries.insert(entry);
        }

        // Validate serialized bucket map
        for (k, v) in &serialized_bucket_map {
            if !VALID_CRYPTO_KINDS.contains(k) {
                veilid_log!(inner warn "crypto kind is not valid, not loading routing table");
                return Ok(());
            }
            if v.len() != BUCKET_COUNT {
                veilid_log!(inner warn "bucket count is different, not loading routing table");
                return Ok(());
            }
        }

        // Recreate buckets
        for (k, v) in serialized_bucket_map {
            let buckets = inner.buckets.get_mut(&k).unwrap();

            for n in 0..v.len() {
                buckets[n].load_bucket(v[n].clone(), &all_entries)?;
            }
        }

        Ok(())
    }

    /////////////////////////////////////
    // Locked operations

    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), expect(dead_code))]
    pub fn routing_domain_for_address(&self, address: Address) -> Option<RoutingDomain> {
        self.inner.read().routing_domain_for_address(address)
    }

    pub fn routing_domain_for_flow(&self, flow: Flow) -> Option<RoutingDomain> {
        self.inner.read().routing_domain_for_flow(flow)
    }

    pub fn route_spec_store(&self) -> &RouteSpecStore {
        &self.route_spec_store
    }

    pub fn relays(&self, domain: RoutingDomain) -> Vec<RoutingDomainRelay> {
        self.inner.read().relays(domain)
    }

    pub fn relays_and_states(
        &self,
        domain: RoutingDomain,
    ) -> Vec<(RoutingDomainRelay, RoutingDomainRelayState)> {
        self.inner.read().relays_and_states(domain)
    }

    pub fn routing_domain_state(&self, domain: RoutingDomain) -> RoutingDomainState {
        self.inner.read().routing_domain_state(domain)
    }

    pub fn dial_info_details(&self, domain: RoutingDomain) -> Vec<DialInfoDetail> {
        self.inner.read().dial_info_details(domain)
    }

    pub fn routing_domain_debug(&self, domain: RoutingDomain, alt: bool) -> String {
        self.inner
            .read()
            .with_routing_domain(domain, |_rti, rdd| rdd.debug(alt))
    }

    #[expect(dead_code)]
    pub fn all_filtered_dial_info_details(
        &self,
        routing_domain_set: RoutingDomainSet,
        filter: &DialInfoFilter,
    ) -> Vec<DialInfoDetail> {
        self.inner
            .read()
            .all_filtered_dial_info_details(routing_domain_set, filter)
    }
    pub fn get_node_info_routing_domains(&self, node_info: &NodeInfo) -> RoutingDomainSet {
        self.inner.read().get_node_info_routing_domains(node_info)
    }
    pub fn origin_routing_domains(&self, routing_domain: RoutingDomain) -> RoutingDomainSet {
        self.inner.read().origin_routing_domains(routing_domain)
    }

    /// Look up the best way for two nodes to reach each other over a specific routing domain
    pub fn get_contact_method(
        &self,
        routing_domain: RoutingDomain,
        peer_a: Arc<PeerInfo>,
        peer_b: Arc<PeerInfo>,
        dial_info_filter: DialInfoFilter,
        sequencing: Sequencing,
        context_sort: Option<&DialInfoDetailSort>,
    ) -> ContactMethod {
        self.inner.read().get_contact_method(
            routing_domain,
            peer_a,
            peer_b,
            dial_info_filter,
            sequencing,
            context_sort,
        )
    }

    /// Edit the PublicInternet RoutingDomain
    pub fn edit_public_internet_routing_domain(&self) -> RoutingDomainEditorPublicInternet<'_> {
        RoutingDomainEditorPublicInternet::new(self)
    }

    /// Edit the LocalNetwork RoutingDomain
    pub fn edit_local_network_routing_domain(&self) -> RoutingDomainEditorLocalNetwork<'_> {
        RoutingDomainEditorLocalNetwork::new(self)
    }

    /// Return a copy of our node's peerinfo (may not yet be published)
    pub fn get_published_peer_info(&self, routing_domain: RoutingDomain) -> Option<Arc<PeerInfo>> {
        self.inner.read().get_published_peer_info(routing_domain)
    }

    /// Return a copy of our node's peerinfo (may not yet be published)
    pub fn get_current_peer_info(&self, routing_domain: RoutingDomain) -> Arc<PeerInfo> {
        self.inner.read().get_current_peer_info(routing_domain)
    }

    /// Return a list of the current valid bootstrap peers in a particular routing domain
    #[expect(dead_code)]
    pub fn get_bootstrap_peers(&self, routing_domain: RoutingDomain) -> Vec<NodeRef> {
        self.inner.read().get_bootstrap_peers(routing_domain)
    }

    /// Return the domain's filter for what we can send out in the form of a dial info filter
    pub fn get_outbound_dial_info_filter(&self, routing_domain: RoutingDomain) -> DialInfoFilter {
        self.inner
            .read()
            .get_outbound_dial_info_filter(routing_domain)
    }
    /// Return the domain's filter for what we can receive in the form of a node ref filter
    pub fn get_outbound_node_ref_filter(&self, routing_domain: RoutingDomain) -> NodeRefFilter {
        self.inner
            .read()
            .get_outbound_node_ref_filter(routing_domain)
    }

    /// Attempt to empty the routing table
    /// May not empty buckets completely if there are existing node_refs
    pub fn purge_buckets(&self) {
        self.inner.write().purge_buckets();
    }

    /// Attempt to remove last_connections from entries
    pub fn purge_last_connections(&self) {
        self.inner.write().purge_last_connections();
    }

    /// See which nodes need to be pinged
    pub fn get_nodes_needing_ping(
        &self,
        routing_domain: RoutingDomain,
        cur_ts: Timestamp,
    ) -> Vec<FilteredNodeRef> {
        self.inner
            .read()
            .get_nodes_needing_ping(routing_domain, cur_ts)
    }

    fn queue_bucket_kicks(&self, node_ids: NodeIdGroup) {
        for node_id in node_ids.iter() {
            // Skip node ids we didn't add to buckets
            if !VALID_CRYPTO_KINDS.contains(&node_id.kind()) {
                continue;
            }

            // Put it in the kick queue
            let x = self
                .calculate_bucket_index(node_id)
                .expect("node ids should already be the right length");
            self.kick_queue.lock().insert(x);
        }
    }

    /// Resolve an existing routing table entry using any crypto kind and return a reference to it
    pub fn lookup_any_node_ref(&self, node_id_key: BareNodeId) -> EyreResult<Option<NodeRef>> {
        self.inner.read().lookup_bare_node_ref(node_id_key)
    }

    /// Resolve an existing routing table entry and return a reference to it
    pub fn lookup_node_ref(&self, node_id: NodeId) -> EyreResult<Option<NodeRef>> {
        self.inner.read().lookup_node_ref(node_id)
    }

    /// Resolve an existing routing table entry and return a filtered reference to it
    #[instrument(level = "trace", skip_all)]
    pub fn lookup_and_filter_noderef(
        &self,
        node_id: NodeId,
        routing_domain_set: RoutingDomainSet,
        dial_info_filter: DialInfoFilter,
    ) -> EyreResult<Option<FilteredNodeRef>> {
        self.inner
            .read()
            .lookup_and_filter_noderef(node_id, routing_domain_set, dial_info_filter)
    }

    /// Shortcut function to add a node to our routing table if it doesn't exist
    /// and add the dial info we have for it. Returns a noderef filtered to
    /// the routing domain in which this node was registered for convenience.
    #[instrument(level = "trace", skip_all, err)]
    pub fn register_node_with_peer_info(
        &self,
        peer_info: Arc<PeerInfo>,
        allow_invalid: bool,
    ) -> EyreResult<FilteredNodeRef> {
        self.inner
            .write()
            .register_node_with_peer_info(peer_info, allow_invalid)
    }

    /// Shortcut function to add a node to our routing table if it doesn't exist
    /// Returns a noderef filtered to
    /// the routing domain in which this node was registered for convenience.
    #[instrument(level = "trace", skip_all, err)]
    pub fn register_node_with_id(
        &self,
        routing_domain: RoutingDomain,
        node_id: NodeId,
        timestamp: Timestamp,
    ) -> EyreResult<FilteredNodeRef> {
        self.inner
            .write()
            .register_node_with_id(routing_domain, node_id, timestamp)
    }

    //////////////////////////////////////////////////////////////////////
    // Routing Table Health Metrics

    pub fn get_routing_table_health(&self) -> RoutingTableHealth {
        self.inner.read().get_routing_table_health()
    }

    pub fn cached_live_entry_counts(&self) -> Arc<LiveEntryCounts> {
        self.inner.read().cached_live_entry_counts()
    }

    #[instrument(level = "trace", skip_all)]
    pub fn get_recent_peers(&self) -> Vec<(NodeId, RecentPeersEntry)> {
        let mut recent_peers = Vec::new();
        let mut dead_peers = Vec::new();
        let mut out = Vec::new();

        // collect all recent peers
        {
            let inner = self.inner.read();
            for (k, _v) in &inner.recent_peers {
                recent_peers.push(k.clone());
            }
        }

        // look up each node and make sure the connection is still live
        // (uses same logic as send_data, ensuring last_connection works for UDP)
        for node_id in &recent_peers {
            let mut dead = true;
            if let Ok(Some(nr)) = self.lookup_node_ref(node_id.clone()) {
                if let Some(last_connection) = nr.last_flow() {
                    out.push((node_id.clone(), RecentPeersEntry { last_connection }));
                    dead = false;
                }
            }
            if dead {
                dead_peers.push(node_id);
            }
        }

        // purge dead recent peers
        {
            let mut inner = self.inner.write();
            for d in dead_peers {
                inner.recent_peers.remove(d);
            }
        }

        out
    }

    pub fn clear_punishments(&self) {
        let cur_ts = Timestamp::now();
        self.inner
            .write()
            .with_entries_mut(cur_ts, BucketEntryState::Punished, |rti, e| {
                e.with_mut(rti, |_rti, ei| ei.set_punished(None));
                Option::<()>::None
            });
    }

    //////////////////////////////////////////////////////////////////////
    // Find Nodes

    /// Makes a filter that finds nodes with a matching inbound dialinfo
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), expect(dead_code))]
    pub fn make_inbound_dial_info_entry_filter<'a>(
        routing_domain: RoutingDomain,
        dial_info_filter: DialInfoFilter,
    ) -> RoutingTableEntryFilter<'a> {
        // does it have matching public dial info?
        Box::new(move |rti, e, _cur_ts| {
            if let Some(e) = e {
                e.with(rti, |_rti, e| {
                    if let Some(ni) = e.node_info(routing_domain) {
                        if ni
                            .first_filtered_dial_info_detail(DialInfoDetail::NO_SORT, &|did| {
                                did.matches_filter(&dial_info_filter)
                            })
                            .is_some()
                        {
                            return true;
                        }
                    }
                    false
                })
            } else {
                rti.first_filtered_dial_info_detail(routing_domain.into(), &dial_info_filter)
                    .is_some()
            }
        })
    }

    /// Makes a filter that finds nodes capable of dialing a particular outbound dialinfo
    pub fn make_outbound_dial_info_entry_filter<'a>(
        routing_domain: RoutingDomain,
        dial_info: DialInfo,
    ) -> RoutingTableEntryFilter<'a> {
        // does the node's outbound capabilities match the dialinfo?
        Box::new(move |rti, e, _cur_ts| {
            if let Some(e) = e {
                e.with(rti, |_rti, e| {
                    if let Some(ni) = e.node_info(routing_domain) {
                        let dif = DialInfoFilter::all()
                            .with_protocol_type_set(ni.outbound_protocols())
                            .with_address_type_set(ni.address_types());
                        if dial_info.matches_filter(&dif) {
                            return true;
                        }
                    }
                    false
                })
            } else {
                let dif = rti.get_outbound_dial_info_filter(routing_domain);
                dial_info.matches_filter(&dif)
            }
        })
    }

    pub fn find_fast_non_local_nodes_filtered(
        &self,
        routing_domain: RoutingDomain,
        node_count: usize,
        filters: VecDeque<RoutingTableEntryFilter>,
    ) -> Vec<NodeRef> {
        self.inner.read().find_fast_non_local_nodes_filtered(
            self.registry(),
            routing_domain,
            node_count,
            filters,
        )
    }

    pub fn find_preferred_fastest_nodes<'a, T, O>(
        &self,
        node_count: usize,
        filters: VecDeque<RoutingTableEntryFilter>,
        transform: T,
    ) -> Vec<O>
    where
        T: for<'r> FnMut(&'r RoutingTableInner, Option<Arc<BucketEntry>>) -> O + Send,
    {
        self.inner
            .read()
            .find_preferred_fastest_nodes(node_count, filters, transform)
    }

    pub fn find_preferred_closest_nodes<'a, T, O>(
        &self,
        node_count: usize,
        hash_coordinate: HashCoordinate,
        filters: VecDeque<RoutingTableEntryFilter>,
        transform: T,
    ) -> VeilidAPIResult<Vec<O>>
    where
        T: for<'r> FnMut(&'r RoutingTableInner, Option<Arc<BucketEntry>>) -> O + Send,
    {
        self.inner.read().find_preferred_closest_nodes(
            node_count,
            hash_coordinate,
            filters,
            transform,
        )
    }

    #[expect(dead_code)]
    pub fn sort_and_clean_closest_noderefs(
        &self,
        hash_coordinate: HashCoordinate,
        closest_nodes: &[NodeRef],
    ) -> Vec<NodeRef> {
        self.inner
            .read()
            .sort_and_clean_closest_noderefs(hash_coordinate, closest_nodes)
    }

    #[instrument(level = "trace", skip(self, peer_info_list))]
    pub fn register_nodes_with_peer_info_list(
        &self,
        peer_info_list: Vec<Arc<PeerInfo>>,
    ) -> Vec<NodeRef> {
        // Register nodes we'd found
        let mut out = Vec::<NodeRef>::with_capacity(peer_info_list.len());
        for p in peer_info_list {
            // Don't register our own node
            if self.matches_own_node_id(p.node_ids()) {
                continue;
            }

            // Register the node if it's new
            match self.register_node_with_peer_info(p, false) {
                Ok(nr) => out.push(nr.unfiltered()),
                Err(e) => {
                    veilid_log!(self debug "failed to register node with peer info from find node answer: {}", e);
                }
            }
        }
        out
    }

    /// Finds nodes near a particular node id
    /// Ensures all returned nodes have a set of capabilities enabled
    #[instrument(level = "trace", skip(self), err)]
    pub async fn find_nodes_close_to_node_id(
        &self,
        node_ref: FilteredNodeRef,
        node_id: NodeId,
        capabilities: Vec<VeilidCapability>,
    ) -> EyreResult<NetworkResult<Vec<NodeRef>>> {
        let rpc_processor = self.rpc_processor();

        let res = network_result_try!(
            Box::pin(rpc_processor.rpc_call_find_node(
                Destination::direct(node_ref),
                node_id,
                capabilities
            ))
            .await?
        );

        // register nodes we'd found
        Ok(NetworkResult::value(
            self.register_nodes_with_peer_info_list(res.answer),
        ))
    }

    /// Ask a remote node to list the nodes it has around the current node
    /// Ensures all returned nodes have a set of capabilities enabled
    #[instrument(level = "trace", skip(self), err)]
    pub async fn find_nodes_close_to_self(
        &self,
        crypto_kind: CryptoKind,
        node_ref: FilteredNodeRef,
        capabilities: Vec<VeilidCapability>,
    ) -> EyreResult<NetworkResult<Vec<NodeRef>>> {
        let self_node_id = self.node_id(crypto_kind);
        Box::pin(self.find_nodes_close_to_node_id(node_ref, self_node_id, capabilities)).await
    }

    /// Ask a remote node to list the nodes it has around itself
    /// Ensures all returned nodes have a set of capabilities enabled
    #[instrument(level = "trace", skip(self), err)]
    pub async fn find_nodes_close_to_node_ref(
        &self,
        crypto_kind: CryptoKind,
        node_ref: FilteredNodeRef,
        capabilities: Vec<VeilidCapability>,
    ) -> EyreResult<NetworkResult<Vec<NodeRef>>> {
        let Some(target_node_id) = node_ref.node_ids().get(crypto_kind) else {
            bail!("no target node ids for this crypto kind");
        };
        Box::pin(self.find_nodes_close_to_node_id(node_ref, target_node_id, capabilities)).await
    }

    /// Ask node to 'find node' on own node so we can get some more nodes near ourselves
    /// and then contact those nodes to inform -them- that we exist
    #[instrument(level = "trace", skip(self))]
    pub async fn reverse_find_node(
        &self,
        crypto_kind: CryptoKind,
        node_ref: NodeRef,
        wide: bool,
        capabilities: Vec<VeilidCapability>,
    ) {
        // Ask node for nodes closest to our own node
        let closest_nodes = network_result_value_or_log!(self match pin_future!(self.find_nodes_close_to_self(crypto_kind, node_ref.default_filtered_with_sequencing(Sequencing::PreferOrdered), capabilities.clone())).await {
            Err(e) => {
                veilid_log!(self error
                    "find_self failed for {:?}: {:?}",
                    &node_ref, e
                );
                return;
            }
            Ok(v) => v,
        } => [ format!(": crypto_kind={} node_ref={} wide={}", crypto_kind, node_ref, wide) ] {
            return;
        });

        // Ask each node near us to find us as well
        if wide {
            for closest_nr in closest_nodes {
                network_result_value_or_log!(self match pin_future!(self.find_nodes_close_to_self(crypto_kind, closest_nr.default_filtered_with_sequencing(Sequencing::PreferOrdered), capabilities.clone())).await {
                    Err(e) => {
                        veilid_log!(self error
                            "find_self failed for {:?}: {:?}",
                            &closest_nr, e
                        );
                        continue;
                    }
                    Ok(v) => v,
                } => [ format!(": crypto_kind={} closest_nr={} wide={}", crypto_kind, closest_nr, wide) ] {
                    // Do nothing with non-values
                    continue;
                });
            }
        }
    }

    #[instrument(level = "trace", skip(self, filter, metric), ret)]
    #[expect(dead_code)]
    pub fn find_fastest_node(
        &self,
        cur_ts: Timestamp,
        filter: impl Fn(&BucketEntryInner) -> bool,
        metric: impl Fn(&LatencyStats) -> TimestampDuration,
    ) -> Option<NodeRef> {
        let inner = self.inner.read();
        inner.find_fastest_node(cur_ts, filter, metric)
    }

    #[instrument(level = "trace", skip(self, filter, metric), ret)]
    pub fn find_random_fast_node(
        &self,
        cur_ts: Timestamp,
        filter: impl Fn(&BucketEntryInner) -> bool,
        percentile: f32,
        metric: impl Fn(&LatencyStats) -> TimestampDuration,
    ) -> Option<NodeRef> {
        let inner = self.inner.read();
        inner.find_random_fast_node(cur_ts, filter, percentile, metric)
    }

    #[instrument(level = "trace", skip(self, filter, metric), ret)]
    #[expect(dead_code)]
    pub fn get_node_speed_percentile(
        &self,
        node_id: NodeId,
        cur_ts: Timestamp,
        filter: impl Fn(&BucketEntryInner) -> bool,
        metric: impl Fn(&LatencyStats) -> TimestampDuration,
    ) -> Option<NodeRelativePerformance> {
        let inner = self.inner.read();
        inner.get_node_relative_performance(node_id, cur_ts, filter, metric)
    }

    /// Find the best routing domain for a node info
    /// Returns Some(rd) if there is a 'best' routing domain for this node info given its stated origin
    /// Returns None if no node info from this origin is acceptable
    #[instrument(level = "trace", target = "rtab", skip_all)]
    pub fn find_best_node_info_routing_domain(
        &self,
        origin_routing_domain: RoutingDomain,
        node_info: &NodeInfo,
    ) -> Option<RoutingDomain> {
        // See what routing domains it could be placed in
        let valid_routing_domains = self.get_node_info_routing_domains(node_info);

        // For each valid routing domain in preference order,
        // see if the valid domains can accept peer info from this origin
        for rd in valid_routing_domains {
            let origin_routing_domains = self.origin_routing_domains(rd);
            if origin_routing_domains.contains(origin_routing_domain) {
                return Some(rd);
            }
        }

        None
    }
}
