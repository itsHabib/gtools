use std::cmp::min;
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Graph {
    nodes: usize,
    edges: Vec<Edge>,
}

impl Graph {
    pub fn new(nodes: usize) -> Graph {
        Graph{
            nodes,
            edges: Vec::new(),
        }
    }

    pub fn critical_components(&self) -> (Vec<NodeId>, Vec<(NodeId, NodeId)>) {
        let adj = self.adjacency_list();
        let mut disc: Vec<Option<u32>> = vec![None; self.nodes];
        let mut low: Vec<u32> = vec![0; self.nodes];
        let mut parent: Vec<Option<usize>> = vec![None; self.nodes];
        let mut bridges: Vec<(NodeId, NodeId)> = Vec::new();
        let mut points: HashSet<NodeId> = HashSet::new();
        let mut time: u32 = 0;

        fn dfs(
            u: usize,
            adj: &Vec<Vec<NodeId>>,
            parent: &mut Vec<Option<usize>>,
            disc: &mut Vec<Option<u32>>,
            low: &mut Vec<u32>,
            points: &mut HashSet<NodeId>,
            bridges: &mut Vec<(NodeId,NodeId)>,
            time: &mut u32,
        ) {
            disc[u] = Some(*time);
            low[u] = *time;
            *time += 1;

            let mut children: u32 = 0;

            for v in &adj[u] {
                let v_i = v.0 as usize;
                match disc[v_i] {
                    None => {
                        children += 1;
                        parent[v_i] = Some(u);

                        dfs(v_i, adj, parent, disc, low, points, bridges, time);

                        low[u] = min(low[u], low[v_i]);

                        // v or its subtree cant reach u without u-v
                        if low[v_i] > disc[u].expect("disc[u] already initialized above") {
                            bridges.push((NodeId(u as u32), *v))
                        }

                        // u is critical to v connectivity
                        if low[v_i] >= disc[u].expect("disc[u] already initialized above")  && parent[u].is_some() {
                            points.insert(NodeId(u as u32));
                        }
                    }
                    Some(t) => {
                        if Some(v_i) != parent[u] {
                            low[u] = min(low[u], t);
                        }
                    }
                }
            }

            if parent[u].is_none() && children >= 2 {
                points.insert(NodeId(u as u32));
            }
        }

        for n in 0..self.nodes {
            if disc[n].is_some() {
                // already visited
                continue
            }

            dfs(n, &adj, &mut parent, &mut disc, &mut low, &mut points, &mut bridges, &mut time);
        }

        (points.into_iter().collect(), bridges)
    }

    pub fn add_edge(&mut self, edge: Edge) {
        assert!(edge.u.0 < self.nodes as u32 && edge.v.0 < self.nodes as u32, "edge vertices out of bounds");
        self.edges.push(edge);
    }

    pub fn edges(&self) -> Vec<Edge> {
        self.edges.clone()
    }

    pub fn size(&self) -> usize {
        self.nodes
    }

    fn adjacency_list(&self) -> Vec<Vec<NodeId>> {
        let mut adj = vec![Vec::new(); self.nodes];
        for e in &self.edges {
            adj[e.v.0 as usize].push(e.u);
            adj[e.u.0 as usize].push(e.v);
        }

        adj
    }
}


#[derive(Debug, Clone, Copy)]
pub(crate) struct Edge {
    pub(crate) u: NodeId,
    pub(crate) v: NodeId,
    pub(crate) weight: f32,
}

impl PartialEq<Self> for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.u == other.u && self.v == other.v && self.weight == other.weight
    }
}

impl Eq for Edge {}

impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.weight.partial_cmp(&other.weight)
    }
}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight.partial_cmp(&other.weight).unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NodeId(pub(crate) u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_chain() {
        let mut g = Graph::new(3);
        g.add_edge(Edge { u: NodeId(0), v: NodeId(1), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(1), v: NodeId(2), weight: 1.0 });
        
        let (aps, bridges) = g.critical_components();
        assert_eq!(bridges.len(), 2);
        // node 1 is articulation point
        assert_eq!(aps.len(), 1);
    }

    #[test]
    fn test_cycle_no_critical() {
        let mut g = Graph::new(4);
        g.add_edge(Edge { u: NodeId(0), v: NodeId(1), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(1), v: NodeId(2), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(2), v: NodeId(3), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(3), v: NodeId(0), weight: 1.0 });
        
        let (aps, bridges) = g.critical_components();
        assert_eq!(bridges.len(), 0);
        assert_eq!(aps.len(), 0);
    }

    #[test]
    fn test_cycle_with_tail() {
        let mut g = Graph::new(5);
        g.add_edge(Edge { u: NodeId(0), v: NodeId(1), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(1), v: NodeId(2), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(2), v: NodeId(0), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(2), v: NodeId(3), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(3), v: NodeId(4), weight: 1.0 });
        
        let (aps, bridges) = g.critical_components();
        // (2,3) and (3,4)
        assert_eq!(bridges.len(), 2);
        // nodes 2 and 3
        assert_eq!(aps.len(), 2);
    }

    #[test]
    fn test_disconnected() {
        let mut g = Graph::new(6);
        g.add_edge(Edge { u: NodeId(0), v: NodeId(1), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(1), v: NodeId(2), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(3), v: NodeId(4), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(4), v: NodeId(5), weight: 1.0 });
        
        let (aps, bridges) = g.critical_components();
        // all edges
        assert_eq!(bridges.len(), 4);
        // nodes 1 and 4
        assert_eq!(aps.len(), 2);
    }

    #[test]
    fn test_single_edge() {
        let mut g = Graph::new(2);
        g.add_edge(Edge { u: NodeId(0), v: NodeId(1), weight: 1.0 });
        
        let (aps, bridges) = g.critical_components();
        assert_eq!(bridges.len(), 1);
        // no articulation points (can't disconnect with only 2 nodes)
        assert_eq!(aps.len(), 0);
    }

    #[test]
    fn test_root_with_two_children() {
        let mut g = Graph::new(5);
        g.add_edge(Edge { u: NodeId(0), v: NodeId(1), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(1), v: NodeId(2), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(0), v: NodeId(3), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(3), v: NodeId(4), weight: 1.0 });
        
        let (aps, bridges) = g.critical_components();
        // all edges are bridges
        assert_eq!(bridges.len(), 4);
        // nodes 0, 1, and 3
        assert_eq!(aps.len(), 3);
    }

    #[test]
    fn test_no_edges() {
        let g = Graph::new(3);
        let (aps, bridges) = g.critical_components();
        assert_eq!(bridges.len(), 0);
        assert_eq!(aps.len(), 0);
    }
}
