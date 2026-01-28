use super::*;

use futures_util::stream::{FuturesUnordered, StreamExt};
use futures_util::FutureExt;
use stop_token::future::FutureExt as _;

impl_veilid_log_facility!("rtab");

/// Number of automatic safety routes of each safe/unsafe default length to keep around in the background
const BACKGROUND_SAFETY_ROUTE_COUNT: usize = 2;

impl RoutingTable {
    fn get_background_safety_route_count(&self) -> usize {
        BACKGROUND_SAFETY_ROUTE_COUNT
    }

    /// Fastest routes sort
    fn route_sort_latency_fn(a: &(RouteId, u64), b: &(RouteId, u64)) -> cmp::Ordering {
        let mut al = a.1;
        let mut bl = b.1;
        // Treat zero latency as uncalculated
        if al == 0 {
            al = u64::MAX;
        }
        if bl == 0 {
            bl = u64::MAX;
        }
        // Less is better
        let c = al.cmp(&bl);
        if c != cmp::Ordering::Equal {
            return c;
        }

        // Otherwise, just sort by route id
        a.0.cmp(&b.0)
    }

    /// Get the list of routes to test and drop
    ///
    /// Allocated routes to test:
    /// * if a route 'needs_testing'
    ///   . all published allocated routes
    ///   . routes that have never been tested
    ///   . the fastest 0..N automatic safety routes of safe default length
    ///   . the fastest 0..N automatic safety routes of unsafe default length
    ///
    /// Routes to drop:
    /// * if a route 'needs_testing'
    ///   . the remaining automatic safety routes
    ///   . the rest of the allocated unpublished routes
    ///
    /// If a route doesn't 'need_testing', then we neither test nor drop it
    #[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), ret, fields(__VEILID_LOG_KEY = self.log_key())))]
    fn get_allocated_routes_to_test(&self, cur_ts: Timestamp) -> Vec<RouteId> {
        let default_route_hop_count_safe =
            self.route_spec_store().get_default_route_hop_count_safe();
        let default_route_hop_count_unsafe =
            self.route_spec_store().get_default_route_hop_count_unsafe();

        let mut must_test_routes = Vec::<RouteId>::new();
        let mut automatic_safe_routes = Vec::<(RouteId, u64)>::new();
        let mut automatic_unsafe_routes = Vec::<(RouteId, u64)>::new();
        let mut expired_routes = Vec::<RouteId>::new();
        self.route_spec_store().list_allocated_routes(|k, v| {
            let stats = v.get_stats();
            // Ignore nodes that don't need testing
            if !stats.needs_testing(cur_ts) {
                return Option::<()>::None;
            }
            // If this has been published, always test if we need it
            // Also if the route has never been tested, test it at least once
            if v.is_published() || stats.last_known_valid_ts.is_none() {
                must_test_routes.push(k.clone());
            }
            // If this is of default safe route hop length, include it in routes to keep alive
            else if v.is_automatic() && v.hop_count() == default_route_hop_count_safe {
                automatic_safe_routes.push((k.clone(), stats.latency.average.as_u64()));
            }
            // If this is a default unsafe route hop length, include it in routes to keep alive
            else if v.is_automatic() && v.hop_count() == default_route_hop_count_unsafe {
                automatic_unsafe_routes.push((k.clone(), stats.latency.average.as_u64()));
            }
            // Else this is a route that hasnt been used recently enough and we can tear it down
            else {
                expired_routes.push(k.clone());
            }
            Option::<()>::None
        });

        // Sort automatic routes by speed if we know the speed
        automatic_safe_routes.sort_by(Self::route_sort_latency_fn);
        automatic_unsafe_routes.sort_by(Self::route_sort_latency_fn);

        // Save up to N unpublished routes and test them
        let background_safety_route_count = self.get_background_safety_route_count();
        let safe_routes_to_keep =
            usize::min(background_safety_route_count, automatic_safe_routes.len());
        for automatic_safe_route in automatic_safe_routes.iter().take(safe_routes_to_keep) {
            must_test_routes.push(automatic_safe_route.0.clone());
        }
        let unsafe_routes_to_keep =
            usize::min(background_safety_route_count, automatic_unsafe_routes.len());
        for automatic_unsafe_route in automatic_unsafe_routes.iter().take(unsafe_routes_to_keep) {
            must_test_routes.push(automatic_unsafe_route.0.clone());
        }

        // Kill off all but N unpublished routes rather than testing them
        if automatic_safe_routes.len() > safe_routes_to_keep {
            for x in &automatic_safe_routes[safe_routes_to_keep..] {
                expired_routes.push(x.0.clone());
            }
        }
        if automatic_unsafe_routes.len() > unsafe_routes_to_keep {
            for x in &automatic_unsafe_routes[unsafe_routes_to_keep..] {
                expired_routes.push(x.0.clone());
            }
        }

        // Process dead routes
        for r in expired_routes {
            veilid_log!(self debug "Expired route: {}", r);
            self.route_spec_store().release_route(r);
        }

        // return routes to test
        must_test_routes
    }

