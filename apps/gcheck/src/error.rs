/// Errors that can occur when building a graph from input.
#[derive(thiserror::Error, Debug)]
pub enum GraphBuildError {
    /// A node name appears more than once in the node list
    #[error("duplicate node name: {0}")]
    DuplicateNode(String),
    /// An edge references a non-existent source node
    #[error("unknown node in edge 'from': {0}")]
    UnknownFrom(String),
    /// An edge references a non-existent destination node
    #[error("unknown node in edge 'to': {0}")]
    UnknownTo(String),
    /// An edge has a negative latency value
    #[error("negative latency on edge {from}->{to}: {latency_ms}")]
    NegativeLatency {
        from: String,
        to: String,
        latency_ms: f32,
    },
    /// A self-loop was detected (node pointing to itself)
    #[error("self loop detected on node {node}")]
    SelfLoop { node: String },
}

/// Errors that can occur when finding a path through the graph.
#[derive(thiserror::Error, Debug)]
pub enum PathError {
    /// The specified node does not exist in the graph
    #[error("node not found: {0}")]
    NodeNotFound(String),
    /// No path exists between the source and destination nodes
    #[error("path not found {from}->{to}")]
    PathNotFound { from: String, to: String },
}
