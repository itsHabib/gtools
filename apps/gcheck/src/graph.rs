use crate::error::{GraphBuildError, PathError};
use crate::io::GraphInput;
use crate::path::{Edge, Path};
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Internal node identifier
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub(crate) struct NodeId(pub u32);

/// A directed weighted graph optimized for shortest path queries.
/// The graph stores nodes as string names with integer-based internal
/// representation. Edges are stored in adjacency lists with latency weights
/// in milliseconds (as u32).
#[derive(Clone)]
pub(crate) struct Graph {
    /// Maps NodeId to node name
    pub(crate) to_name: Vec<String>,
    /// Maps node name to NodeId
    pub(crate) to_id: HashMap<String, NodeId>,
    /// Adjacency list: for each node, stores (neighbor, weight_ms) pairs
    pub(crate) adj: Vec<Vec<(NodeId, u32)>>,
}

impl Graph {
    /// Loads a graph from a JSON file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON file containing graph data
    ///
    /// # Returns
    ///
    /// * `Ok(Graph)` - Successfully loaded and validated graph
    /// * `Err` - If file cannot be read, JSON is invalid, or graph validation fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let graph = Graph::load_json("graph.json")?;
    /// ```
    pub fn load_json(path: &str) -> anyhow::Result<Graph> {
        use anyhow::Context;

        let contents =
            std::fs::read_to_string(path).context(format!("Failed to read file: {}", path))?;

        let input: GraphInput = serde_json::from_str(&contents).context("Failed to parse JSON")?;

        let graph = Graph::try_from(input).context("Failed to build graph from input")?;

        Ok(graph)
    }

    /// Finds the shortest path between two nodes using Dijkstra's algorithm.
    ///
    /// # Arguments
    ///
    /// * `from` - Source node name
    /// * `to` - Destination node name
    ///
    /// # Returns
    ///
    /// * `Ok(Path)` - The shortest path with cost and node sequence
    /// * `Err(PathError::NodeNotFound)` - If either node doesn't exist
    /// * `Err(PathError::PathNotFound)` - If no path exists between the nodes
    ///
    /// # Example
    ///
    /// ```ignore
    /// let path = graph.shortest_path("api", "db")?;
    /// println!("Cost: {}, Path: {:?}", path.cost, path.path);
    /// ```
    pub fn shortest_path(&self, from: &str, to: &str) -> Result<Path, PathError> {
        let from_id = self
            .to_id
            .get(from)
            .ok_or_else(|| PathError::NodeNotFound(from.to_string()))?;
        let to_id = self
            .to_id
            .get(to)
            .ok_or_else(|| PathError::NodeNotFound(to.to_string()))?;

        let n = self.to_name.len();
        let mut distances = vec![u32::MAX; n];
        let mut parents: Vec<Option<NodeId>> = vec![None; n];
        distances[from_id.0 as usize] = 0;

        let mut h = BinaryHeap::new();
        h.push(Reverse(State {
            cost: 0,
            node: *from_id,
        }));

        while let Some(Reverse(State { cost, node })) = h.pop() {
            if node == *to_id {
                let path = self.path(*to_id, &parents);
                let cost = distances[node.0 as usize];
                let bottleneck = self.bottleneck(&path);

                return Ok(Path {
                    from: *from_id,
                    to: *to_id,
                    path,
                    cost,
                    bottleneck,
                });
            }

            if cost > distances[node.0 as usize] {
                continue;
            }

            for (neighbor, weight) in &self.adj[node.0 as usize] {
                let new_cost = cost + weight;

                if new_cost < distances[neighbor.0 as usize] {
                    distances[neighbor.0 as usize] = new_cost;
                    parents[neighbor.0 as usize] = Some(node);

                    h.push(Reverse(State {
                        cost: new_cost,
                        node: neighbor.clone(),
                    }));
                }
            }
        }

        Err(PathError::PathNotFound {
            from: from.to_string(),
            to: to.to_string(),
        })
    }

    /// Reconstructs the path from source to destination by walking backwards through parents.
    ///
    /// # Arguments
    ///
    /// * `start` - The destination NodeId
    /// * `parents` - Parent tracking array from Dijkstra's algorithm
    ///
    /// # Returns
    ///
    /// A vector of NodeIds representing the path from source to destination
    fn path(&self, start: NodeId, parents: &Vec<Option<NodeId>>) -> Vec<NodeId> {
        let mut cur = Some(start);
        let mut path = Vec::new();

        while let Some(n) = cur {
            path.push(n);
            cur = parents[n.0 as usize];
        }

        path.reverse();

        path
    }

