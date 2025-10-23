use crate::graph::Edge;

pub(crate) struct Mst {
    pub(crate) edges: Vec<Edge>,
    pub(crate) total_weight: f32,
}