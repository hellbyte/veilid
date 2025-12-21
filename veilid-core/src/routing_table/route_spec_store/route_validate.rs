use super::*;

impl RouteSpecStore {
    /// validate data using a private route's key and signature chain
    #[instrument(level = "trace", target = "route", skip(self, data, callback), ret)]
    pub fn with_signature_validated_route<F, R>(
        &self,
        public_key: &PublicKey,
        signatures: &[Signature],
        data: &[u8],
        last_hop_id: &NodeId,
        callback: F,
    ) -> Option<R>
    where
        F: FnOnce(&RouteSetSpecDetail, &RouteSpecDetail) -> R,
        R: fmt::Debug,
    {
        let inner = &*self.inner.lock();
        let crypto = self.crypto();

        let Some(rsid) = inner.content.get_id_by_key(public_key) else {
            veilid_log!(self debug target: "network_result", "route id does not exist: {:?}", public_key.ref_value());
            return None;
        };
        let Some(rssd) = inner.content.get_detail(&rsid) else {
            veilid_log!(self debug "route detail does not exist: {:?}", rsid);
            return None;
        };
        let Some(rsd) = rssd.get_route_by_key(public_key) else {
            veilid_log!(self debug "route set {:?} does not have key: {:?}", rsid, public_key.ref_value());
            return None;
        };

        // Ensure we have the right number of signatures
        if signatures.len() != rsd.hops.len() - 1 {
            // Wrong number of signatures
            veilid_log!(self debug "wrong number of signatures ({} should be {}) for routed operation on private route {}", signatures.len(), rsd.hops.len() - 1, public_key);
            return None;
        }
        // Validate signatures to ensure the route was handled by the nodes and not messed with
        // This is in private route (reverse) order as we are receiving over the route
        for (hop_n, hop_node_ref) in rssd.hop_node_refs().iter().rev().enumerate() {
            // The last hop is not signed, as the whole packet is signed
            if hop_n == signatures.len() {
                // Verify the node we received the routed operation from is the last hop in our route
                if !hop_node_ref.node_ids().contains(last_hop_id) {
                    veilid_log!(self debug "received routed operation from the wrong hop ({} should be {}) on private route {}", hop_node_ref, last_hop_id, public_key);
                    return None;
                }
            } else {
                let Some(hop_public_key) = hop_node_ref
                    .public_keys(RoutingDomain::PublicInternet)
                    .get(signatures[hop_n].kind())
                else {
                    veilid_log!(self debug "no hop public key matching signature kind {} at hop {} for routed operation on private route {}", signatures[hop_n].kind(), hop_n, public_key);
                    return None;
                };
                // Verify a signature for a hop node along the route
                let Some(vcrypto) = crypto.get(hop_public_key.kind()) else {
                    veilid_log!(self debug "can't handle route hop with public key: {:?}", hop_public_key.kind());
                    return None;
                };
                match vcrypto.verify(&hop_public_key, data, &signatures[hop_n]) {
                    Ok(true) => {}
                    Ok(false) => {
                        veilid_log!(self debug "invalid signature for hop {} at {} on private route {}", hop_n, hop_node_ref, public_key);
                        return None;
                    }
                    Err(e) => {
                        veilid_log!(self debug "error verifying signature for hop {} at {} on private route {}: {}", hop_n, hop_node_ref, public_key, e);
                        return None;
                    }
                }
            }
        }
        // We got the correct signatures, return a key and response safety spec
        Some(callback(rssd, rsd))
    }
}
