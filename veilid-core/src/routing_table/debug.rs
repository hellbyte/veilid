use super::*;

impl RoutingTable {
    pub fn debug_info_nodeid(&self) -> String {
        let mut out = String::new();
        for nid in self.node_ids().iter() {
            out += &format!("{}\n", nid);
        }
        out
    }

    pub fn debug_info_nodeinfo(&self) -> String {
        let mut out = String::new();
        let inner = self.inner.read();
        out += &format!("Node Ids: {}\n", self.node_ids());
        out += &format!(
            "Version: {}{}\n",
            veilid_version_string(),
            if cfg!(debug_assertions) { "-debug" } else { "" }
        );
        out += &format!("Features: {:?}\n", veilid_features());
        out += &format!(
            "Self Transfer Stats:\n{}",
            indent_all_string(&inner.self_transfer_stats)
        );

        out
    }

    pub fn debug_info_dialinfo(&self) -> String {
        let mut out = String::new();

        for rd in RoutingDomainSet::all() {
            out += &format!("{:?}:\n", rd);

            let mut rdout = String::new();
            rdout += "Dial Info Details:\n";
            let ldis = self.dial_info_details(rd);
            for (n, ldi) in ldis.iter().enumerate() {
                rdout += &indent_all_string(&format!("{:>2}: {}\n", n, ldi));
            }
            rdout += &format!(
                "Routing Domain State:\n{:?}\n",
                self.routing_domain_state(rd)
            );
            rdout += "Routing Domain Relays:\n";
            for (n, rdr) in self.relays(rd).iter().enumerate() {
                rdout += &indent_all_string(&format!("{:>2}: {:?}\n", n, rdr));
            }
            rdout += "Routing Domain Details:\n";
            rdout += &self.routing_domain_debug(rd, true);

            out += &(indent_all_string(&rdout) + "\n");
        }

        out
    }

    pub fn debug_info_peerinfo(&self, routing_domain: RoutingDomain, published: bool) -> String {
        let mut out = String::new();
        if published {
            let pistr = if let Some(pi) = self.get_published_peer_info(routing_domain) {
                format!("\n{}\n", indent_all_string(&pi))
            } else {
                " None".to_owned()
            };
            out += &format!("{:?} Published PeerInfo:{}", routing_domain, pistr);
        } else {
            let pi = self.get_current_peer_info(routing_domain);
            let pistr = format!("\n{}\n", indent_all_string(&pi));
            out += &format!("{:?} Current PeerInfo:{}", routing_domain, pistr);
        }
        out
    }

