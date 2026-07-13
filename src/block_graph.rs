use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, path::Path, str::FromStr};
use petgraph::{Graph, Undirected, graph::{EdgeIndex, NodeIndex, UnGraph}};

use crate::cube::{Cube, CubeKind, CubePosition, Pipe, ZXCube};
use crate::positioned::PositionedZX;

pub struct BlockGraph {
    name: String,
    graph: Graph<Cube, Pipe, Undirected>,
    node_indices: HashMap<CubePosition, NodeIndex>, // Used for client lookups
    edge_indices: HashMap<Pipe, EdgeIndex>,
    // cube_data: HashMap<CubePosition, Cube>,
    ports: HashMap<String, CubePosition>
}

impl BlockGraph {

    pub fn new(name: String) -> Self {
        Self {
            name: name,
            graph: UnGraph::default(),
            // cube_data: HashMap::new(),
            node_indices: HashMap::new(),
            edge_indices: HashMap::new(),
            ports: HashMap::new()
        }
    }

    pub fn from_bgraph_file(filepath: &str) -> Result<Self, String> {
        // Based on https://tqec.github.io/tqec/user_guide/bgraph.html
        let path = Path::new(filepath);
        let file = File::open(&path);
        let mut to_return = Self::new(format!("block_graph[{}]", filepath));
        match file {
            Ok(goodfile) => {
                let reader = BufReader::new(goodfile);
                let mut parse_cubes = false;
                let mut parse_pipes = false;
                let mut cubeIdToNodeIndex: HashMap<String, NodeIndex> = HashMap::new();
                for (index, line) in reader.lines().enumerate() {
                    // TODO: skip header and metadata
                    let _line = line.unwrap();
                    if _line.len() == 1 || _line.is_empty() {
                        continue;
                    }
                    if _line.starts_with("CUBE") {
                        parse_cubes = true; // start parsing cubes
                        continue;
                    } else if _line.starts_with("PIPE") {
                        parse_pipes = true; // start parsing pipes
                        parse_cubes = false;
                        continue;
                    }
                    if parse_cubes {
                        let items: Vec<&str> = _line.split(";").collect();
                        // if items.len() != 6 {
                        //     Err(String::from("Incorrect cube spec"))
                        // }
                        let cube_id: &str = items[0];
                        // TODO: when cube is added to the graph, map its cube
                        // id to its NodeIndex, then use that to link up the pipes
                        let x_coord: i32 = items[1].parse().unwrap();
                        let y_coord: i32 = items[2].parse().unwrap();
                        let z_coord: i32 = items[3].parse().unwrap();
                        let kind: String = items[4].to_uppercase();
                        // FIXME: need to generate the correct kind of cube here.
                        // ZXCube, YHalfCube, Port
                        let zx_cube_kind = CubeKind::ZX(ZXCube::from_str(&kind)?);
                        let pos: CubePosition = CubePosition::new(x_coord, y_coord, z_coord);
                        let annotation: &str = items[5]; // TODO: how is this used?
                        let cube: Cube = Cube::new(zx_cube_kind, pos);
                        let idx = to_return.graph.add_node(cube);
                        to_return.node_indices.insert(pos, idx);
                        // to_return.cube_data.insert(pos, zx_cube);
                        cubeIdToNodeIndex.insert(cube_id.to_string(), idx);
                    } else if parse_pipes {
                        let items: Vec<&str> = _line.split(";").collect();
                        let cube1_id: &str = items[0];
                        let cube2_id: &str = items[1];
                        let kind = items[2];
                        let cube1_idx = cubeIdToNodeIndex.get(cube1_id).unwrap();
                        let cube2_idx = cubeIdToNodeIndex.get(cube2_id).unwrap();
                        let weight: Pipe = Pipe::from_str(kind)?;
                        let eidx = to_return.graph.add_edge(*cube1_idx, *cube2_idx, weight);
                        to_return.edge_indices.insert(weight, eidx);
                    }
                }
                Ok(to_return)
            },
            Err(e) => Err(e.to_string()),
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

    pub fn cubes(&self) -> Vec<&Cube> {
        self.graph.node_weights().collect()
    }

    // pub fn cube_positions(&self) -> Vec<&CubePosition> {
    //     self.cube_data.keys().collect()
    // }

    // pub fn cube_from_cube_position(&self, cube_pos: &CubePosition) -> &Cube {
    //     self.cube_data.get(cube_pos).unwrap()
    // }

    pub fn pipes(&self) -> Vec<&Pipe> {
        self.graph.edge_references().map(|e| e.weight()).collect()
    }

    // pub fn occupied_positions(&self) -> Vec<&CubePosition> {
    //     self.cube_data.keys().collect()
    // }

    pub fn spanning_cubes_of(&self, pipe: &Pipe) -> (&Cube, &Cube) {
        let (idx1, idx2) = self.graph.edge_endpoints(*self.edge_indices.get(pipe).unwrap()).unwrap();
        let cube1 = self.graph.node_weight(idx1).unwrap();
        let cube2 = self.graph.node_weight(idx2).unwrap();
        (cube1, cube2)
    }

    // pub fn num_y_half_cubes(&self) -> usize {
    //     self.cube_data.values().filter(|cube| cube.is_y_half_cube()).count()
    // }

    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn is_open(&self) -> bool {
        self.num_ports() > 0
    }

    // pub fn spacetime_volume(&self) -> f64 {
    //     ((self.num_cubes() - self.num_ports() - self.num_y_half_cubes()) as f64)
    //         / 2.0
    // }

    // pub fn add_cube(&mut self, pos: CubePosition, cube: Cube, label: String) {
    //     let idx: NodeIndex = self.graph.add_node(pos.clone());
    //     self.node_indices.insert(pos.clone(), idx);
    //     self.cube_data.insert(pos.clone(), cube);
    // }

    pub fn degree(&self, cube_pos: &CubePosition) -> usize {
        let idx: Option<&NodeIndex> = self.node_indices.get(cube_pos);
        match idx {
            Some(val) => self.graph.neighbors(*val).count(),
            None => 0
        }
    }

    pub fn leaves(&self) -> Vec<CubePosition> {
        self.node_indices.keys()
            .cloned()
            .filter(|pos| self.degree(pos) == 1)
            .collect()
    }

    pub fn to_zx_graph(&self) -> PositionedZX {
        PositionedZX::from_block_graph(self) //
    }

}


