use std::collections::HashMap;
use rust_3d::Point3D;
use petgraph::graph::UnGraph;

pub struct BlockGraph {
    name: String,
    graph: UnGraph<Point3D, ()>, // TODO: replace Point3D with Cube everywhere
    ports: HashMap<String, Point3D>
}

impl BlockGraph {

    pub fn new(name: &str) -> Self {
        let graph: UnGraph<Point3D, ()> = UnGraph::default();
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

    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }
}


