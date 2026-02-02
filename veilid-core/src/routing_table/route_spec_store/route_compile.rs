use super::*;

impl RouteSpecStore {
    /// Compiles a safety route to the private route, with caching
    ///
    /// Parameters:
    ///   * safety_selection - The safety selection to use
    ///   * private_route - The private route we are sending to
    ///   * reply_private_route - The private route we want the response to go to, if None, we will se a new safety route
    ///
    /// Returns:
    ///   * Err(VeilidAPIError::TryAgain) if no allocation could happen at this time (not an error)
    ///   * Other Err() if the parameters are wrong
    ///   * Ok(compiled route) on success
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip_all, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub async fn compile_safety_route(
        &self,
        safety_selection: SafetySelection,
        mut private_route: PrivateRoute,
        reply_private_route: Option<PublicKey>,
    ) -> VeilidAPIResult<CompiledRoute> {
        // let profile_start_ts = get_timestamp();
        let routing_table = self.routing_table();

        // Get useful private route properties
        let crypto_kind = private_route.crypto_kind();
        let crypto = routing_table.crypto();
        let Some(vcrypto) = crypto.get_async(crypto_kind) else {
            apibail_generic!("crypto not supported for route");
        };
        let pr_pubkey = private_route.public_key.clone();

        // See if we are using a safety route, if not, short circuit this operation
        let safety_spec = match safety_selection {
            // Safety route spec to use
            SafetySelection::Safe(safety_spec) => safety_spec,
            // Safety route stub with the node's public key as the safety route key since it's the 0th hop
            SafetySelection::Unsafe(sequencing) => {
                let Some(pr_first_hop_node) = private_route.pop_first_hop() else {
                    apibail_generic!("compiled private route should have first hop");
                };

                let opt_first_hop = match pr_first_hop_node {
                    RouteNode::NodeId(id) => {
                        let rti = &*routing_table.inner.read();
                        rti.lookup_node_ref(id).map_err(VeilidAPIError::internal)?
                    }
                    RouteNode::PeerInfo(pi) => {
                        let rti = &mut *routing_table.inner.write();
                        Some(
                            rti.register_node_with_peer_info(pi, false)
                                .map_err(VeilidAPIError::internal)?
                                .unfiltered(),
                        )
                    }
                };
                let Some(first_hop) = opt_first_hop else {
                    // Can't reach this private route any more
                    apibail_generic!("can't reach private route any more");
                };

                // Set sequencing requirement
                let mut first_hop = first_hop.default_filtered_with_sequencing(sequencing);

                // Enforce the routing domain
                first_hop.merge_filter(
                    NodeRefFilter::new().with_routing_domain(RoutingDomain::PublicInternet),
                );

                // Return the compiled safety route
                //veilid_log!(self info "compile_safety_route profile (stub): {} us", (get_timestamp() - profile_start_ts));
                return Ok(CompiledRoute {
                    safety_route: SafetyRoute::new_stub(
                        routing_table.public_key(crypto_kind),
                        private_route,
                    ),
                    secret: routing_table.secret_key(crypto_kind),
                    first_hop,
                });
            }
        };

        // Get the compile lock to ensure we only do this for a particular private route one at at time
        let compile_lock = self
            .compile_lock_table
            .lock_tag(CompileLockKey {
                pr_pubkey: pr_pubkey.clone(),
            })
            .await;

        // If the safety route requested is also the private route, this is a loopback test, just accept it
        let opt_safety_route_id_and_public_key = {
            let inner = self.inner.read();
            let opt_private_route_id = inner.content.get_id_by_key(&pr_pubkey);

            if safety_spec.preferred_route.is_some()
                && safety_spec.preferred_route == opt_private_route_id
            {
                // Private route is also safety route during loopback test
                Some((opt_private_route_id.unwrap(), pr_pubkey.clone()))
            } else {
                match reply_private_route.as_ref() {
                    Some(pr_public_key) => {
                        // Symmetric routing: Use the reply private route as the safety route
                        // XXX: add a safety selection switch to use asymmetric routes
                        let Some(reply_private_route_id) =
                            inner.content.get_id_by_key(pr_public_key)
                        else {
                            apibail_generic!("safety route id missing");
                        };
                        Some((reply_private_route_id, pr_public_key.clone()))
                    }
                    None => {
                        // Reply route not specified, choose a safety route
                        None
                    }
                }
            }
        };

        let (safety_route_id, sr_pubkey) =
            if let Some(safety_route_id_and_public_key) = opt_safety_route_id_and_public_key {
                safety_route_id_and_public_key
            } else {
                let Some(avoid_node_id) = private_route.first_hop_node_id() else {
                    apibail_generic!("compiled private route should have first hop");
                };
                let params = RouteSelectParams {
                    crypto_kind,
                    safety_spec: safety_spec.clone(),
                    directions: Direction::Out.into(),
                    avoid_nodes: vec![avoid_node_id],
                    is_destination_safe: !private_route.is_stub(),
                };

                let RouteIdAndPublicKeys {
                    route_id,
                    public_keys,
                } = self.select_single_route(params).await?;
                (route_id, public_keys.get(crypto_kind).unwrap())
            };

        let (optimize, first_hop, secret, hop_node_refs) = {
            let mut inner = self.inner.write();

            let Some(safety_rssd) = inner.content.get_detail(&safety_route_id) else {
                apibail_internal!("safety route set detail missing");
            };
            let Some(safety_rsd) = safety_rssd.get_route_by_key(&sr_pubkey) else {
                apibail_internal!("safety route detail missing");
            };

            // We can optimize the peer info in this safety route if it has been successfully
            // communicated over either via an outbound test, or used as a private route inbound
            // and we are replying over the same route as our safety route outbound
            let optimize = safety_rssd.get_stats().last_known_valid_ts.is_some();

            // Get the first hop noderef of the safety route
            let first_hop = safety_rssd.hop_node_ref(0).unwrap_or_log();

            // Ensure sequencing requirement is set on first hop
            let mut first_hop = first_hop.default_filtered_with_sequencing(safety_spec.sequencing);

            // Enforce the routing domain
            first_hop.merge_filter(
                NodeRefFilter::new().with_routing_domain(RoutingDomain::PublicInternet),
            );

            // Get the safety route secret key
            let secret = safety_rsd.secret_key.clone();

            // Get the hop node refs
            let hop_node_refs = safety_rssd.hop_node_refs();

            // See if we have a cached route we can use
            if optimize {
                if let Some(safety_route) = inner
                    .cache
                    .lookup_compiled_route_cache(sr_pubkey.clone(), pr_pubkey.clone())
                {
                    // Build compiled route
                    let compiled_route = CompiledRoute {
                        safety_route,
                        secret,
                        first_hop,
                    };
                    // Return compiled route
                    //veilid_log!(self info "compile_safety_route profile (cached): {} us", (get_timestamp() - profile_start_ts));
                    return Ok(compiled_route);
                }
            }

            (optimize, first_hop, secret, hop_node_refs)
        };

        // Create hops
        let hops = {
            // start last blob-to-encrypt data off as private route
            let mut blob_data = {
                let mut pr_message = ::capnp::message::Builder::new_default();
                let mut pr_builder = pr_message.init_root::<veilid_capnp::private_route::Builder>();
                encode_private_route(&private_route, &mut pr_builder)?;
                let mut blob_data = canonical_message_builder_to_vec_packed(pr_message)?;

                // append the private route tag so we know how to decode it later
                blob_data.push(1u8);
                blob_data
            };

            // Encode each hop from inside to outside
            // skips the outermost hop since that's entering the
            // safety route and does not include the dialInfo
            // (outer hop is a RouteHopData, not a RouteHop).
            // Each loop mutates 'nonce', and 'blob_data'
            let mut nonce = vcrypto.random_nonce().await;

            let mut hop_info = Vec::with_capacity(hop_node_refs.len() - 1);
            let first_hop_public_key = {
                let rti = &*routing_table.inner.read();
                // Forward order (safety route), but inside-out
                for h in (1..hop_node_refs.len()).rev() {
                    let hop_node_ref = hop_node_refs[h].locked(rti);
                    let (hop_node_id, hop_public_key, hop_peer_info) = {
                        hop_node_ref.operate(|_rti, e| {
                            let Some(hop_node_id) = e.node_ids().get(crypto_kind) else {
                                apibail_invalid_argument!(
                                    "no hop node id for route hop",
                                    "crypto_kind",
                                    crypto_kind
                                );
                            };
                            let Some(hop_public_key) = e
                                .public_keys(RoutingDomain::PublicInternet)
                                .get(crypto_kind)
                            else {
                                apibail_invalid_argument!(
                                    "no hop public key for route hop",
                                    "crypto_kind",
                                    crypto_kind
                                );
                            };
                            let Some(hop_peer_info) =
                                e.get_peer_info(RoutingDomain::PublicInternet)
                            else {
                                apibail_invalid_argument!(
                                    "no hop peer info for route hop",
                                    "crypto_kind",
                                    crypto_kind
                                );
                            };
                            Ok((hop_node_id, hop_public_key, hop_peer_info))
                        })?
                    };

                    hop_info.push((hop_node_id, hop_public_key, hop_peer_info));
                }

                let first_hop_node_ref = hop_node_refs[0].locked(rti);
                let Some(first_hop_public_key) = first_hop_node_ref
                    .public_keys(RoutingDomain::PublicInternet)
                    .get(crypto_kind)
                else {
                    apibail_invalid_argument!(
                        "no hop public key for route hop",
                        "crypto_kind",
                        crypto_kind
                    );
                };
                first_hop_public_key
            };

            for (hop_node_id, hop_public_key, hop_peer_info) in hop_info {
                // Get blob to encrypt for next hop
                blob_data = {
                    // Encrypt the previous blob ENC(nonce, DH(PKhop,SKsr))
                    let dh_secret = vcrypto
                        .cached_dh(&hop_public_key, &secret)
                        .await
                        .map_err(VeilidAPIError::internal)?;
                    let enc_msg_data = vcrypto
                        .encrypt_aead(blob_data.as_slice(), &nonce, &dh_secret, None)
                        .await
                        .map_err(VeilidAPIError::internal)?;

                    // Make route hop data
                    let route_hop_data = RouteHopData {
                        nonce,
                        blob: enc_msg_data,
                    };

                    // Make route hop
                    let route_hop = RouteHop {
                        node: if optimize {
                            // Optimized, no peer info, just the dht key
                            RouteNode::NodeId(hop_node_id)
                        } else {
                            // Full peer info, required until we are sure the route has been fully established
                            RouteNode::PeerInfo(hop_peer_info)
                        },
                        next_hop: Some(route_hop_data),
                    };

                    // Make next blob from route hop
                    let mut rh_message = ::capnp::message::Builder::new_default();
                    let mut rh_builder = rh_message.init_root::<veilid_capnp::route_hop::Builder>();
                    encode_route_hop(&route_hop, &mut rh_builder)?;
                    let mut blob_data = canonical_message_builder_to_vec_packed(rh_message)?;

                    // Append the route hop tag so we know how to decode it later
                    blob_data.push(0u8);
                    blob_data
                };

                // Make another nonce for the next hop
                nonce = vcrypto.random_nonce().await;
            }

            // Encode first RouteHopData

            let dh_secret = vcrypto
                .cached_dh(&first_hop_public_key, &secret)
                .await
                .map_err(VeilidAPIError::internal)?;
            let enc_msg_data = vcrypto
                .encrypt_aead(blob_data.as_slice(), &nonce, &dh_secret, None)
                .await
                .map_err(VeilidAPIError::internal)?;

            let route_hop_data = RouteHopData {
                nonce,
                blob: enc_msg_data,
            };

            SafetyRouteHops::Data(route_hop_data)
        };

        // Build safety route
        let safety_route = SafetyRoute {
            public_key: sr_pubkey,
            hops,
        };

        // Add to cache but only if we have an optimized route
        if optimize {
            let mut inner = self.inner.write();
            inner
                .cache
                .add_to_compiled_route_cache(pr_pubkey.clone(), safety_route.clone());
        }

        // Build compiled route
        let compiled_route = CompiledRoute {
            safety_route,
            secret,
            first_hop,
        };

        // Release compile lock (doing this explicitly here for clarity)
        drop(compile_lock);

        // Return compiled route
        //veilid_log!(self info "compile_safety_route profile (uncached): {} us", (get_timestamp() - profile_start_ts));
        Ok(compiled_route)
    }
}
