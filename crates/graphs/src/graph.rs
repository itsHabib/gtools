#[derive(Debug, Clone)]
pub(crate) struct Graph {
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
