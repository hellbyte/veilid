use super::*;

impl RouteSpecStore {
    /// Assemble a single private route for publication from an allocated route key
    /// Returns a PrivateRoute object for an allocated route key
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip_all, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub fn assemble_single_private_route(
        &self,
        allocated_route_key: &PublicKey,
        optimized: Option<bool>,
    ) -> VeilidAPIResult<PrivateRoute> {
        let inner: &RouteSpecStoreInner = &self.inner.lock();
        let Some(rsid) = inner.content.get_id_by_key(allocated_route_key) else {
            // Route doesn't exist
            apibail_invalid_target!("route id does not exist");
        };
        let Some(rssd) = inner.content.get_detail(&rsid) else {
            apibail_internal!("route id does not exist");
        };

        // See if we can optimize this compilation yet
        // We don't want to include full nodeinfo if we don't have to
        let optimized = optimized.unwrap_or(rssd.get_stats().last_known_valid_ts.is_some());

        let rsd = rssd
            .get_route_by_key(allocated_route_key)
            .expect_or_log("route key index is broken");

        self.assemble_single_private_route_inner(allocated_route_key, rsd, rssd, optimized)
    }

    /// Assemble private route set for publication
    /// Returns a vec of assembled PrivateRoute objects for an RouteId
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab::route", skip_all, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub fn assemble_private_route_set(
        &self,
        id: &RouteId,
        optimized: Option<bool>,
    ) -> VeilidAPIResult<Vec<PrivateRoute>> {
        let inner = &*self.inner.lock();
        let Some(rssd) = inner.content.get_detail(id) else {
            apibail_invalid_target!("route id does not exist");
        };

        // See if we can optimize this compilation yet
        // We don't want to include full nodeinfo if we don't have to
        let optimized = optimized.unwrap_or(rssd.get_stats().last_known_valid_ts.is_some());

        let mut out = Vec::new();
        for (key, rsd) in rssd.iter_route_set() {
            out.push(self.assemble_single_private_route_inner(key, rsd, rssd, optimized)?);
        }
        Ok(out)
    }

    pub(super) fn assemble_single_private_route_inner(
        &self,
        key: &PublicKey,
        rsd: &RouteSpecDetail,
        rssd: &RouteSetSpecDetail,
        optimized: bool,
    ) -> VeilidAPIResult<PrivateRoute> {
        let routing_table = self.routing_table();
        let rti = &*routing_table.inner.read();

        // Ensure we get the crypto for it
        let crypto = routing_table.network_manager().crypto();
        let crypto_kind = key.kind();
        let Some(vcrypto) = crypto.get(crypto_kind) else {
            apibail_invalid_argument!("crypto not supported for route", "crypto_kind", crypto_kind);
        };

        // Ensure our network class is valid before attempting to assemble any routes
        let Some(published_peer_info) = rti.get_published_peer_info(RoutingDomain::PublicInternet)
        else {
            apibail_try_again!("unable to assemble route until we have published peerinfo");
        };

        // Make innermost route hop to our own node
        let mut route_hop = RouteHop {
            node: if optimized {
                let Some(node_id) = routing_table.node_ids().get(crypto_kind) else {
                    apibail_invalid_argument!(
                        "missing node id for crypto kind",
                        "crypto_kind",
                        crypto_kind
                    );
                };
                RouteNode::NodeId(node_id)
            } else {
                RouteNode::PeerInfo(published_peer_info)
            },
            next_hop: None,
        };

        // Iterate hops in private route order (reverse, but inside out)
        for hop_node_ref in rssd.hop_node_refs() {
            let hop_node_ref = hop_node_ref.locked(rti);

            let Some(hop_node_id) = hop_node_ref.node_ids().get(crypto_kind) else {
                apibail_invalid_argument!(
                    "no hop node id for route hop",
                    "crypto_kind",
                    crypto_kind
                );
            };
            let Some(hop_public_key) = hop_node_ref
                .public_keys(RoutingDomain::PublicInternet)
                .get(crypto_kind)
            else {
                apibail_invalid_argument!(
                    "no hop public key for route hop",
                    "crypto_kind",
                    crypto_kind
                );
            };
            let Some(hop_peer_info) = hop_node_ref.get_peer_info(RoutingDomain::PublicInternet)
            else {
                apibail_invalid_argument!(
                    "no hop peer info for route hop",
                    "crypto_kind",
                    crypto_kind
                );
            };

            // Encrypt the previous blob ENC(nonce, DH(PKhop,SKpr))
            let nonce = vcrypto.random_nonce();

            let blob_data = {
                let mut rh_message = ::capnp::message::Builder::new_default();
                let mut rh_builder = rh_message.init_root::<veilid_capnp::route_hop::Builder>();
                encode_route_hop(&route_hop, &mut rh_builder)?;
                canonical_message_builder_to_vec_packed(rh_message)?
            };

            let dh_secret = vcrypto.cached_dh(&hop_public_key, &rsd.secret_key)?;
            let enc_msg_data =
                vcrypto.encrypt_aead(blob_data.as_slice(), &nonce, &dh_secret, None)?;
            let route_hop_data = RouteHopData {
                nonce,
                blob: enc_msg_data,
            };

            route_hop = RouteHop {
                node: if optimized {
                    // Optimized, no peer info, just the dht key
                    RouteNode::NodeId(hop_node_id)
                } else {
                    // Full peer info, required until we are sure the route has been fully established
                    RouteNode::PeerInfo(hop_peer_info)
                },
                next_hop: Some(route_hop_data),
            }
        }

        let private_route = PrivateRoute {
            public_key: key.clone(),
            hops: PrivateRouteHops::FirstHop(Box::new(route_hop)),
        };
        Ok(private_route)
    }
}
