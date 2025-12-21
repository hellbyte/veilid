use super::*;

#[derive(Debug)]
#[must_use]
pub(super) struct WaitableReplyContext {
    pub timeout: TimestampDuration,
    pub send_ts: Timestamp,
    pub send_data_result: SendDataResult,
    pub node_ref: NodeRef,
    pub safety_route: Option<PublicKey>,
    pub remote_private_route: Option<PublicKey>,
    pub reply_private_route: Option<PublicKey>,
}

impl WaitableReplyContext {
    #[cfg_attr(not(feature = "verbose-tracing"), expect(dead_code))]
    pub fn debug(&self, routing_table: &RoutingTable) -> String {
        let rss = routing_table.route_spec_store();

        let opt_srstr = self
            .safety_route
            .as_ref()
            .map(|key| rss.display_route_by_key(key));
        let opt_remprstr = self
            .remote_private_route
            .as_ref()
            .map(|key| rss.display_route_by_key(key));
        let opt_repprstr = if self.reply_private_route != self.safety_route {
            self.reply_private_route
                .as_ref()
                .map(|key| rss.display_route_by_key(key))
        } else {
            None
        };

        format!(
            "timeout={} send_ts={} send_data_result={} node={}{}{}{}",
            self.timeout,
            self.send_ts,
            self.send_data_result,
            self.node_ref,
            if let Some(srstr) = opt_srstr {
                format!("\nsafety_route={}", srstr)
            } else {
                "".to_string()
            },
            if let Some(remprstr) = opt_remprstr {
                format!("\nremote_private_route={}", remprstr)
            } else {
                "".to_string()
            },
            if let Some(repprstr) = opt_repprstr {
                format!("\nreply_private_route={}", repprstr)
            } else {
                "".to_string()
            },
        )
    }
}

#[derive(Debug)]
#[must_use]
pub(super) struct WaitableReply {
    pub handle: OperationWaitHandle<Message, Option<Arc<QuestionContext>>>,
    _opt_connection_ref_scope: Option<ConnectionRefScope>,
    pub context: WaitableReplyContext,
}

impl WaitableReply {
    pub fn new(
        handle: OperationWaitHandle<Message, Option<Arc<QuestionContext>>>,
        opt_connection_ref_scope: Option<ConnectionRefScope>,
        context: WaitableReplyContext,
    ) -> Self {
        Self {
            handle,
            _opt_connection_ref_scope: opt_connection_ref_scope,
            context,
        }
    }
}
