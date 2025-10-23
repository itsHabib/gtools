use crate::dsu::DisjointSet;
use crate::graph::{Edge, Graph};

pub struct Mst {
    pub(crate) edges: Vec<Edge>,
    pub(crate) total_weight: f32,
}

pub fn kruskal(g: &Graph) -> Mst {
    let mut edges = g.edges();
    let n = g.size();
    let mut ds = DisjointSet::new(n);

    edges.sort();
    let mut span = Vec::new();
    let mut total_weight = 0.0;
    for e in edges {
        if ds.union(e.u.0 as usize, e.v.0 as usize) {
            span.push(e);
            total_weight += e.weight;
        }
    }

    Mst{
        edges: span,
        total_weight,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, NodeId};

    #[test]
    fn test_triangle() {
        let mut g = Graph::new(3);
        g.add_edge(Edge { u: NodeId(0), v: NodeId(1), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(1), v: NodeId(2), weight: 2.0 });
        g.add_edge(Edge { u: NodeId(2), v: NodeId(0), weight: 3.0 });

        let mst = kruskal(&g);
        assert_eq!(mst.total_weight, 3.0);
        assert_eq!(mst.edges.len(), 2);
    }

    #[test]
    fn test_disconnected() {
        let mut g = Graph::new(4);
        g.add_edge(Edge { u: NodeId(0), v: NodeId(1), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(2), v: NodeId(3), weight: 2.0 });

        let mst = kruskal(&g);
        assert_eq!(mst.total_weight, 3.0);
        assert_eq!(mst.edges.len(), 2);
    }

    #[test]
    fn test_square_with_diagonal() {
        let mut g = Graph::new(4);
        g.add_edge(Edge { u: NodeId(0), v: NodeId(1), weight: 1.0 });
        g.add_edge(Edge { u: NodeId(1), v: NodeId(2), weight: 2.0 });
        g.add_edge(Edge { u: NodeId(2), v: NodeId(3), weight: 3.0 });
        g.add_edge(Edge { u: NodeId(3), v: NodeId(0), weight: 4.0 });
        g.add_edge(Edge { u: NodeId(0), v: NodeId(2), weight: 5.0 });

        let mst = kruskal(&g);
        assert_eq!(mst.total_weight, 6.0);
        assert_eq!(mst.edges.len(), 3);
    }
}