    /// Identifies the bottleneck edge (highest latency) on a given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Sequence of nodes representing a path through the graph
    ///
    /// # Returns
    ///
    /// * `Some(Edge)` - The edge with maximum latency on the path
    /// * `None` - If the path has fewer than 2 nodes (no edges)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // For path api → auth → db with edges (5ms, 3ms)
    /// // Returns Edge { from: "api", to: "auth", latency_ms: 5 }
    /// ```
    fn bottleneck(&self, path: &Vec<NodeId>) -> Option<Edge> {
        let mut max: u32 = 0;
        let mut e = None;

        for i in 0..path.len() - 1 {
            let from = path[i];
            let to = path[i + 1];

            for (neighbor, weight) in &self.adj[from.0 as usize] {
                if neighbor.0 == to.0 && *weight > max {
                    max = *weight;
                    e = Some(Edge {
                        from,
                        to,
                        latency_ms: max,
                    });
                    break;
                }
            }
        }

        e
    }

    /// Formats a path as a human-readable string with arrow separators.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to format
    ///
    /// # Returns
    ///
    /// A string like "api → auth → db"
    ///
    /// # Example
    ///
    /// ```ignore
    /// let path = graph.shortest_path("api", "db")?;
    /// println!("{}", graph.format_path(&path));
    /// // Output: "api → auth → db"
    /// ```
    pub fn format_path(&self, path: &Path) -> String {
        path.path
            .iter()
            .map(|node_id| self.to_name[node_id.0 as usize].as_str())
            .collect::<Vec<_>>()
            .join(" → ")
    }

    /// Apply modifications to create a simulation graph.
    /// Returns a new Graph with modified/dropped edges.
    ///
    /// # Arguments
    ///
    /// * `overrides` - Edges to modify with new weights: (from, to, new_weight)
    /// * `drop` - Edges to remove: (from, to)
    ///
    /// # Returns
    ///
    /// * `Ok(Graph)` - Modified graph with changes applied
    /// * `Err(PathError::NodeNotFound)` - If any node in overrides/drops doesn't exist
    ///
    /// # Example
    ///
    /// ```ignore
    /// let modified = graph.with_modifications(
    ///     &[("auth".to_string(), "db".to_string(), 200)],
    ///     &[("api".to_string(), "cache".to_string())]
    /// )?;
    /// ```
    pub fn with_modifications(
        &self,
        overrides: &[(String, String, u32)],
        drop: &[(String, String)],
    ) -> Result<Graph, PathError> {
        let mut modified = self.clone();

        // apply drops
        for (from_name, to_name) in drop {
            let from_id = self
                .to_id
                .get(from_name)
                .ok_or_else(|| PathError::NodeNotFound(from_name.clone()))?;
            let to_id = self
                .to_id
                .get(to_name)
                .ok_or_else(|| PathError::NodeNotFound(to_name.clone()))?;

            modified.adj[from_id.0 as usize].retain(|(neighbor, _)| neighbor.0 != to_id.0);
        }

        // apply weight overrides
        for (from_name, to_name, new_weight) in overrides {
            let from_id = self
                .to_id
                .get(from_name)
                .ok_or_else(|| PathError::NodeNotFound(from_name.clone()))?;
            let to_id = self
                .to_id
                .get(to_name)
                .ok_or_else(|| PathError::NodeNotFound(to_name.clone()))?;

            let adj_list = &mut modified.adj[from_id.0 as usize];
            if let Some(edge) = adj_list
                .iter_mut()
                .find(|(neighbor, _)| neighbor.0 == to_id.0)
            {
                edge.1 = *new_weight;
            }
        }

        Ok(modified)
    }

    /// Converts an internal Path to PathOutput with human-readable node names.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to convert
    ///
    /// # Returns
    ///
    /// PathOutput with node names instead of NodeIds, suitable for JSON serialization
    ///
    /// # Example
    ///
    /// ```ignore
    /// let path = graph.shortest_path("api", "db")?;
    /// let output = graph.path_output(&path);
    /// println!("{}", serde_json::to_string_pretty(&output)?);
    /// ```
    pub fn path_output(&self, path: &Path) -> crate::io::PathOutput {
        use crate::io::{EdgeOutput, PathOutput};

        PathOutput {
            from: self.to_name[path.from.0 as usize].clone(),
            to: self.to_name[path.to.0 as usize].clone(),
            path: path
                .path
                .iter()
                .map(|id| self.to_name[id.0 as usize].clone())
                .collect(),
            total_latency_ms: path.cost,
            bottleneck: path.bottleneck.as_ref().map(|b| EdgeOutput {
                from: self.to_name[b.from.0 as usize].clone(),
                to: self.to_name[b.to.0 as usize].clone(),
                latency_ms: b.latency_ms,
            }),
        }
    }
}

