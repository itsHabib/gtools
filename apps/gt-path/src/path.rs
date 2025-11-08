use crate::graph::NodeId;

/// Represents a path through the graph with its total cost.
///
/// Returned by `Graph::shortest_path()` to indicate the sequence of nodes
/// and the total latency in milliseconds.
pub(crate) struct Path {
    /// Source node
    pub(crate) from: NodeId,
    /// Destination node
    pub(crate) to: NodeId,
    /// Sequence of nodes from source to destination
    pub(crate) path: Vec<NodeId>,
    /// Total latency in milliseconds
    pub(crate) cost: u32,
    /// Edge with the highest latency along the path
    pub(crate) bottleneck: Option<Edge>,
}

/// Represents a directed edge in the graph with its latency.
pub(crate) struct Edge {
    /// Source node
    pub(crate) from: NodeId,
    /// Destination node
    pub(crate) to: NodeId,
    /// Edge latency/weight in milliseconds
    pub(crate) latency_ms: u32,
}
