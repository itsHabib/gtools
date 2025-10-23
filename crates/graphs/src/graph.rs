use std::cmp::min;

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

    pub fn bridges(&self) -> Vec<(NodeId, NodeId)> {
        let adj = self.adjacency_list();
        let mut disc: Vec<Option<u32>> = vec![None; self.nodes];
        let mut low: Vec<u32> = vec![0; self.nodes];
        let mut parent: Vec<Option<usize>> = vec![None; self.nodes];
        let mut bridges: Vec<(NodeId, NodeId)> = Vec::new();
        let mut time: u32 = 0;

        fn dfs(
            u: usize,
            adj: &Vec<Vec<NodeId>>,
            parent: &mut Vec<Option<usize>>,
            disc: &mut Vec<Option<u32>>,
            low: &mut Vec<u32>,
            bridges: &mut Vec<(NodeId,NodeId)>,
            time: &mut u32,
        ) {
            disc[u] = Some(*time);
            low[u] = *time;
            *time += 1;

            for v in &adj[u] {
                let v_i = v.0 as usize;
                match disc[v_i] {
                    None => {
                        parent[v_i] = Some(u);

                        dfs(v_i, adj, parent, disc, low, bridges, time);

                        low[u] = min(low[u], low[v_i]);

                        // v or its subtree cant reach u without u-v
                        if low[v_i] > disc[u].expect("disc[u] already initialized above") {
                            bridges.push((NodeId(u as u32), *v))
                        }
                    }
                    Some(t) => {
                        if Some(v_i) != parent[u] {
                            low[u] = min(low[u], t);
                        }
                    }
                }
            }
        }

        for n in 0..self.nodes {
            if disc[n].is_some() {
                // already visited
                continue
            }

            dfs(n, &adj, &mut parent, &mut disc, &mut low, &mut bridges, &mut time);
        }

        bridges
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
