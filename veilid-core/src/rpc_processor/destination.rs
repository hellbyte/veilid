use super::*;

impl_veilid_log_facility!("rpc");

/// Where to send an RPC message
#[derive(Debug, Clone)]
pub(crate) enum Destination {
    /// Send to node directly
    Direct {
        /// The node to send to
        node: FilteredNodeRef,
        /// Require safety route or not
        safety_selection: SafetySelection,
    },
    /// Send to node for relay purposes
    Relay {
        /// The relay to send to
        relay: FilteredNodeRef,
        /// The final destination the relay should send to
        node: NodeRef,
        /// Require safety route or not
        safety_selection: SafetySelection,
    },
    /// Send to private route
    PrivateRoute {
        /// A private route to send to
        private_route: PrivateRoute,
        /// Require safety route or not
        safety_selection: SafetySelection,
    },
}

/// Routing configuration for destination
#[derive(Debug, Clone)]
pub(crate) struct UnsafeRoutingInfo {
    pub opt_node: Option<NodeRef>,
    pub opt_relay: Option<NodeRef>,
    pub opt_routing_domain: Option<RoutingDomain>,
}

impl Destination {
    pub fn direct(node: FilteredNodeRef) -> Self {
        let sequencing = node.sequencing();
        Self::Direct {
            node,
            safety_selection: SafetySelection::Unsafe(sequencing),
        }
    }
    pub fn relay(relay: FilteredNodeRef, node: NodeRef) -> Self {
        let sequencing = relay.sequencing().max(node.sequencing());
        Self::Relay {
            relay,
            node,
            safety_selection: SafetySelection::Unsafe(sequencing),
        }
    }
    pub fn private_route(private_route: PrivateRoute, safety_selection: SafetySelection) -> Self {
        Self::PrivateRoute {
            private_route,
            safety_selection,
        }
    }

    pub fn with_safety(self, safety_selection: SafetySelection) -> Self {
        match self {
            Destination::Direct {
                node,
                safety_selection: _,
            } => Self::Direct {
                node,
                safety_selection,
            },
            Destination::Relay {
                relay,
                node,
                safety_selection: _,
            } => Self::Relay {
                relay,
                node,
                safety_selection,
            },
            Destination::PrivateRoute {
                private_route,
                safety_selection: _,
            } => Self::PrivateRoute {
                private_route,
                safety_selection,
            },
        }
    }

    pub fn get_safety_selection(&self) -> &SafetySelection {
        match self {
            Destination::Direct {
                node: _,
                safety_selection,
            } => safety_selection,
            Destination::Relay {
                relay: _,
                node: _,
                safety_selection,
            } => safety_selection,
            Destination::PrivateRoute {
                private_route: _,
                safety_selection,
            } => safety_selection,
        }
    }

    pub fn get_target_node_ids(&self) -> Option<NodeIdGroup> {
        match self {
            Destination::Direct {
                node,
                safety_selection: _,
            } => Some(node.node_ids()),
            Destination::Relay {
                relay: _,
                node,
                safety_selection: _,
            } => Some(node.node_ids()),
            Destination::PrivateRoute {
                private_route: _,
                safety_selection: _,
            } => None,
        }
    }

    pub fn get_target(&self, routing_table: &RoutingTable) -> Result<Target, RPCError> {
        match self {
            Destination::Direct {
                node,
                safety_selection: _,
            } => {
                Ok(Target::NodeId(node.best_node_id().ok_or_else(|| {
                    RPCError::protocol("no supported node id")
                })?))
            }
            Destination::Relay {
                relay: _,
                node,
                safety_selection: _,
            } => {
                Ok(Target::NodeId(node.best_node_id().ok_or_else(|| {
                    RPCError::protocol("no supported node id")
                })?))
            }
            Destination::PrivateRoute {
                private_route,
                safety_selection: _,
            } => {
                // Add the remote private route if we're going to keep the id
                let route_id = routing_table
                    .route_spec_store()
                    .import_single_remote_route(private_route.clone())
                    .map_err(RPCError::protocol)?;

                Ok(Target::RouteId(route_id))
            }
        }
    }

