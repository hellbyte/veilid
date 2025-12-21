use super::*;

impl RouteSpecStore {
    /// Get a single allocated route that matches a particular safety spec
    /// Returns the public key associated with a single allocated route
    #[instrument(level = "trace", target = "route", skip_all)]
    pub fn select_single_route(
        &self,
        crypto_kind: CryptoKind,
        safety_spec: &SafetySpec,
        avoid_nodes: &[NodeId],
        is_destination_safe: bool,
    ) -> VeilidAPIResult<PublicKey> {
        let inner = &mut *self.inner.lock();
        let routing_table = self.routing_table();
        let rti = &mut *routing_table.inner.write();

        self.select_single_route_inner(
            inner,
            rti,
            crypto_kind,
            safety_spec,
            //Direction::Inbound.into(),
            DirectionSet::all(),
            avoid_nodes,
            is_destination_safe,
        )
    }

    #[instrument(level = "trace", target = "route", skip_all)]
    #[expect(clippy::too_many_arguments)]
    pub(super) fn select_single_route_inner(
        &self,
        inner: &mut RouteSpecStoreInner,
        rti: &mut RoutingTableInner,
        crypto_kind: CryptoKind,
        safety_spec: &SafetySpec,
        direction: DirectionSet,
        avoid_nodes: &[NodeId],
        is_destination_safe: bool,
    ) -> VeilidAPIResult<PublicKey> {
        // Ensure the total hop count isn't too long for our config
        if safety_spec.hop_count == 0 {
            apibail_invalid_argument!(
                "safety route hop count is zero",
                "safety_spec.hop_count",
                safety_spec.hop_count
            );
        }

        if safety_spec.hop_count > self.get_max_route_hop_count() {
            apibail_invalid_argument!(
                "safety route hop count too long",
                "safety_spec.hop_count",
                safety_spec.hop_count
            );
        }

        // Increase hop count if too short when targeting unsafe destinations
        let safety_spec = if is_destination_safe
            || safety_spec.hop_count > self.get_default_route_hop_count_unsafe()
        {
            safety_spec.clone()
        } else {
            SafetySpec {
                hop_count: (safety_spec.hop_count * 2).min(self.get_max_route_hop_count()),
                ..safety_spec.clone()
            }
        };

        // See if the preferred route is here
        if let Some(preferred_route) = &safety_spec.preferred_route {
            if let Some(preferred_rssd) = inner.content.get_detail(preferred_route) {
                // Only use the preferred route if it has the desired crypto kind
                if let Some(preferred_key) = preferred_rssd.get_route_set_keys().get(crypto_kind) {
                    // Only use the preferred route if it doesn't contain the avoid nodes
                    if !preferred_rssd.contains_nodes(avoid_nodes) {
                        return Ok(preferred_key);
                    }
                }
            }
        }

        // Select a safety route from the pool or make one if we don't have one that matches
        let sr_route_id = if let Some(sr_route_id) = Self::first_available_route_inner(
            inner,
            crypto_kind,
            safety_spec.hop_count,
            safety_spec.hop_count,
            safety_spec.stability,
            safety_spec.sequencing,
            direction,
            avoid_nodes,
        ) {
            // Found a route to use
            sr_route_id
        } else {
            // No route found, gotta allocate one
            self.allocate_route_inner(
                inner,
                rti,
                &[crypto_kind],
                &safety_spec,
                direction,
                avoid_nodes,
                true,
            )?
        };

        let sr_pubkey = inner
            .content
            .get_detail(&sr_route_id)
            .unwrap()
            .get_route_set_keys()
            .get(crypto_kind)
            .unwrap();

        Ok(sr_pubkey)
    }

    /// Find first matching unpublished route that fits into the selection criteria
    /// Don't pick any routes that have failed and haven't been tested yet
    #[allow(clippy::too_many_arguments)]
    #[instrument(level = "trace", target = "route", skip_all)]
    fn first_available_route_inner(
        inner: &RouteSpecStoreInner,
        crypto_kind: CryptoKind,
        min_hop_count: usize,
        max_hop_count: usize,
        stability: Stability,
        sequencing: Sequencing,
        directions: DirectionSet,
        avoid_nodes: &[NodeId],
    ) -> Option<RouteId> {
        let cur_ts = Timestamp::now();

        let mut routes = Vec::new();

        // Get all valid routes, allow routes that need testing
        // but definitely prefer routes that have been recently tested
        for (id, rssd) in inner.content.iter_details() {
            if rssd.is_sequencing_match(sequencing)
                && rssd.hop_count() >= min_hop_count
                && rssd.hop_count() <= max_hop_count
                && rssd.get_directions().is_superset(directions)
                && rssd
                    .get_route_set_keys()
                    .iter()
                    .any(|x| x.kind() == crypto_kind)
                && !rssd.is_published()
                && !rssd.contains_nodes(avoid_nodes)
            {
                routes.push((id, rssd));
            }
        }

        // Sort the routes by preference
        routes.sort_by(|a, b| {
            // Prefer routes that don't need testing
            let a_needs_testing = a.1.get_stats().needs_testing(cur_ts);
            let b_needs_testing = b.1.get_stats().needs_testing(cur_ts);
            if !a_needs_testing && b_needs_testing {
                return cmp::Ordering::Less;
            }
            if !b_needs_testing && a_needs_testing {
                return cmp::Ordering::Greater;
            }

            // Prefer routes that meet the stability selection
            let a_meets_stability = a.1.get_stability() >= stability;
            let b_meets_stability = b.1.get_stability() >= stability;
            if a_meets_stability && !b_meets_stability {
                return cmp::Ordering::Less;
            }
            if b_meets_stability && !a_meets_stability {
                return cmp::Ordering::Greater;
            }

            // Prefer faster routes
            let a_latency = a.1.get_stats().latency_stats().average;
            let b_latency = b.1.get_stats().latency_stats().average;

            a_latency.cmp(&b_latency)
        });

        // Return the best one if we got one
        routes.first().map(|r| r.0.clone())
    }
}
