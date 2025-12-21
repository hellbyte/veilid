use super::*;

/// Transaction id and node id pair
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeTransactionId {
    node_id: NodeId,
    xid: u64,
}

impl NodeTransactionId {
    pub fn new(node_id: NodeId, xid: u64) -> Self {
        Self { node_id, xid }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id.clone()
    }

    pub fn xid(&self) -> u64 {
        self.xid
    }
}

impl fmt::Display for NodeTransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:xid={}", self.node_id, self.xid)
    }
}