    pub fn get_unsafe_routing_info(
        &self,
        routing_table: &RoutingTable,
    ) -> Option<UnsafeRoutingInfo> {
        // If there's a safety route in use, the safety route will be responsible for the routing
        match self.get_safety_selection() {
            SafetySelection::Unsafe(_) => {}
            SafetySelection::Safe(_) => {
                return None;
            }
        }

        // Get:
        // * The target node (possibly relayed)
        // * The routing domain we are sending to if we can determine it
        let (opt_node, opt_relay, opt_routing_domain) = match self {
            Destination::Direct {
                node,
                safety_selection: _,
            } => {
                let opt_routing_domain = node.best_routing_domain();
                if opt_routing_domain.is_none() {
                    // No routing domain for target, no node info
                    // Only a stale connection or no connection exists
                    veilid_log!(node warn "No routing domain for node: node={}", node);
                };
                (Some(node.unfiltered()), None, opt_routing_domain)
            }
            Destination::Relay {
                relay,
                node,
                safety_selection: _,
            } => {
                // Outbound relays are defined as routing to and from PublicInternet only right now

                // Resolve the relay for this target's routing domain and see if it matches this relay
                let mut opt_routing_domain = None;
                for target_rd in node.routing_domain_set() {
                    // Check out inbound/outbound relay to match routing domain
                    for rdr in routing_table.relays(target_rd) {
                        if relay.same_entry(&rdr.relay_node) {
                            // Relay for this destination is one of our routing domain relays (our inbound or outbound)
                            opt_routing_domain = Some(target_rd);
                            break;
                        }
                    }
                    if opt_routing_domain.is_some() {
                        break;
                    }
                    // Check remote node's published relay to see if that who is relaying
                    for relay_id in node
                        .node_info(target_rd)
                        .map(|ni| ni.relay_ids())
                        .unwrap_or_default()
                    {
                        if relay.node_ids().contains(&relay_id) {
                            // Relay for this destination is one of its published relays
                            opt_routing_domain = Some(target_rd);
                            break;
                        }
                    }
                    if opt_routing_domain.is_some() {
                        break;
                    }
                }
                if opt_routing_domain.is_none() {
                    // In the case of an unexpected relay, log it and don't pass any sender peer info into an unexpected relay
                    veilid_log!(node debug "Unexpected relay: relay={}, node={}", relay, node);
                };

                (
                    Some(node.clone()),
                    Some(relay.unfiltered()),
                    opt_routing_domain,
                )
            }
            Destination::PrivateRoute {
                private_route: _,
                safety_selection: _,
            } => (None, None, Some(RoutingDomain::PublicInternet)),
        };

        Some(UnsafeRoutingInfo {
            opt_node,
            opt_relay,
            opt_routing_domain,
        })
    }

    pub fn destination_key(&self) -> Result<PublicKey, RPCError> {
        match self {
            Destination::Direct {
                node,
                safety_selection: _,
            } => {
                let routing_domain = node
                    .best_routing_domain()
                    .ok_or_else(|| RPCError::internal("no reachable routing domain"))?;
                let public_key = node
                    .best_public_key(routing_domain)
                    .ok_or_else(|| RPCError::internal("no public key in routing domain"))?;
                Ok(public_key)
            }
            Destination::Relay {
                relay: _,
                node,
                safety_selection: _,
            } => {
                let routing_domain = node
                    .best_routing_domain()
                    .ok_or_else(|| RPCError::internal("no reachable routing domain"))?;
                let public_key = node
                    .best_public_key(routing_domain)
                    .ok_or_else(|| RPCError::internal("no public key in routing domain"))?;
                Ok(public_key)
            }
            Destination::PrivateRoute {
                private_route,
                safety_selection: _,
            } => Ok(private_route.public_key.clone()),
        }
    }
}

impl fmt::Display for Destination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Destination::Direct {
                node,
                safety_selection,
            } => {
                let sr = if matches!(safety_selection, SafetySelection::Safe(_)) {
                    "%SR"
                } else {
                    ""
                };

                write!(f, "{}{}", node, sr)
            }
            Destination::Relay {
                relay,
                node,
                safety_selection,
            } => {
                let sr = if matches!(safety_selection, SafetySelection::Safe(_)) {
                    "%SR"
                } else {
                    ""
                };

                write!(f, "{}@{}{}", node, relay, sr)
            }
            Destination::PrivateRoute {
                private_route,
                safety_selection,
            } => {
                let sr = if matches!(safety_selection, SafetySelection::Safe(_)) {
                    "%SR"
                } else {
                    ""
                };

                write!(f, "PR%{}{}", private_route.public_key, sr)
            }
        }
    }
}

