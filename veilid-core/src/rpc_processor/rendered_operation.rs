use super::*;

/// An operation that has been fully prepared for envelope
pub struct RenderedOperation {
    /// The rendered operation id for logging purposes,
    /// which may be different from the message's op_id
    /// if it wrapped with a route
    pub outer_op_id: OperationId,
    /// The rendered signed operation bytes
    pub message: Vec<u8>,
    /// Destination node we're sending to
    pub destination_node_ref: NodeRef,
    /// Node to send envelope to (may not be destination node in case of relay)
    pub node_ref: FilteredNodeRef,
    /// The safety route used to send the message
    pub safety_route: Option<PublicKey>,
    /// The private route used to send the message
    pub remote_private_route: Option<PublicKey>,
    /// The private route requested to receive the reply
    pub reply_private_route: Option<PublicKey>,
}

impl fmt::Debug for RenderedOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderedOperation")
            .field("outer_op_id", &self.outer_op_id)
            .field("message(len)", &self.message.len())
            .field("destination_node_ref", &self.destination_node_ref)
            .field("node_ref", &self.node_ref)
            .field("safety_route", &self.safety_route)
            .field("remote_private_route", &self.remote_private_route)
            .field("reply_private_route", &self.reply_private_route)
            .finish()
    }
}
