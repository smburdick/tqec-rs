use std::collections::HashMap;
use rust_3d::Point3D;
use petgraph::graph::UnGraph;
use crate::cube::Cube;

pub struct BlockGraph {
    name: String,
    graph: UnGraph<Cube, ()>,
    ports: HashMap<String, Point3D>
}

impl BlockGraph {

    pub fn new(name: &str) -> Self {
        let graph: UnGraph<Cube, ()> = UnGraph::default();
        let ports: HashMap<String, Point3D> = HashMap::new();
        Self {
            name: name.to_string(),
            graph: graph,
            ports: ports
        }
    }

    pub fn num_cubes(&self) -> usize {
        return self.graph.node_count();
    }

    pub fn num_pipes(&self) -> usize {
        return self.graph.edge_count();
    }

    pub fn num_ports(&self) -> usize {
        return self.ports.len();
    }

    pub fn num_y_half_cubes(&self) -> usize {
        return 0; // TODO:
    }

    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn is_open(&self) -> bool {
        return self.num_ports() > 0;
    }

    pub fn spacetime_volume(&self) -> f64 {
        return 0.0; // TODO: need YHalfCube
    }

}