impl RPCProcessor {
    /// Convert a 'Target' into a 'Destination'
    pub async fn resolve_target_to_destination(
        &self,
        target: Target,
        safety_selection: SafetySelection,
    ) -> Result<rpc_processor::Destination, RPCError> {
        match target {
            Target::NodeId(node_id) => {
                // Resolve node
                let nr = match self.resolve_node(node_id, safety_selection.clone()).await? {
                    Some(nr) => nr,
                    None => {
                        return Err(RPCError::network("could not resolve node id"));
                    }
                };
                // Apply sequencing to match safety selection
                let nr = nr.default_filtered_with_sequencing(safety_selection.get_sequencing());

                Ok(rpc_processor::Destination::Direct {
                    node: nr,
                    safety_selection,
                })
            }
            Target::RouteId(rsid) => {
                // Get remote private route
                let Some(private_route) = self
                    .routing_table()
                    .route_spec_store()
                    .best_remote_private_route(&rsid)
                else {
                    return Err(RPCError::network("could not get remote private route"));
                };

                Ok(rpc_processor::Destination::PrivateRoute {
                    private_route,
                    safety_selection,
                })
            }
        }
    }

    /// Convert the 'Destination' into a 'RespondTo' for a response
    pub(super) fn get_destination_respond_to(
        &self,
        dest: &Destination,
    ) -> RPCNetworkResult<RespondTo> {
        let routing_table = self.routing_table();
        let rss = routing_table.route_spec_store();

        match dest {
            Destination::Direct {
                node: target,
                safety_selection,
            } => match safety_selection {
                SafetySelection::Unsafe(_) => {
                    // Sent directly with no safety route, can respond directly
                    Ok(NetworkResult::value(RespondTo::Sender))
                }
                SafetySelection::Safe(safety_spec) => {
                    // Sent directly but with a safety route, respond to private route
                    let crypto_kind = target
                        .best_node_id()
                        .ok_or_else(|| RPCError::protocol("no supported node id"))?
                        .kind();
                    let pr_key = network_result_try!(rss
                        .select_single_route(crypto_kind, safety_spec, &target.node_ids(), false)
                        .to_rpc_network_result()?);

                    // Get the assembled route for response
                    let private_route = network_result_try!(rss
                        .assemble_single_private_route(&pr_key, None)
                        .to_rpc_network_result()?);

                    Ok(NetworkResult::Value(RespondTo::PrivateRoute(private_route)))
                }
            },
            Destination::Relay {
                relay,
                node: target,
                safety_selection,
            } => match safety_selection {
                SafetySelection::Unsafe(_) => {
                    // Sent via a relay with no safety route, can respond directly
                    Ok(NetworkResult::value(RespondTo::Sender))
                }
                SafetySelection::Safe(safety_spec) => {
                    // Sent via a relay but with a safety route, respond to private route
                    let crypto_kind = target
                        .best_node_id()
                        .ok_or_else(|| RPCError::protocol("no supported node id"))?
                        .kind();

                    let mut avoid_nodes = relay.node_ids();
                    avoid_nodes.add_all_from_slice(&target.node_ids());
                    let allocated_route_key = network_result_try!(rss
                        .select_single_route(crypto_kind, safety_spec, &avoid_nodes, false)
                        .to_rpc_network_result()?);

                    // Get the assembled route for response
                    let private_route = network_result_try!(rss
                        .assemble_single_private_route(&allocated_route_key, None)
                        .to_rpc_network_result()?);

                    Ok(NetworkResult::Value(RespondTo::PrivateRoute(private_route)))
                }
            },
            Destination::PrivateRoute {
                private_route,
                safety_selection,
            } => {
                let Some(avoid_node_id) = private_route.first_hop_node_id() else {
                    return Err(RPCError::internal(
                        "destination private route must have first hop",
                    ));
                };

                let crypto_kind = private_route.public_key.kind();

                match safety_selection {
                    SafetySelection::Unsafe(_) => {
                        // Sent to a private route with no safety route, use a stub safety route for the response

                        let Some(published_peer_info) =
                            routing_table.get_published_peer_info(RoutingDomain::PublicInternet)
                        else {
                            return Ok(NetworkResult::service_unavailable(
                                "Own node info must be published to use private route",
                            ));
                        };

                        // Determine if we can use optimized nodeinfo
                        let route_node = if rss.has_remote_private_route_seen_our_node_info(
                            &private_route.public_key,
                            &published_peer_info,
                        ) {
                            RouteNode::NodeId(routing_table.node_id(crypto_kind))
                        } else {
                            RouteNode::PeerInfo(published_peer_info)
                        };

                        Ok(NetworkResult::value(RespondTo::PrivateRoute(
                            PrivateRoute::new_stub(
                                routing_table.public_key(crypto_kind),
                                route_node,
                            ),
                        )))
                    }
                    SafetySelection::Safe(safety_spec) => {
                        // Sent to a private route via a safety route, respond to private route

                        // Check for loopback test
                        let opt_private_route_id =
                            rss.get_route_id_for_key(&private_route.public_key);
                        let pr_key = if opt_private_route_id.is_some()
                            && safety_spec.preferred_route == opt_private_route_id
                        {
                            // Private route is also safety route during loopback test
                            private_route.public_key.clone()
                        } else {
                            // Get the private route to respond to that matches the safety route spec we sent the request with
                            network_result_try!(rss
                                .select_single_route(
                                    crypto_kind,
                                    safety_spec,
                                    &[avoid_node_id],
                                    true,
                                )
                                .to_rpc_network_result()?)
                        };

                        // Get the assembled route for response
                        let private_route = network_result_try!(rss
                            .assemble_single_private_route(&pr_key, None)
                            .to_rpc_network_result()?);

                        Ok(NetworkResult::Value(RespondTo::PrivateRoute(private_route)))
                    }
                }
            }
        }
    }

