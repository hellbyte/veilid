use super::*;

#[derive(Clone, Debug)]
pub struct RouteSelectParams {
    pub crypto_kind: CryptoKind,
    pub safety_spec: SafetySpec,
    pub directions: DirectionSet,
    pub avoid_nodes: Vec<NodeId>,
    pub is_destination_safe: bool,
}

#[derive(Clone, Debug)]
struct FirstAvailableRouteParams {
    pub crypto_kind: CryptoKind,
    pub min_hop_count: usize,
    pub max_hop_count: usize,
    pub stability: Stability,
    pub sequencing: Sequencing,
    pub directions: DirectionSet,
    pub avoid_nodes: Vec<NodeId>,
}

#[derive(Clone, Debug)]
pub struct RouteIdAndPublicKeys {
    pub route_id: RouteId,
    pub public_keys: PublicKeyGroup,
}

impl RouteSpecStore {
    /// Get a single allocated route that matches a particular safety spec
    /// Returns the public key associated with a single allocated route
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip_all, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub async fn select_single_route(
        &self,
        mut params: RouteSelectParams,
    ) -> VeilidAPIResult<RouteIdAndPublicKeys> {
        // Ensure the total hop count isn't too long for our config
        if params.safety_spec.hop_count == 0 {
            apibail_invalid_argument!(
                "safety route hop count is zero",
                "safety_spec.hop_count",
                params.safety_spec.hop_count
            );
        }

        if params.safety_spec.hop_count > self.get_max_route_hop_count() {
            apibail_invalid_argument!(
                "safety route hop count too long",
                "safety_spec.hop_count",
                params.safety_spec.hop_count
            );
        }

        // Increase hop count if too short when targeting unsafe destinations
        if !params.is_destination_safe
            && params.safety_spec.hop_count < self.get_default_route_hop_count_unsafe()
        {
            params.safety_spec.hop_count =
                (params.safety_spec.hop_count * 2).min(self.get_max_route_hop_count());
        };

        let first_available_route_params = FirstAvailableRouteParams {
            crypto_kind: params.crypto_kind,
            min_hop_count: params.safety_spec.hop_count,
            max_hop_count: params.safety_spec.hop_count,
            stability: params.safety_spec.stability,
            sequencing: params.safety_spec.sequencing,
            directions: params.directions,
            avoid_nodes: params.avoid_nodes,
        };

        let opt_first_available_route_lock_guard = {
            let inner = self.inner.read();

            // See if the preferred route is already available
            if let Some(preferred_route) = &params.safety_spec.preferred_route {
                if let Some(preferred_rssd) = inner.content.get_detail(preferred_route) {
                    // Only use the preferred route if it has the desired crypto kind
                    let public_keys = preferred_rssd.get_route_set_keys();
                    if public_keys.contains_kind(params.crypto_kind) {
                        // Only use the preferred route if it doesn't contain the avoid nodes
                        if !preferred_rssd.contains_nodes(&first_available_route_params.avoid_nodes)
                        {
                            return Ok(RouteIdAndPublicKeys {
                                route_id: preferred_route.clone(),
                                public_keys,
                            });
                        }
                    }
                }
            }

            // Select a safety route from the pool or make one if we don't have one that matches
            if let Some(sr_route_id_and_public_keys) =
                Self::first_available_route_inner(&inner, &first_available_route_params)
            {
                // Found a route to use
                return Ok(sr_route_id_and_public_keys);
            }

            // No matching route found so allocate one

            // Trade locks to get the first available allocate lock
            self.first_available_route_lock.try_lock()

            // Drop inner read lock because it is synchronous
        };

        let _first_available_route_lock_guard = match opt_first_available_route_lock_guard {
            Some(g) => {
                // No need to re-check first available route because try_lock means no contention
                g
            }
            None => {
                // Get the first available allocate lock
                let g = self.first_available_route_lock.lock().await;

                // Must re-check first available route to avoid race condition due to await
                let inner = self.inner.read();
                if let Some(sr_route_id_and_public_keys) =
                    Self::first_available_route_inner(&inner, &first_available_route_params)
                {
                    // Found a route to use
                    return Ok(sr_route_id_and_public_keys);
                }

                g
            }
        };

        let params = AllocateRouteParams {
            crypto_kinds: vec![params.crypto_kind],
            safety_spec: params.safety_spec,
            directions: params.directions,
            avoid_nodes: first_available_route_params.avoid_nodes,
            automatic: true,
        };
        self.allocate_route(&params).await
    }

    /// Find first matching unpublished route that fits into the selection criteria
    /// Don't pick any routes that have failed and haven't been tested yet
    #[allow(clippy::too_many_arguments)]
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip_all, fields(__VEILID_LOG_KEY = inner.cache.log_key()))
    )]
    fn first_available_route_inner(
        inner: &RouteSpecStoreInner,
        params: &FirstAvailableRouteParams,
    ) -> Option<RouteIdAndPublicKeys> {
        let cur_ts = Timestamp::now();

        let mut routes = Vec::new();

        // Get all valid routes, allow routes that need testing
        // but definitely prefer routes that have been recently tested
        for (id, rssd) in inner.content.iter_details() {
            if rssd.is_sequencing_match(params.sequencing)
                && rssd.hop_count() >= params.min_hop_count
                && rssd.hop_count() <= params.max_hop_count
                && rssd.get_directions().is_superset(params.directions)
                && rssd
                    .get_route_set_keys()
                    .iter()
                    .any(|x| x.kind() == params.crypto_kind)
                && !rssd.is_published()
                && !rssd.contains_nodes(&params.avoid_nodes)
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
            let a_meets_stability = a.1.get_stability() >= params.stability;
            let b_meets_stability = b.1.get_stability() >= params.stability;
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
        routes.first().map(|r| RouteIdAndPublicKeys {
            route_id: r.0.clone(),
            public_keys: r.1.get_route_set_keys().clone(),
        })
    }
}