    /// Test set of routes and remove the ones that don't test clean
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", skip(self, stop_token), err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    async fn test_route_set(
        &self,
        stop_token: StopToken,
        routes_needing_testing: Vec<RouteId>,
    ) -> EyreResult<()> {
        if routes_needing_testing.is_empty() {
            return Ok(());
        }
        veilid_log!(self trace "Testing routes: {:?}", routes_needing_testing);

        #[derive(Default, Debug)]
        struct TestRouteContext {
            dead_routes: Vec<RouteId>,
        }

        let ctx = Arc::new(Mutex::new(TestRouteContext::default()));
        {
            let mut unord = FuturesUnordered::new();
            for r in routes_needing_testing {
                let ctx = ctx.clone();
                unord.push(
                    async move {
                        let success = match Box::pin(self.route_spec_store().test_route(r.clone()))
                            .await
                            .ok_try_again()
                        {
                            // Test had result
                            Ok(Some(v)) => v,
                            // Test could not be performed at this time
                            Ok(None) => true,
                            // Test failure
                            Err(e) => {
                                veilid_log!(self error "Test route failed: {}", e);
                                return;
                            }
                        };
                        if success {
                            // Route is okay, leave it alone
                            return;
                        }
                        // Route test failed
                        ctx.lock().dead_routes.push(r);
                    }
                    .instrument(Span::current())
                    .boxed(),
                );
            }

            // Wait for test_route futures to complete in parallel
            while let Ok(Some(())) = unord.next().timeout_at(stop_token.clone()).await {}
        }

        // Process failed routes
        let ctx = Arc::try_unwrap(ctx).unwrap_or_log().into_inner();
        for r in ctx.dead_routes {
            veilid_log!(self debug "Dead route failed to test: {}", r);
            self.route_spec_store().release_route(r);
        }

        Ok(())
    }

    /// Keep private routes assigned and accessible
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", skip(self, stop_token), err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub async fn private_route_management_task_routine(
        &self,
        stop_token: StopToken,
        _last_ts: Timestamp,
        cur_ts: Timestamp,
    ) -> EyreResult<()> {
        // Test locally allocated routes first
        // This may remove dead routes
        let routes_needing_testing = self.get_allocated_routes_to_test(cur_ts);
        if !routes_needing_testing.is_empty() {
            self.test_route_set(stop_token.clone(), routes_needing_testing)
                .await?;
        }

        // Ensure we have a minimum of N allocated local, unpublished routes with the default number of hops
        // and all our supported crypto kinds. Also allocate some routes of twice the default length for
        // safety routes to direct nodes.
        let default_route_hop_count_safe =
            self.route_spec_store().get_default_route_hop_count_safe();
        let default_route_hop_count_unsafe =
            self.route_spec_store().get_default_route_hop_count_unsafe();

        let mut local_safety_route_count_safe = 0usize;
        let mut local_safety_route_count_unsafe = 0usize;

        self.route_spec_store().list_allocated_routes(|_k, v| {
            if !v.is_published() && v.is_automatic() {
                if v.hop_count() == default_route_hop_count_safe {
                    local_safety_route_count_safe += 1;
                } else if v.hop_count() == default_route_hop_count_unsafe {
                    local_safety_route_count_unsafe += 1;
                }
            }
            Option::<()>::None
        });

        let background_safety_route_count = self.get_background_safety_route_count();

        // Newly allocated routes
        let mut newly_allocated_routes = Vec::new();

        // Allocate more routes if needed
        for (route_count, hop_count) in [
            (local_safety_route_count_safe, default_route_hop_count_safe),
            (
                local_safety_route_count_unsafe,
                default_route_hop_count_unsafe,
            ),
        ] {
            if route_count < background_safety_route_count {
                let routes_to_allocate = background_safety_route_count - route_count;

                for _n in 0..routes_to_allocate {
                    // Parameters here must be the most inclusive safety route spec
                    // These will be used by test_remote_route as well
                    let safety_spec = SafetySpec {
                        preferred_route: None,
                        hop_count,
                        stability: Stability::Reliable,
                        sequencing: Sequencing::PreferOrdered,
                    };
                    match self.route_spec_store().allocate_route(
                        &VALID_CRYPTO_KINDS,
                        &safety_spec,
                        DirectionSet::all(),
                        &[],
                        true,
                    ) {
                        Err(VeilidAPIError::TryAgain { message }) => {
                            veilid_log!(self debug "Route allocation unavailable: {}", message);
                        }
                        Err(e) => return Err(e.into()),
                        Ok(v) => {
                            newly_allocated_routes.push(v);
                        }
                    }
                }
            }
        }

        // Immediately test them
        if !newly_allocated_routes.is_empty() {
            self.test_route_set(stop_token.clone(), newly_allocated_routes)
                .await?;
        }

        // Test remote routes next
        let remote_routes_needing_testing = self.route_spec_store().list_remote_routes(|k, v| {
            let stats = v.get_stats();
            if stats.needs_testing(cur_ts) {
                Some(k.clone())
            } else {
                None
            }
        });
        if !remote_routes_needing_testing.is_empty() {
            self.test_route_set(stop_token.clone(), remote_routes_needing_testing)
                .await?;
        }

        // Send update (also may send updates for released routes done by other parts of the program)
        self.route_spec_store().send_route_update();

        Ok(())
    }
}