    /// Convert the 'RespondTo' into a 'Destination' for a response
    pub(super) fn get_respond_to_destination(
        &self,
        request: &Message,
    ) -> NetworkResult<Destination> {
        // Get the question 'respond to'
        let respond_to = match request.operation.kind() {
            RPCOperationKind::Question(q) => q.respond_to(),
            _ => {
                panic!("not a question");
            }
        };

        // To where should we respond?
        match respond_to {
            RespondTo::Sender => {
                // Parse out the header detail from the question
                let detail = match &request.header.detail {
                    RPCMessageHeaderDetail::Direct(detail) => detail,
                    RPCMessageHeaderDetail::SafetyRouted(_)
                    | RPCMessageHeaderDetail::PrivateRouted(_) => {
                        // If this was sent via a private route, we don't know what the sender was, so drop this
                        return NetworkResult::invalid_message(
                            "can't respond directly to non-direct question",
                        );
                    }
                };

                // Get the filtered noderef of the sender
                let sender_noderef = detail.sender_noderef.clone();
                NetworkResult::value(Destination::direct(sender_noderef))
            }
            RespondTo::PrivateRoute(pr) => {
                match &request.header.detail {
                    RPCMessageHeaderDetail::Direct(_) => {
                        // If this was sent directly, we should only ever respond directly
                        NetworkResult::invalid_message(
                            "not responding to private route from direct question",
                        )
                    }
                    RPCMessageHeaderDetail::SafetyRouted(detail) => {
                        // If this was sent via a safety route, but not received over our private route, don't respond with a safety route,
                        // it would give away which safety routes belong to this node
                        NetworkResult::value(Destination::private_route(
                            pr.clone(),
                            SafetySelection::Unsafe(detail.sequencing),
                        ))
                    }
                    RPCMessageHeaderDetail::PrivateRouted(detail) => {
                        // If this was received over our private route, it's okay to respond to a private route via our safety route
                        NetworkResult::value(Destination::private_route(
                            pr.clone(),
                            SafetySelection::Safe(detail.safety_spec.clone()),
                        ))
                    }
                }
            }
        }
    }
}