impl TryFrom<GraphInput> for Graph {
    type Error = GraphBuildError;
    fn try_from(src: GraphInput) -> Result<Self, Self::Error> {
        let mut nodes: HashSet<String> = HashSet::new();
        let mut to_name: Vec<String> = Vec::new();
        let mut to_id: HashMap<String, NodeId> = HashMap::new();

        for n in src.nodes.iter() {
            if nodes.contains(n) {
                return Err(GraphBuildError::DuplicateNode(n.to_string()));
            }

            nodes.insert(n.to_string());
            to_name.push(n.to_string());
            to_id.insert(n.clone(), NodeId((to_name.len() - 1) as u32));
        }

        let mut adj: Vec<Vec<(NodeId, u32)>> = vec![Vec::new(); nodes.len()];
        for edge in src.edges.into_iter() {
            if !nodes.contains(&edge.from) {
                return Err(GraphBuildError::UnknownFrom(edge.from));
            }

            if !nodes.contains(&edge.to) {
                return Err(GraphBuildError::UnknownTo(edge.to));
            }

            if edge.latency_ms < 0.0 {
                return Err(GraphBuildError::NegativeLatency {
                    from: edge.from,
                    to: edge.to,
                    latency_ms: edge.latency_ms,
                });
            }

            if edge.from == edge.to {
                return Err(GraphBuildError::SelfLoop { node: edge.from });
            }

            let from = to_id
                .get(&edge.from)
                .expect("from node must exist: validated above");
            let to = to_id
                .get(&edge.to)
                .expect("to node must exist: validated above");

            adj[from.0 as usize].push((to.clone(), edge.latency_ms as u32));
        }

        Ok(Graph {
            adj,
            to_name,
            to_id,
        })
    }
}

