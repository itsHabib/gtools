use serde::{Deserialize, Serialize};

/// JSON input format for a graph.
///
/// Expected format:
/// ```json
/// {
///   "nodes": ["api", "auth", "db"],
///   "edges": [
///     { "from": "api", "to": "auth", "latency_ms": 5.2 }
///   ]
/// }
/// ```
#[derive(Debug, Deserialize)]
pub(crate) struct GraphInput {
    /// List of node names
    pub(crate) nodes: Vec<String>,
    /// List of directed edges with latencies
    pub(crate) edges: Vec<EdgeInput>,
}

/// Represents a directed edge in the input graph.
#[derive(Debug, Deserialize, Clone)]
pub(crate) struct EdgeInput {
    /// Source node name
    pub(crate) from: String,
    /// Destination node name
    pub(crate) to: String,
    /// Edge weight/latency in milliseconds
    pub(crate) latency_ms: f32,
}

/// JSON-serializable path output with human-readable node names.
///
/// Suitable for CLI output and API responses.
#[derive(Debug, Serialize)]
pub struct PathOutput {
    /// Source node name
    pub from: String,
    /// Destination node name
    pub to: String,
    /// Sequence of node names from source to destination
    pub path: Vec<String>,
    /// Total latency in milliseconds
    pub total_latency_ms: u32,
    /// Edge with the highest latency (bottleneck)
    pub bottleneck: Option<EdgeOutput>,
}

/// JSON-serializable edge with human-readable node names.
#[derive(Debug, Serialize)]
pub struct EdgeOutput {
    /// Source node name
    pub from: String,
    /// Destination node name
    pub to: String,
    /// Edge latency in milliseconds
    pub latency_ms: u32,
}
