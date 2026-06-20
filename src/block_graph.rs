use std::{collections::HashMap, string};
use petgraph::graph::{NodeIndex, UnGraph};

use crate::cube::{Cube, CubeKind};

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct CubePosition {
    x: i32,
    y: i32,
    z: i32
}

pub struct BlockGraph {
    name: String,
    graph: UnGraph<CubePosition, ()>,
    node_indices: HashMap<CubePosition, NodeIndex>,
    cube_data: HashMap<CubePosition, Cube>,
    ports: HashMap<String, CubePosition>
}

impl BlockGraph {

    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            graph: UnGraph::default(),
            cube_data: HashMap::new(),
            node_indices: HashMap::new(),
            ports: HashMap::new()
        }
    }

    pub fn num_cubes(&self) -> usize {
        self.graph.node_count()
    }

    pub fn num_pipes(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn num_ports(&self) -> usize {
        self.ports.len()
    }

    pub fn num_y_half_cubes(&self) -> usize {
        0 // TODO:
    }

    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn is_open(&self) -> bool {
        self.num_ports() > 0
    }

    pub fn spacetime_volume(&self) -> f64 {
        0.0 // TODO: need YHalfCube
    }

    pub fn add_cube(&mut self, pos: CubePosition, kind: CubeKind, label: String) {
        // add position to the graph
        let idx= self.graph.add_node(pos.clone());
        self.node_indices.insert(pos, idx);
        // TODO: update cube metadata corresponding to position
    }

    pub fn degree(&self, pos: CubePosition) -> usize {
        let idx: Option<&NodeIndex> = self.node_indices.get(&pos);
        match idx {
            Some(val) => self.graph.neighbors(*val).count(),
            None => 0
        }
    }

    pub fn leaves(&self) -> Vec<CubePosition> {
        let leaves: Vec<CubePosition> = Vec::new();
        // self.graph.
        leaves
    }

}


