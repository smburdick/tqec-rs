use std::collections::HashMap;
use petgraph::graph::{NodeIndex, UnGraph};
use dae_parser::*;

use crate::cube::{Cube, CubeKind};

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct CubePosition {
    x: i32,
    y: i32,
    z: i32
}

impl CubePosition {
    pub fn new(x: i32, y: i32, z: i32) -> CubePosition {
        Self {
            x: x, y: y, z: z
        }
    }
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

    pub fn from_dae_file(filepath: &str) -> Result<Self, Error> {
        let contents: Result<Document, Error> = Document::from_file(filepath);
        match contents {
            Ok(val) => {
                // let doc = contents.unwrap();
                Ok(BlockGraph::new("Example"))
            },
            Err(e) => Err(e),
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
        self.cube_data.values().filter(|cube| cube.is_y_half_cube()).count()
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
        ((self.num_cubes() - self.num_ports() - self.num_y_half_cubes()) as f64)
            / 2.0
    }

    pub fn add_cube(&mut self, pos: CubePosition, kind: CubeKind, label: String) {
        let idx: NodeIndex = self.graph.add_node(pos.clone());
        self.node_indices.insert(pos.clone(), idx);
        let cube: Cube = Cube::new(kind, label.clone());
        self.cube_data.insert(pos.clone(), cube);
    }

    pub fn degree(&self, pos: CubePosition) -> usize {
        let idx: Option<&NodeIndex> = self.node_indices.get(&pos);
        match idx {
            Some(val) => self.graph.neighbors(*val).count(),
            None => 0
        }
    }

    pub fn leaves(&self) -> Vec<CubePosition> {
        self.cube_data.keys()
            .cloned()
            .filter(|pos| self.degree(*pos) == 1)
            .collect()
    }

}