    fn format_state_reason(state_reason: BucketEntryStateReason) -> &'static str {
        match state_reason {
            BucketEntryStateReason::Punished(p) => match p {
                PunishmentReason::Manual => "PMANUL",
                PunishmentReason::FailedToDecryptEnvelopeBody => "PCRYPT",
                PunishmentReason::FailedToDecodeEnvelope => "PDECEN",
                PunishmentReason::ShortPacket => "PSHORT",
                PunishmentReason::InvalidFraming => "PFRAME",
                PunishmentReason::FailedToDecodeOperation => "PDECOP",
                PunishmentReason::WrongSenderPeerInfo => "PSPBAD",
                // PunishmentReason::FailedToVerifySenderPeerInfo => "PSPVER",
                PunishmentReason::FailedToRegisterSenderPeerInfo => "PSPREG",
                //
            },
            BucketEntryStateReason::Dead(d) => match d {
                BucketEntryDeadReason::CanNotSend => "DFSEND",
                BucketEntryDeadReason::TooManyLostAnswers => "DALOST",
                BucketEntryDeadReason::NoPingResponse => "DNOPNG",
            },
            BucketEntryStateReason::Unreliable(u) => match u {
                BucketEntryUnreliableReason::FailedToSend => "UFSEND",
                BucketEntryUnreliableReason::LostAnswers => "UALOST",
                BucketEntryUnreliableReason::NotSeenConsecutively => "UNSEEN",
                BucketEntryUnreliableReason::InUnreliablePingSpan => "UUPING",
                //
            },
            BucketEntryStateReason::Reliable => "RELIBL",
        }
    }

    fn format_entry(
        cur_ts: Timestamp,
        node_id_str: &str,
        e: &BucketEntryInner,
        relay_tag: &str,
    ) -> String {
        let state_reason = Self::format_state_reason(e.state_reason(cur_ts));

        let average_latency = e
            .peer_stats()
            .latency
            .as_ref()
            .map(|l| l.to_string())
            .unwrap_or_else(|| "???".to_string());

        let capabilities = if let Some(ni) = e.node_info(RoutingDomain::PublicInternet) {
            ni.capabilities()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(",")
        } else {
            "???".to_owned()
        };

        let since_last_question = e
            .peer_stats()
            .rpc_stats
            .last_question_ts
            .as_ref()
            .map(|l| cur_ts.duration_since(*l).to_string())
            .unwrap_or_else(|| "???".to_string());

        let since_last_seen = e
            .peer_stats()
            .rpc_stats
            .last_seen_ts
            .as_ref()
            .map(|l| cur_ts.duration_since(*l).to_string())
            .unwrap_or_else(|| "???".to_string());

        #[allow(unused_mut)]
        let mut result = format!(
            "    {} [{}][{}] {} [{}] lastq@{} seen@{}",
            // node id
            node_id_str,
            // state reason
            state_reason,
            // Relay tag
            relay_tag,
            // average latency
            average_latency,
            // capabilities
            capabilities,
            // duration since last question
            since_last_question,
            // duration since last seen
            since_last_seen,
        );

        #[cfg(feature = "geolocation")]
        {
            let geolocation_info = e.geolocation_info();

            if let Some(cc) = geolocation_info.country_code() {
                result += &format!(" {cc}");
            } else {
                result += " ??";
            }

            if !geolocation_info.relay_country_codes().is_empty() {
                result += "/";
            }

            for (i, cc) in geolocation_info.relay_country_codes().iter().enumerate() {
                if i > 0 {
                    result += ",";
                }

                if let Some(cc) = cc {
                    result += &format!("{cc}");
                } else {
                    result += "??";
                }
            }
        }

        result
    }

    pub fn debug_info_entries(
        &self,
        min_state: BucketEntryState,
        capabilities: Vec<VeilidCapability>,
    ) -> String {
        let relay_node_filter = self.make_public_internet_relay_node_filter();

        let inner = self.inner.read();
        let inner = &*inner;
        let cur_ts = Timestamp::now();

        let mut out = String::new();

        out += &format!("Entries: {}\n", inner.bucket_entry_count());
        out += "   Live:\n";
        for ec in inner.cached_live_entry_counts().any_capabilities.iter() {
            let routing_domain = ec.0 .0;
            let crypto_kind = ec.0 .1;
            let count = ec.1;
            out += &format!("{:?}: {}: {}\n", routing_domain, crypto_kind, count);
        }
        for ck in &VALID_CRYPTO_KINDS {
            let our_node_id = self.node_id(*ck);

            let mut filtered_total = 0;
            let mut b = 0;
            let blen = inner.buckets[ck].len();
            while b < blen {
                let filtered_entries: Vec<(&BareNodeId, &Arc<BucketEntry>)> = inner.buckets[ck][b]
                    .entries()
                    .filter(|e| {
                        let cap_match = e.1.with(inner, |_rti, e| {
                            e.has_all_capabilities(RoutingDomain::PublicInternet, &capabilities)
                        });
                        let state = e.1.with(inner, |_rti, e| e.state(cur_ts));
                        state >= min_state && cap_match
                    })
                    .collect();
                filtered_total += filtered_entries.len();
                if !filtered_entries.is_empty() {
                    out += &format!("{} Bucket #{}:\n", ck, b);
                    for e in filtered_entries {
                        let node = e.0.clone();

                        let can_be_relay = e.1.with(inner, |_rti, e| relay_node_filter(e));
                        let is_relay = inner
                            .relays(RoutingDomain::PublicInternet)
                            .iter()
                            .find_map(|r| {
                                if r.relay_node.same_bucket_entry(e.1) {
                                    Some(true)
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();

                        let is_relaying =
                            e.1.with(inner, |_rti, e| {
                                e.node_info(RoutingDomain::PublicInternet)
                                    .map(|ni| ni.relay_ids().contains(&our_node_id))
                            })
                            .unwrap_or(false);
                        let relay_tag = format!(
                            "{}{}",
                            if is_relay {
                                "R"
                            } else if can_be_relay {
                                "r"
                            } else {
                                "-"
                            },
                            if is_relaying { ">" } else { "-" }
                        );

                        out += "    ";
                        out += &e.1.with(inner, |_rti, e| {
                            let node_id_str = NodeId::new(*ck, node).to_string();
                            Self::format_entry(cur_ts, &node_id_str, e, &relay_tag)
                        });
                        out += "\n";
                    }
                }
                b += 1;
            }
            out += &format!("{} Filtered Total: {}\n", ck, filtered_total);
        }

        out
    }

    pub fn debug_info_entries_fastest(
        &self,
        min_state: BucketEntryState,
        capabilities: Vec<VeilidCapability>,
        node_count: usize,
    ) -> String {
        let cur_ts = Timestamp::now();
        let relay_node_filter = self.make_public_internet_relay_node_filter();
        let our_node_ids = self.node_ids();
        let mut relay_count = 0usize;
        let mut relaying_count = 0usize;

        let mut filters = VecDeque::new();
        filters.push_front(Box::new(
            |rti: &RoutingTableInner, e: Option<Arc<BucketEntry>>, _cur_ts: Timestamp| {
                let Some(e) = e else {
                    return false;
                };
                let cap_match = e.with(rti, |_rti, e| {
                    e.has_all_capabilities(RoutingDomain::PublicInternet, &capabilities)
                });
                let state = e.with(rti, |_rti, e| e.state(cur_ts));
                state >= min_state && cap_match
            },
        ) as RoutingTableEntryFilter);
        let nodes = self.find_preferred_fastest_nodes(
            node_count,
            filters,
            |_rti, entry: Option<Arc<BucketEntry>>| {
                NodeRef::new(self.registry(), entry.unwrap().clone())
            },
        );
        let mut out = String::new();
        let entry_count = nodes.len();
        for node in nodes {
            let can_be_relay = node.operate(|_rti, e| relay_node_filter(e));
            let is_relay = self
                .relays(RoutingDomain::PublicInternet)
                .iter()
                .find_map(|r| {
                    if r.relay_node.same_entry(&node) {
                        Some(true)
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let is_relaying = node
                .operate(|_rti, e| {
                    e.node_info(RoutingDomain::PublicInternet)
                        .map(|ni| our_node_ids.contains_any_from_slice(&ni.relay_ids()))
                })
                .unwrap_or(false);

            let relay_tag = format!(
                "{}{}",
                if is_relay {
                    "R"
                } else if can_be_relay {
                    "r"
                } else {
                    "-"
                },
                if is_relaying { ">" } else { "-" }
            );
            if can_be_relay {
                relay_count += 1;
            }
            if is_relaying {
                relaying_count += 1;
            }

            let node_id_str = node.to_string();

            out += "    ";
            out += &node.operate(|_rti, e| Self::format_entry(cur_ts, &node_id_str, e, &relay_tag));
            out += "\n";
        }

        out += &format!(
            "Entries: {}\nRelay Capable: {}  Relay Capable %: {:.2}\nRelaying Through This Node: {}\n",
            entry_count,
            relay_count,
            (relay_count as f64) * 100.0 / (entry_count as f64),
            relaying_count,
        );

        out
    }

    pub fn debug_info_entry(&self, node_ref: NodeRef) -> String {
        let cur_ts = Timestamp::now();

        let mut out = String::new();
        out += &node_ref.operate(|_rti, e| {
            let state_reason = e.state_reason(cur_ts);
            format!(
                "{}\nstate: {}\n",
                e,
                Self::format_state_reason(state_reason),
            )
        });
        out
    }

    pub fn debug_info_buckets(&self, min_state: BucketEntryState) -> String {
        let inner = self.inner.read();
        let inner = &*inner;
        let cur_ts = Timestamp::now();

        let mut out = String::new();
        const COLS: usize = 16;
        out += "Buckets:\n";
        for ck in &VALID_CRYPTO_KINDS {
            out += &format!("  {}:\n", ck);
            let rows = inner.buckets[ck].len() / COLS;
            let mut r = 0;
            let mut b = 0;
            while r < rows {
                let mut c = 0;
                out += format!("    {:>3}: ", b).as_str();
                while c < COLS {
                    let mut cnt = 0;
                    for e in inner.buckets[ck][b].entries() {
                        if e.1.with(inner, |_rti, e| e.state(cur_ts) >= min_state) {
                            cnt += 1;
                        }
                    }
                    out += format!("{:>3} ", cnt).as_str();
                    b += 1;
                    c += 1;
                }
                out += "\n";
                r += 1;
            }
        }

        out
    }
}
