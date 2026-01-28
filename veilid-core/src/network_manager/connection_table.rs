use super::*;
use futures_util::StreamExt;
use hashlink::LruCache;

impl_veilid_log_facility!("net");

/// Allow 25% of the table size to be occupied by priority flows
/// that will not be subject to LRU termination.
const PRIORITY_FLOW_PERCENTAGE: usize = 25;

///////////////////////////////////////////////////////////////////////////////
#[derive(ThisError, Debug)]
pub enum ConnectionTableAddError {
    #[error("Connection already added to table")]
    AlreadyExists(Box<NetworkConnection>),
    #[error("Connection address was filtered")]
    AddressFilter(Box<NetworkConnection>, AddConnectionError),
    #[error("Connection table is full")]
    TableFull(Box<NetworkConnection>),
}

impl ConnectionTableAddError {
    pub fn already_exists(conn: NetworkConnection) -> Self {
        ConnectionTableAddError::AlreadyExists(Box::new(conn))
    }
    pub fn address_filter(conn: NetworkConnection, err: AddConnectionError) -> Self {
        ConnectionTableAddError::AddressFilter(Box::new(conn), err)
    }
    pub fn table_full(conn: NetworkConnection) -> Self {
        ConnectionTableAddError::TableFull(Box::new(conn))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ConnectionRefKind {
    AddRef,
    RemoveRef,
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ConnectionTableInner {
    max_connections: BTreeMap<ProtocolType, usize>,
    conn_by_id: BTreeMap<ProtocolType, LruCache<NetworkConnectionId, NetworkConnection>>,
    protocol_type_by_id: BTreeMap<NetworkConnectionId, ProtocolType>,
    id_by_flow: BTreeMap<Flow, NetworkConnectionId>,
    ids_by_remote: BTreeMap<PeerAddress, Vec<NetworkConnectionId>>,
    priority_flows: BTreeMap<ProtocolType, LruCache<Flow, ()>>,
}

#[derive(Debug)]
pub struct ConnectionTable {
    registry: VeilidComponentRegistry,
    inner: Mutex<ConnectionTableInner>,
}

impl_veilid_component_accessors!(ConnectionTable);

impl ConnectionTable {
    pub fn new(registry: VeilidComponentRegistry) -> Self {
        let config = registry.config();
        let max_connections = {
            let mut max_connections = BTreeMap::<ProtocolType, usize>::new();

            max_connections.insert(
                ProtocolType::TCP,
                config.network.protocol.tcp.max_connections as usize,
            );
            max_connections.insert(
                ProtocolType::WS,
                config.network.protocol.ws.max_connections as usize,
            );
            #[cfg(feature = "enable-protocol-wss")]
            max_connections.insert(
                ProtocolType::WSS,
                config.network.protocol.wss.max_connections as usize,
            );

            max_connections
        };
        Self {
            registry,
            inner: Mutex::new(ConnectionTableInner {
                conn_by_id: max_connections
                    .keys()
                    .map(|pt| (*pt, LruCache::new_unbounded()))
                    .collect(),
                protocol_type_by_id: BTreeMap::new(),
                id_by_flow: BTreeMap::new(),
                ids_by_remote: BTreeMap::new(),
                priority_flows: max_connections
                    .iter()
                    .map(|(pt, x)| (*pt, LruCache::new(*x * PRIORITY_FLOW_PERCENTAGE / 100)))
                    .collect(),
                max_connections,
            }),
        }
    }

    #[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), fields(__VEILID_LOG_KEY = self.log_key())))]
    pub async fn join(&self) {
        let mut unord = {
            let mut inner = self.inner.lock();
            let unord = FuturesUnordered::new();
            for (pt, table) in &mut inner.conn_by_id {
                for (_, mut v) in table.drain() {
                    veilid_log!(self trace "connection table {} join: {:?}", pt, v);
                    v.close();
                    unord.push(v);
                }
            }
            inner.protocol_type_by_id.clear();
            inner.id_by_flow.clear();
            inner.ids_by_remote.clear();
            unord
        };

        while unord.next().await.is_some() {}
    }

    // Return true if there is another connection in the table using a different protocol type
    // to the same address and port with the same low level protocol type.
    // Specifically right now this checks for a TCP connection that exists to the same
    // low level TCP remote as a WS or WSS connection, since they are all low-level TCP
    #[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), ret, fields(__VEILID_LOG_KEY = self.log_key())))]
    pub fn check_for_colliding_connection(&self, dial_info: &DialInfo) -> bool {
        let inner = self.inner.lock();

        let protocol_type = dial_info.protocol_type();
        let low_level_protocol_type = protocol_type.low_level_protocol_type();

        // check protocol types
        let mut check_protocol_types = ProtocolTypeSet::empty();
        for check_pt in ProtocolTypeSet::all().iter() {
            if check_pt != protocol_type
                && check_pt.low_level_protocol_type() == low_level_protocol_type
            {
                check_protocol_types.insert(check_pt);
            }
        }
        let socket_address = dial_info.socket_address();

        for check_pt in check_protocol_types {
            let check_pa = PeerAddress::new(socket_address, check_pt);
            if inner.ids_by_remote.contains_key(&check_pa) {
                return true;
            }
        }
        false
    }

    /// Add a priority flow, which is protected from eviction but without the
    /// punishment expectations of a fully 'protected' connection.
    /// This is an LRU set, so there is no removing the flows by hand, and
    /// they are kept in a 'best effort' fashion.
    /// If connections 'should' stay alive, use this mechanism.
    /// If connections 'must' stay alive, use 'NetworkConnection::protect'.
    pub fn add_priority_flow(&self, flow: Flow) {
        let mut inner = self.inner.lock();
        let protocol_type = flow.protocol_type();
        if let Some(pf) = inner.priority_flows.get_mut(&protocol_type) {
            pf.insert(flow, ());
        } else {
            veilid_log!(self error "Missing priority flow table for protocol type: {}", protocol_type);
        }
    }

    /// The mechanism for selecting which connections get evicted from the connection table
    /// when it is getting full while adding a new connection.
    /// Factored out into its own function for clarity.
    fn lru_out_connection_inner(
        &self,
        inner: &mut ConnectionTableInner,
        protocol_type: ProtocolType,
    ) -> Result<Option<NetworkConnection>, ()> {
        // If nothing needs to be LRUd out right now, then just return
        if inner.conn_by_id[&protocol_type].len() < inner.max_connections[&protocol_type] {
            return Ok(None);
        }

        // Find a free connection to terminate to make room
        let dead_k = {
            let Some(lruk) = inner.conn_by_id[&protocol_type].iter().find_map(|(k, v)| {
                // Ensure anything being LRU evicted isn't protected somehow
                // 1. connections that are 'in-use' are kept
                // 2. connections with flows in the priority list are kept
                // 3. connections that are protected are kept
                if !v.is_in_use()
                    && !inner.priority_flows[&protocol_type].contains_key(&v.flow())
                    && v.protected_node_ref().is_none()
                {
                    Some(*k)
                } else {
                    None
                }
            }) else {
                // Can't make room, connection table is full
                return Err(());
            };
            lruk
        };

        let dead_conn = self.remove_connection_records_inner(inner, dead_k);
        Ok(Some(dead_conn))
    }

    #[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), ret, fields(__VEILID_LOG_KEY = self.log_key())))]
    pub fn add_connection(
        &self,
        network_connection: NetworkConnection,
    ) -> Result<Option<NetworkConnection>, ConnectionTableAddError> {
        // Get indices for network connection table
        let id = network_connection.connection_id();
        let flow = network_connection.flow();
        let protocol_type = flow.protocol_type();
        let remote = flow.remote();

        let mut inner = self.inner.lock();

        // Two connections to the same flow should be rejected (soft rejection)
        if inner.id_by_flow.contains_key(&flow) {
            return Err(ConnectionTableAddError::already_exists(network_connection));
        }

        // Validate this implementation (hard fails that would invalidate the representation)
        if inner.conn_by_id[&protocol_type].contains_key(&id) {
            panic!("duplicate connection id: {:#?}", network_connection);
        }
        if inner.protocol_type_by_id.contains_key(&id) {
            panic!("duplicate id to protocol index: {:#?}", network_connection);
        }
        if let Some(ids) = inner.ids_by_remote.get(&flow.remote()) {
            if ids.contains(&id) {
                panic!("duplicate id by remote: {:#?}", network_connection);
            }
        }

        // Filter by ip for connection limits
        let ip_addr = flow.remote_address().ip_addr();
        if let Err(e) = self
            .network_manager()
            .address_filter()
            .add_connection(ip_addr)
        {
            // Return the connection in the error to be disposed of
            return Err(ConnectionTableAddError::address_filter(
                network_connection,
                e,
            ));
        }

        // if we have reached the maximum number of connections per protocol type
        // then drop the least recently used connection that is not protected or referenced
        let out_conn = match self.lru_out_connection_inner(&mut inner, protocol_type) {
            Ok(v) => v,
            Err(()) => {
                return Err(ConnectionTableAddError::table_full(network_connection));
            }
        };

        // Add the connection to the table
        let conn_by_id_table = inner.conn_by_id.get_mut(&protocol_type).unwrap_or_log();
        let res = conn_by_id_table.insert(id, network_connection);
        assert!(res.is_none());

        // add connection records
        inner.protocol_type_by_id.insert(id, protocol_type);
        inner.id_by_flow.insert(flow, id);
        inner.ids_by_remote.entry(remote).or_default().push(id);

        Ok(out_conn)
    }

    //#[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), ret, fields(__VEILID_LOG_KEY = self.log_key())))]
    pub fn peek_connection_by_flow(&self, flow: Flow) -> Option<ConnectionHandle> {
        if flow.protocol_type() == ProtocolType::UDP {
            return None;
        }

        let inner = self.inner.lock();

        let id = *inner.id_by_flow.get(&flow)?;
        let protocol_type = flow.protocol_type();
        let out = inner.conn_by_id[&protocol_type].peek(&id).unwrap_or_log();
        Some(out.get_handle())
    }

    //#[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), ret, fields(__VEILID_LOG_KEY = self.log_key())))]
    pub fn touch_connection_by_id(&self, id: NetworkConnectionId) {
        let mut inner = self.inner.lock();
        let Some(protocol_type) = inner.protocol_type_by_id.get(&id).copied() else {
            return;
        };
        let conn_by_id_table = inner.conn_by_id.get_mut(&protocol_type).unwrap_or_log();
        let _ = conn_by_id_table.get(&id);
    }

    #[expect(dead_code)]
    pub fn with_connection_by_flow<R, F: FnOnce(&NetworkConnection) -> R>(
        &self,
        flow: Flow,
        closure: F,
    ) -> Option<R> {
        if flow.protocol_type() == ProtocolType::UDP {
            return None;
        }

        let inner = self.inner.lock();

        let id = *inner.id_by_flow.get(&flow)?;
        let protocol_type = flow.protocol_type();
        let out = inner.conn_by_id[&protocol_type].peek(&id).unwrap_or_log();
        Some(closure(out))
    }

    #[expect(dead_code)]
    pub fn with_connection_by_flow_mut<R, F: FnOnce(&mut NetworkConnection) -> R>(
        &self,
        flow: Flow,
        closure: F,
    ) -> Option<R> {
        if flow.protocol_type() == ProtocolType::UDP {
            return None;
        }

        let mut inner = self.inner.lock();

        let id = *inner.id_by_flow.get(&flow)?;
        let protocol_type = flow.protocol_type();
        let conn_by_id_table = inner.conn_by_id.get_mut(&protocol_type).unwrap_or_log();
        let out = conn_by_id_table.peek_mut(&id).unwrap_or_log();
        Some(closure(out))
    }

    pub fn with_all_connections_mut<R, F: FnMut(&mut NetworkConnection) -> Option<R>>(
        &self,
        mut closure: F,
    ) -> Option<R> {
        let mut inner_lock = self.inner.lock();
        let inner = &mut *inner_lock;
        for (id, pt) in inner.protocol_type_by_id.iter() {
            let conn_by_id_table = inner.conn_by_id.get_mut(pt).unwrap_or_log();
            if let Some(conn) = conn_by_id_table.peek_mut(id) {
                if let Some(out) = closure(conn) {
                    return Some(out);
                }
            }
        }
        None
    }

    //#[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), ret, fields(__VEILID_LOG_KEY = self.log_key())))]
    pub fn ref_connection_by_id(
        &self,
        id: NetworkConnectionId,
        ref_type: ConnectionRefKind,
    ) -> bool {
        let mut inner = self.inner.lock();
        let Some(protocol_type) = inner.protocol_type_by_id.get(&id).copied() else {
            // Sometimes network connections die before we can ref/unref them
            return false;
        };
        let conn_by_id_table = inner.conn_by_id.get_mut(&protocol_type).unwrap_or_log();
        let out = conn_by_id_table.get_mut(&id).unwrap_or_log();
        match ref_type {
            ConnectionRefKind::AddRef => out.add_ref(),
            ConnectionRefKind::RemoveRef => out.remove_ref(),
        }
        true
    }

    // #[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), ret, fields(__VEILID_LOG_KEY = self.log_key())))]
    pub fn get_best_connection_by_remote(
        &self,
        best_port: Option<u16>,
        remote: PeerAddress,
    ) -> Option<ConnectionHandle> {
        let inner = &mut *self.inner.lock();

        let all_ids_by_remote = inner.ids_by_remote.get(&remote)?;
        let protocol_type = remote.protocol_type();
        let conn_by_id_table = inner.conn_by_id.get_mut(&protocol_type).unwrap_or_log();

        if all_ids_by_remote.is_empty() {
            // no connections
            return None;
        }
        if all_ids_by_remote.len() == 1 {
            // only one connection
            let id = all_ids_by_remote[0];
            let nc = conn_by_id_table.get(&id).unwrap_or_log();
            return Some(nc.get_handle());
        }
        // multiple connections, find the one that matches the best port, or the most recent
        if let Some(best_port) = best_port {
            for id in all_ids_by_remote {
                let nc = conn_by_id_table.peek(id).unwrap_or_log();
                if let Some(local_addr) = nc.flow().local() {
                    if local_addr.port() == best_port {
                        let nc = conn_by_id_table.get(id).unwrap_or_log();
                        return Some(nc.get_handle());
                    }
                }
            }
        }
        // just return most recent network connection if a best port match can not be found
        let best_id = *all_ids_by_remote.last().unwrap_or_log();
        let nc = conn_by_id_table.get(&best_id).unwrap_or_log();
        Some(nc.get_handle())
    }

    //#[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), ret, fields(__VEILID_LOG_KEY = self.log_key())))]
    #[expect(dead_code)]
    pub fn get_connection_ids_by_remote(&self, remote: PeerAddress) -> Vec<NetworkConnectionId> {
        let inner = self.inner.lock();
        inner
            .ids_by_remote
            .get(&remote)
            .cloned()
            .unwrap_or_default()
    }

    // pub fn drain_filter<F>(&self, mut filter: F) -> Vec<NetworkConnection>
    // where
    //     F: FnMut(Flow) -> bool,
    // {
    //     let mut inner = self.inner.lock();
    //     let mut filtered_ids = Vec::new();
    //     for cbi in &mut inner.conn_by_id {
    //         for (id, conn) in cbi {
    //             if filter(conn.flow()) {
    //                 filtered_ids.push(*id);
    //             }
    //         }
    //     }
    //     let mut filtered_connections = Vec::new();
    //     for id in filtered_ids {
    //         let conn = Self::remove_connection_records(&mut *inner, id);
    //         filtered_connections.push(conn)
    //     }
    //     filtered_connections
    // }

    #[cfg(any(test, feature = "test-util"))]
    pub fn connection_count(&self) -> usize {
        let inner = self.inner.lock();
        inner
            .conn_by_id
            .iter()
            .fold(0, |acc, (_pt, c)| acc + c.len())
    }

    #[cfg_attr(feature = "instrument", instrument(level = "trace", skip(inner), ret, fields(__VEILID_LOG_KEY = self.log_key())))]
    fn remove_connection_records_inner(
        &self,
        inner: &mut ConnectionTableInner,
        id: NetworkConnectionId,
    ) -> NetworkConnection {
        // protocol_type_by_id
        let protocol_type = inner.protocol_type_by_id.remove(&id).unwrap_or_log();
        // conn_by_id
        let conn_by_id_table = inner.conn_by_id.get_mut(&protocol_type).unwrap_or_log();
        let conn = conn_by_id_table.remove(&id).unwrap_or_log();
        // id_by_flow
        let flow = conn.flow();
        let _ = inner
            .id_by_flow
            .remove(&flow)
            .expect_or_log("must have removed something here");
        // ids_by_remote
        let remote = flow.remote();
        let ids = inner.ids_by_remote.get_mut(&remote).unwrap_or_log();
        for (n, elem) in ids.iter().enumerate() {
            if *elem == id {
                let _ = ids.remove(n);
                if ids.is_empty() {
                    inner.ids_by_remote.remove(&remote).unwrap_or_log();
                }
                break;
            }
        }
        // address_filter
        let ip_addr = remote.socket_addr().ip();
        self.network_manager()
            .address_filter()
            .remove_connection(ip_addr)
            .expect_or_log("Inconsistency in connection table");
        conn
    }

    #[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), ret, fields(__VEILID_LOG_KEY = self.log_key())))]
    pub fn remove_connection_by_id(&self, id: NetworkConnectionId) -> Option<NetworkConnection> {
        let mut inner = self.inner.lock();

        let protocol_type = *inner.protocol_type_by_id.get(&id)?;
        if !inner.conn_by_id[&protocol_type].contains_key(&id) {
            return None;
        }
        let conn = self.remove_connection_records_inner(&mut inner, id);
        Some(conn)
    }

    pub fn debug_print_table(&self) -> String {
        let mut out = String::new();
        let inner = self.inner.lock();
        let cur_ts = Timestamp::now();
        for pt in inner.conn_by_id.keys() {
            out += &format!(
                "  {} Connections: ({}/{})\n",
                pt,
                inner.conn_by_id[pt].len(),
                inner.max_connections[pt]
            );

            for (_, conn) in &inner.conn_by_id[pt] {
                let is_priority_flow = inner.priority_flows[pt].contains_key(&conn.flow());

                out += &format!(
                    "    {}{}\n",
                    conn.debug_print(cur_ts),
                    if is_priority_flow { " PRIORITY" } else { "" }
                );
            }
        }

        for pt in inner.priority_flows.keys() {
            out += &format!(
                "  {} Priority Flows: ({}/{})\n",
                pt,
                inner.priority_flows[pt].len(),
                inner.priority_flows[pt].capacity(),
            );

            for (flow, _) in &inner.priority_flows[pt] {
                out += &format!("    {}\n", flow);
            }
        }
        out
    }
}
