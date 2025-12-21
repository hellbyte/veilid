#![cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), expect(dead_code))]

use super::*;

#[derive(Debug)]
enum RoutingDomainChangeLocalNetwork {
    SetInterfaceAddresses { interface_addresses: Vec<IfAddr> },
    Common(RoutingDomainChangeCommon),
}

pub struct RoutingDomainEditorLocalNetwork<'a> {
    routing_table: &'a RoutingTable,
    changes: Vec<RoutingDomainChangeLocalNetwork>,
}

impl<'a> RoutingDomainEditorLocalNetwork<'a> {
    pub(in crate::routing_table) fn new(routing_table: &'a RoutingTable) -> Self {
        Self {
            routing_table,
            changes: Vec::new(),
        }
    }

    pub fn set_interface_addresses(&mut self, interface_addresses: Vec<IfAddr>) -> &mut Self {
        self.changes
            .push(RoutingDomainChangeLocalNetwork::SetInterfaceAddresses {
                interface_addresses,
            });
        self
    }
}

impl RoutingDomainEditorCommonTrait for RoutingDomainEditorLocalNetwork<'_> {
    #[instrument(level = "debug", skip(self))]
    fn clear_dial_info_details(
        &mut self,
        address_type: Option<AddressType>,
        protocol_type: Option<ProtocolType>,
    ) -> &mut Self {
        self.changes.push(RoutingDomainChangeLocalNetwork::Common(
            RoutingDomainChangeCommon::ClearDialInfoDetails {
                address_type,
                protocol_type,
            },
        ));

        self
    }
    #[instrument(level = "debug", skip(self))]
    fn set_relays(&mut self, relays: Vec<RoutingDomainRelay>) -> &mut Self {
        self.changes.push(RoutingDomainChangeLocalNetwork::Common(
            RoutingDomainChangeCommon::SetRelays { relays },
        ));
        self
    }
    #[instrument(level = "debug", skip(self))]
    fn set_relay_state(
        &mut self,
        relay: RoutingDomainRelay,
        state: RoutingDomainRelayState,
    ) -> &mut Self {
        self.changes.push(RoutingDomainChangeLocalNetwork::Common(
            RoutingDomainChangeCommon::SetRelayState { relay, state },
        ));
        self
    }

    #[instrument(level = "debug", skip(self))]
    fn add_dial_info(&mut self, dial_info: DialInfo, class: DialInfoClass) -> &mut Self {
        self.changes.push(RoutingDomainChangeLocalNetwork::Common(
            RoutingDomainChangeCommon::AddDialInfo {
                dial_info_detail: DialInfoDetail {
                    dial_info: dial_info.clone(),
                    class,
                },
            },
        ));
        self
    }
    // #[instrument(level = "debug", skip_all)]
    // fn retain_dial_info<F: Fn(&DialInfo, DialInfoClass) -> bool>(
    //     &mut self,
    //     closure: F,
    // ) -> EyreResult<&mut Self> {
    //     let dids = self.routing_table.dial_info_details(self.routing_domain);
    //     for did in dids {
    //         if !closure(&did.dial_info, did.class) {
    //             self.changes
    //                 .push(RoutingDomainChangePublicInternet::Common(RoutingDomainChange::RemoveDialInfoDetail {
    //                     dial_info_detail: did,
    //                 }));
    //         }
    //     }

    //     Ok(self)
    // }

    #[instrument(level = "debug", skip(self))]
    fn setup_network(
        &mut self,
        outbound_protocols: ProtocolTypeSet,
        inbound_protocols: ProtocolTypeSet,
        address_types: AddressTypeSet,
        capabilities: Vec<VeilidCapability>,
        confirmed: bool,
    ) -> &mut Self {
        self.changes.push(RoutingDomainChangeLocalNetwork::Common(
            RoutingDomainChangeCommon::SetupNetwork {
                outbound_protocols,
                inbound_protocols,
                address_types,
                capabilities,
                confirmed,
            },
        ));
        self
    }

    #[instrument(level = "debug", skip(self))]
    fn commit(&mut self, pause_tasks: bool) -> PinBoxFuture<'_, bool> {
        Box::pin(async move {
            // No locking if we have nothing to do
            if self.changes.is_empty() {
                return false;
            }
            // Briefly pause routing table ticker while changes are made
            let _tick_guard = if pause_tasks {
                Some(self.routing_table.pause_tasks().await)
            } else {
                None
            };

            // Apply changes
            let mut peer_info_changed = false;
            {
                let mut rti_lock = self.routing_table.inner.write();
                let rti = &mut rti_lock;
                let detail = &mut rti.local_network_routing_domain;
                {
                    let old_dial_info_details = detail.dial_info_details().clone();
                    let old_relays = detail.relays();
                    let old_outbound_protocols = detail.outbound_protocols();
                    let old_inbound_protocols = detail.inbound_protocols();
                    let old_address_types = detail.address_types();
                    let old_capabilities = detail.capabilities();
                    let old_confirmed = detail.confirmed();

                    for change in self.changes.drain(..) {
                        match change {
                            RoutingDomainChangeLocalNetwork::Common(common_change) => {
                                detail.apply_common_change(common_change);
                            }
                            RoutingDomainChangeLocalNetwork::SetInterfaceAddresses {
                                interface_addresses,
                            } => {
                                detail.set_interface_addresses(interface_addresses);
                            }
                        }
                    }

                    let new_dial_info_details = detail.dial_info_details().clone();
                    let new_relays = detail.relays();
                    let new_outbound_protocols = detail.outbound_protocols();
                    let new_inbound_protocols = detail.inbound_protocols();
                    let new_address_types = detail.address_types();
                    let new_capabilities = detail.capabilities();
                    let new_confirmed = detail.confirmed();

                    // Compare and see if peerinfo needs republication
                    let removed_dial_info = old_dial_info_details
                        .iter()
                        .filter(|di| !new_dial_info_details.contains(di))
                        .collect::<Vec<_>>();
                    if !removed_dial_info.is_empty() {
                        veilid_log!(rti info
                            "[LocalNetwork] removed dial info:\n{}",
                            indent_all_string(&removed_dial_info.to_multiline_string())
                                .strip_trailing_newline()
                        );
                        peer_info_changed = true;
                    }
                    let added_dial_info = new_dial_info_details
                        .iter()
                        .filter(|di| !old_dial_info_details.contains(di))
                        .collect::<Vec<_>>();
                    if !added_dial_info.is_empty() {
                        veilid_log!(rti info
                            "[LocalNetwork] added dial info:\n{}",
                            indent_all_string(&added_dial_info.to_multiline_string())
                                .strip_trailing_newline()
                        );
                        peer_info_changed = true;
                    }

                    if old_relays.len() != new_relays.len()
                        || old_relays
                            .iter()
                            .zip(new_relays.iter())
                            .any(|x| !x.0.relay_node.same_entry(&x.1.relay_node))
                    {
                        veilid_log!(rti info "[LocalNetwork] relays changed: [{}] -> [{}]",
                                old_relays.iter().map(|x| x.relay_node.to_string()).collect::<Vec<_>>().join(","),
                                new_relays.iter().map(|x| x.relay_node.to_string()).collect::<Vec<_>>().join(","));
                        peer_info_changed = true;
                    }

                    if old_outbound_protocols != new_outbound_protocols {
                        veilid_log!(rti info
                            "[LocalNetwork] changed network: outbound {:?}->{:?}",
                            old_outbound_protocols, new_outbound_protocols
                        );
                        peer_info_changed = true;
                    }
                    if old_inbound_protocols != new_inbound_protocols {
                        veilid_log!(rti info
                            "[LocalNetwork] changed network: inbound {:?}->{:?}",
                            old_inbound_protocols, new_inbound_protocols
                        );
                        peer_info_changed = true;
                    }
                    if old_address_types != new_address_types {
                        veilid_log!(rti info
                            "[LocalNetwork] changed network: address types {:?}->{:?}",
                            old_address_types, new_address_types
                        );
                        peer_info_changed = true;
                    }
                    if old_capabilities != new_capabilities {
                        veilid_log!(rti info
                            "[LocalNetwork] changed network: capabilities {:?}->{:?}",
                            old_capabilities, new_capabilities
                        );
                        peer_info_changed = true;
                    }
                    if old_confirmed != new_confirmed {
                        veilid_log!(rti info
                            "[LocalNetwork] changed confirmation: {:?}->{:?}",
                            old_confirmed, new_confirmed
                        );
                        peer_info_changed = true;
                    }
                }

                if peer_info_changed {
                    // Allow signed node info updates at same timestamp for otherwise dead nodes if our network has changed
                    rti.reset_all_updated_since_last_network_change();
                }
            }

            // Operations that require an unlocked routing table go here
            if peer_info_changed {
                // Update protections
                self.routing_table
                    .network_manager()
                    .connection_manager()
                    .update_protections();
            }
            peer_info_changed
        })
    }

    #[instrument(level = "debug", skip(self))]
    fn publish(&mut self) {
        self.routing_table
            .inner
            .write()
            .publish_peer_info(RoutingDomain::LocalNetwork);
    }

    #[instrument(level = "debug", skip(self))]
    fn shutdown(&mut self) -> PinBoxFuture<'_, ()> {
        Box::pin(async move {
            self.clear_dial_info_details(None, None)
                .set_relays(vec![])
                .commit(true)
                .await;
            self.routing_table
                .inner
                .write()
                .unpublish_peer_info(RoutingDomain::LocalNetwork);
        })
    }
}
