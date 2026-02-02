use super::*;

impl RouteSpecStore {
    /// Choose the best private route from a private route set to communicate with
    pub fn best_remote_private_route(&self, id: &RouteId) -> Option<PrivateRoute> {
        let mut inner = self.inner.write();
        let cur_ts = Timestamp::now();
        let rpri = inner.cache.get_remote_private_route(cur_ts, id)?;
        rpri.best_private_route()
    }

    /// Check if a route id is remote or not
    pub fn is_route_id_remote(&self, id: &RouteId) -> bool {
        let mut inner = self.inner.write();
        let cur_ts = Timestamp::now();
        inner
            .cache
            .peek_remote_private_route_mut(cur_ts, id)
            .is_some()
    }

    /// Import a remote private route set blob for compilation
    /// It is safe to import the same route more than once and it will return the same route id
    /// Returns a route set id
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip_all, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub fn import_remote_route_blob(&self, blob: Vec<u8>) -> VeilidAPIResult<RouteId> {
        let cur_ts = Timestamp::now();

        // decode the pr blob
        let private_routes = self.blob_to_private_routes(blob)?;

        // make the route id
        let id = self.generate_remote_route_id(&private_routes)?;

        // validate the private routes
        let mut inner = self.inner.write();
        for private_route in &private_routes {
            // ensure private route has first hop
            if !matches!(private_route.hops, PrivateRouteHops::FirstHop(_)) {
                apibail_generic!("private route must have first hop");
            }

            // ensure this isn't also an allocated route
            // if inner.content.get_id_by_key(&private_route.public_key.value).is_some() {
            //     bail!("should not import allocated route");
            // }
        }

        inner
            .cache
            .cache_remote_private_route(cur_ts, id.clone(), private_routes);

        Ok(id)
    }

    /// Add a single remote private route for compilation
    /// It is safe to add the same route more than once and it will return the same route id
    /// Returns a route set id
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip_all, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub fn import_single_remote_route(
        &self,
        private_route: PrivateRoute,
    ) -> VeilidAPIResult<RouteId> {
        let cur_ts = Timestamp::now();

        // Make a single route set
        let private_routes = vec![private_route];

        // make the route id
        let id = self.generate_remote_route_id(&private_routes)?;

        // validate the private routes
        let mut inner = self.inner.write();
        for private_route in &private_routes {
            // ensure private route has first hop
            if !matches!(private_route.hops, PrivateRouteHops::FirstHop(_)) {
                apibail_generic!("private route must have first hop");
            }

            // ensure this isn't also an allocated route
            // if inner.content.get_id_by_key(&private_route.public_key.value).is_some() {
            //     bail!("should not import allocated route");
            // }
        }

        inner
            .cache
            .cache_remote_private_route(cur_ts, id.clone(), private_routes);

        Ok(id)
    }

    /// Release a remote private route that is no longer in use
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip_all, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub(super) fn release_remote_route_id(&self, id: RouteId) -> bool {
        let mut inner = self.inner.write();
        inner.cache.remove_remote_private_route(id)
    }

    /// Get a route id for a route's public key
    pub fn get_route_id_for_key(&self, key: &PublicKey) -> Option<RouteId> {
        let inner = self.inner.read();
        // Check for allocated route
        if let Some(id) = inner.content.get_id_by_key(key) {
            return Some(id);
        }

        // Check for remote route
        if let Some(rrid) = inner.cache.get_remote_private_route_id_by_key(key) {
            return Some(rrid);
        }

        None
    }

    /// Check to see if this remote (not ours) private route has seen our current node info yet
    /// This happens when you communicate with a private route without a safety route
    pub fn has_remote_private_route_seen_our_node_info(
        &self,
        key: &PublicKey,
        published_peer_info: &PeerInfo,
    ) -> bool {
        let inner = self.inner.read();

        // Check for allocated route. If this is not a remote private route,
        // we may be running a test and using our own allocated route as the destination private route.
        // In that case we definitely have already seen our own node info
        if inner.content.get_id_by_key(key).is_some() {
            return true;
        }

        if let Some(rrid) = inner.cache.get_remote_private_route_id_by_key(key) {
            let cur_ts = Timestamp::now();
            if let Some(rpri) = inner.cache.peek_remote_private_route(cur_ts, &rrid) {
                return rpri.has_seen_our_node_info_ts(published_peer_info.node_info().timestamp());
            }
        }

        false
    }