/// Priority queue state for Dijkstra's algorithm.
///
/// Wraps a node and its current best known distance from the source.
/// Used with `Reverse` to create a min-heap from BinaryHeap's max-heap.
#[derive(PartialEq, Eq, Debug)]
struct State {
    node: NodeId,
    cost: u32,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EdgeInput, GraphInput};

    fn create_test_graph() -> Graph {
        let input = GraphInput {
            nodes: vec!["api".to_string(), "auth".to_string(), "db".to_string()],
            edges: vec![
                EdgeInput {
                    from: "api".to_string(),
                    to: "auth".to_string(),
                    latency_ms: 5.2,
                },
                EdgeInput {
                    from: "auth".to_string(),
                    to: "db".to_string(),
                    latency_ms: 3.1,
                },
            ],
        };
        Graph::try_from(input).unwrap()
    }

    #[test]
    fn test_shortest_path_simple() {
        let graph = create_test_graph();
        let path = graph.shortest_path("api", "db").unwrap();

        assert_eq!(path.cost, 8);
        assert_eq!(path.path.len(), 3);
        assert_eq!(graph.format_path(&path), "api → auth → db");
    }

    #[test]
    fn test_node_not_found() {
        let graph = create_test_graph();
        let result = graph.shortest_path("api", "nonexistent");

        assert!(result.is_err());
        match result {
            Err(PathError::NodeNotFound(node)) => {
                assert_eq!(node, "nonexistent");
            }
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_path_not_found() {
        let input = GraphInput {
            nodes: vec!["a".to_string(), "b".to_string()],
            edges: vec![],
        };
        let graph = Graph::try_from(input).unwrap();

        let result = graph.shortest_path("a", "b");
        assert!(result.is_err());
        match result {
            Err(PathError::PathNotFound { from, to }) => {
                assert_eq!(from, "a");
                assert_eq!(to, "b");
            }
            _ => panic!("Expected PathNotFound error"),
        }
    }

    #[test]
    fn test_bottleneck_identification() {
        let graph = create_test_graph();
        let path = graph.shortest_path("api", "db").unwrap();

        assert!(path.bottleneck.is_some());
        let bottleneck = path.bottleneck.unwrap();

        let from_name = &graph.to_name[bottleneck.from.0 as usize];
        let to_name = &graph.to_name[bottleneck.to.0 as usize];

        assert_eq!(from_name, "api");
        assert_eq!(to_name, "auth");
        assert_eq!(bottleneck.latency_ms, 5);
    }

    #[test]
    fn test_bottleneck_with_larger_graph() {
        let input = GraphInput {
            nodes: vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ],
            edges: vec![
                EdgeInput {
                    from: "a".to_string(),
                    to: "b".to_string(),
                    latency_ms: 2.0,
                },
                EdgeInput {
                    from: "b".to_string(),
                    to: "c".to_string(),
                    latency_ms: 10.0,
                },
                EdgeInput {
                    from: "c".to_string(),
                    to: "d".to_string(),
                    latency_ms: 3.0,
                },
            ],
        };
        let graph = Graph::try_from(input).unwrap();
        let path = graph.shortest_path("a", "d").unwrap();

        assert!(path.bottleneck.is_some());
        let bottleneck = path.bottleneck.unwrap();

        let from_name = &graph.to_name[bottleneck.from.0 as usize];
        let to_name = &graph.to_name[bottleneck.to.0 as usize];

        assert_eq!(from_name, "b");
        assert_eq!(to_name, "c");
        assert_eq!(bottleneck.latency_ms, 10);
    }

    #[test]
    fn test_load_json_from_embedded_data() {
        let json = include_str!("testdata/simple_graph.json");
        let input: GraphInput = serde_json::from_str(json).unwrap();
        let graph = Graph::try_from(input).unwrap();

        assert_eq!(graph.to_name.len(), 3);
        assert!(graph.to_id.contains_key("a"));
        assert!(graph.to_id.contains_key("b"));
        assert!(graph.to_id.contains_key("c"));
    }

    #[test]
    fn test_load_json_file() {
        let graph = Graph::load_json("src/testdata/sample_graph.json").unwrap();

        assert_eq!(graph.to_name.len(), 4);
        assert!(graph.to_id.contains_key("api"));
        assert!(graph.to_id.contains_key("auth"));
        assert!(graph.to_id.contains_key("db"));
        assert!(graph.to_id.contains_key("cache"));

        let path = graph.shortest_path("api", "db").unwrap();
        assert!(path.cost > 0);
    }

    #[test]
    fn test_load_json_invalid_graph() {
        let result = Graph::load_json("src/testdata/invalid_graph.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_json_nonexistent_file() {
        let result = Graph::load_json("nonexistent_file.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_self_loop_detection() {
        let input = GraphInput {
            nodes: vec!["a".to_string(), "b".to_string()],
            edges: vec![EdgeInput {
                from: "a".to_string(),
                to: "a".to_string(), // Self-loop!
                latency_ms: 5.0,
            }],
        };
        let result = Graph::try_from(input);
        assert!(result.is_err());
        match result {
            Err(GraphBuildError::SelfLoop { node }) => {
                assert_eq!(node, "a");
            }
            _ => panic!("Expected SelfLoop error"),
        }
    }

    #[test]
    fn test_with_modifications_override() {
        let graph = create_test_graph();

        let original_path = graph.shortest_path("api", "db").unwrap();
        assert_eq!(original_path.cost, 8);
        assert_eq!(graph.format_path(&original_path), "api → auth → db");

        let modified = graph
            .with_modifications(&[("auth".to_string(), "db".to_string(), 100)], &[])
            .unwrap();

        let new_path = modified.shortest_path("api", "db").unwrap();
        assert_eq!(new_path.cost, 105); // api→auth (5) + auth→db (100)
    }

    #[test]
    fn test_with_modifications_drop() {
        let graph = Graph::load_json("src/testdata/sample_graph.json").unwrap();

        // Original shortest path should be api → auth → db
        let original_path = graph.shortest_path("api", "db").unwrap();
        assert_eq!(graph.format_path(&original_path), "api → auth → db");

        // Drop auth→db edge
        let modified = graph
            .with_modifications(&[], &[("auth".to_string(), "db".to_string())])
            .unwrap();

        // Path should change to go through cache
        let new_path = modified.shortest_path("api", "db").unwrap();
        assert_eq!(graph.format_path(&new_path), "api → cache → db");
    }

    #[test]
    fn test_with_modifications_combined() {
        let graph = Graph::load_json("src/testdata/sample_graph.json").unwrap();

        let modified = graph
            .with_modifications(
                &[("api".to_string(), "cache".to_string(), 1)], // Make cache path faster
                &[("auth".to_string(), "db".to_string())],      // Drop auth→db
            )
            .unwrap();

        let new_path = modified.shortest_path("api", "db").unwrap();
        assert_eq!(graph.format_path(&new_path), "api → cache → db");
        assert!(new_path.cost < 5); // Should be much faster now
    }

    #[test]
    fn test_with_modifications_invalid_node() {
        let graph = create_test_graph();

        // Try to override edge with non-existent node
        let result =
            graph.with_modifications(&[("api".to_string(), "nonexistent".to_string(), 100)], &[]);

        assert!(result.is_err());
        match result {
            Err(PathError::NodeNotFound(node)) => {
                assert_eq!(node, "nonexistent");
            }
            _ => panic!("Expected NodeNotFound error"),
        }
    }
}