    /// Mark a remote private route as having seen our current published node info
    /// PRIVACY:
    /// We do not accept node info timestamps from remote private routes because this would
    /// enable a deanonymization attack, whereby a node could be 'pinged' with a doctored node_info with a
    /// special 'timestamp', which then may be sent back over a private route, identifying that it
    /// was that node that had the private route.
    pub fn mark_remote_private_route_seen_our_node_info(
        &self,
        key: &PublicKey,
        cur_ts: Timestamp,
    ) -> VeilidAPIResult<()> {
        let Some(our_node_info_ts) = self
            .routing_table()
            .get_published_peer_info(RoutingDomain::PublicInternet)
            .map(|pi| pi.node_info().timestamp())
        else {
            apibail_internal!("peer info is not yet published");
        };

        let mut inner = self.inner.write();

        // Check for allocated route. If this is not a remote private route
        // then we just skip the recording. We may be running a test and using
        // our own allocated route as the destination private route.
        if inner.content.get_id_by_key(key).is_some() {
            return Ok(());
        }

        if let Some(rrid) = inner.cache.get_remote_private_route_id_by_key(key) {
            if let Some(rpri) = inner.cache.peek_remote_private_route_mut(cur_ts, &rrid) {
                rpri.set_last_seen_our_node_info_ts(our_node_info_ts);
                return Ok(());
            }
        }

        apibail_invalid_target!("private route is missing from store");
    }

    /// Convert binary blob to private route vector
    fn blob_to_private_routes(&self, blob: Vec<u8>) -> VeilidAPIResult<Vec<PrivateRoute>> {
        // Deserialize count
        if blob.is_empty() {
            apibail_invalid_argument!(
                "not deserializing empty private route blob",
                "blob.is_empty",
                true
            );
        }

        let pr_count = blob[0] as usize;
        if pr_count > MAX_CRYPTO_KINDS {
            apibail_invalid_argument!("too many crypto kinds to decode blob", "blob[0]", pr_count);
        }

        // Deserialize stream of private routes
        let decode_context = RPCDecodeContext {
            registry: self.registry(),
            origin_routing_domain: RoutingDomain::PublicInternet,
        };
        let pr_slice = &blob[1..];
        let mut out = Vec::with_capacity(pr_count);
        for _ in 0..pr_count {
            let reader = capnp::serialize_packed::read_message(
                pr_slice,
                capnp::message::ReaderOptions::new(),
            )
            .map_err(|e| VeilidAPIError::invalid_argument("failed to read blob", "e", e))?;

            let pr_reader = reader
                .get_root::<veilid_capnp::private_route::Reader>()
                .map_err(VeilidAPIError::internal)?;
            let private_route = decode_private_route(&decode_context, &pr_reader).map_err(|e| {
                VeilidAPIError::invalid_argument("failed to decode private route", "e", e)
            })?;

            out.push(private_route);
        }

        // Don't trust the order of the blob
        out.sort_by(|a, b| a.public_key.cmp(&b.public_key));

        Ok(out)
    }

    /// Generate RouteId from set of private routes
    fn generate_remote_route_id(
        &self,
        private_routes: &[PrivateRoute],
    ) -> VeilidAPIResult<RouteId> {
        let crypto = self.crypto();

        let pkbyteslen = private_routes
            .iter()
            .fold(0, |acc, x| acc + x.public_key.ref_value().len());
        let mut pkbytes = Vec::with_capacity(pkbyteslen);
        let mut best_kind: Option<CryptoKind> = None;
        for private_route in private_routes {
            if best_kind.is_none()
                || compare_crypto_kind(
                    &private_route.public_key.kind(),
                    best_kind.as_ref().unwrap_or_log(),
                ) == cmp::Ordering::Less
            {
                best_kind = Some(private_route.public_key.kind());
            }
            pkbytes.extend_from_slice(private_route.public_key.ref_value());
        }
        let Some(best_kind) = best_kind else {
            apibail_internal!("no compatible crypto kinds in route");
        };
        let vcrypto = crypto.get(best_kind).unwrap_or_log();

        Ok(RouteId::new(
            vcrypto.kind(),
            BareRouteId::new(vcrypto.generate_hash(&pkbytes).ref_value()),
        ))
    }
}
