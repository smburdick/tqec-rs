use itertools::Itertools;
use petgraph::{
    Directed, Direction, Graph,
    graph::{DiGraph, EdgeIndex, NodeIndex, UnGraph},
};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
};

use crate::{
    correlation::CorrelationSurface,
    cube::{Cube, CubeKind, Pipe, Position3D, ZXCube},
    types::coord,
};
use crate::{cube::Direction3D, positioned::PositionedZX};

#[derive(Clone, Debug)]
pub struct BlockGraph {
    name: String,
    // In tqec the blockgraph structure is undirected, but the edge weights are pipes,
    // which contain a to/from relationship (u, v)
    // Here I'm electing to encode that information using the graph itself, so I don't have
    // to store it in the pipe object.
    graph: Graph<Cube, Pipe, Directed>,
    node_indices: HashMap<Position3D, NodeIndex>, // Used for client lookups
    edge_indices: HashMap<Pipe, EdgeIndex>,
    ports: HashMap<String, Position3D>, // TODO: how to add ports?
}

impl BlockGraph {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            graph: DiGraph::default(),
            node_indices: HashMap::new(),
            edge_indices: HashMap::new(),
            ports: HashMap::new(),
        }
    }

    pub fn from_bgraph_file(filepath: String) -> Result<Self, String> {
        // Based on https://tqec.github.io/tqec/user_guide/bgraph.html
        let path = Path::new(&filepath);
        let file = File::open(&path);
        let mut to_return = Self::new(format!("block_graph[{}]", filepath));
        match file {
            Ok(goodfile) => {
                let reader = BufReader::new(goodfile);
                #[derive(PartialEq, Eq)]
                enum ParseMode {
                    HEADER,
                    CUBES,
                    PIPES,
                }
                let mut parse_mode: ParseMode = ParseMode::HEADER;
                let mut cubeIdToNodeIndex: HashMap<String, NodeIndex> = HashMap::new();
                for line in reader.lines() {
                    let _line = line.expect("Missing line");
                    if _line.len() == 1 || _line.is_empty() {
                        continue;
                    }
                    if _line.starts_with("CUBE") {
                        parse_mode = ParseMode::CUBES;
                        continue;
                    } else if _line.starts_with("PIPE") {
                        parse_mode = ParseMode::PIPES;
                        continue;
                    }
                    if parse_mode == ParseMode::CUBES {
                        let items: Vec<&str> = _line.split(";").collect();

                        let cube_id: &str = items[0];
                        let x_coord: coord = items[1].parse().expect("X coordinate");
                        let y_coord: coord = items[2].parse().expect("Y coordinate");
                        let z_coord: coord = items[3].parse().expect("Z coordinate");
                        let kind: String = items[4].to_uppercase();
                        let annotation: &str = items[5]; // TODO: how is this used?
                        let pos: Position3D = Position3D::new(x_coord, y_coord, z_coord);

                        let cube_kind: CubeKind;
                        if kind == "OOO" || kind == "P" || kind == "PORT" {
                            cube_kind = CubeKind::PortCube;
                            if to_return.ports.contains_key(annotation) {
                                return Err("Duplicate port.".to_string());
                            }
                            to_return.ports.insert(annotation.to_string(), pos.clone());
                        } else if kind.contains("Y") {
                            cube_kind = CubeKind::YHalfCube;
                        } else {
                            cube_kind = CubeKind::ZX(ZXCube::from_str(&kind)?);
                        }

                        let cube: Cube = Cube::new(cube_kind, pos.clone());
                        let idx = to_return.graph.add_node(cube);
                        to_return.node_indices.insert(pos, idx);
                        cubeIdToNodeIndex.insert(cube_id.to_string(), idx);
                    } else if parse_mode == ParseMode::PIPES {
                        let items: Vec<&str> = _line.split(";").collect();
                        let cube1_id: &str = items[0];
                        let cube2_id: &str = items[1];
                        let kind = &items[2].to_uppercase(); // FIXME: ozx is an invalid kind.

                        if !kind.contains("O") {
                            return Err("Pipe must have an opening.".to_string());
                        }

                        let cube1_idx = cubeIdToNodeIndex.get(cube1_id).unwrap();
                        let cube2_idx = cubeIdToNodeIndex.get(cube2_id).unwrap();

                        if to_return.graph.contains_edge(*cube1_idx, *cube2_idx) {
                            return Err("Invalid".to_string());
                        }

                        let weight: Pipe = Pipe::from_str(kind)?;
                        let eidx = to_return.graph.add_edge(*cube1_idx, *cube2_idx, weight);
                        to_return.edge_indices.insert(weight, eidx);
                    }
                }
                Ok(to_return)
            }
            Err(e) => {
                println!("Invalid file path");
                Err(e.to_string())
            }
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

    pub fn ordered_port_positions(&self) -> Vec<Position3D> {
        self.ports
            .keys()
            .sorted()
            .map(|str| *self.ports.get(str).unwrap())
            .collect::<Vec<Position3D>>()
    }

    pub fn cubes(&self) -> Vec<&Cube> {
        self.graph.node_weights().collect()
    }

    pub fn pipes(&self) -> Vec<&Pipe> {
        self.graph.edge_references().map(|e| e.weight()).collect()
    }

    // TODO: since Pipe implents copy, we can pass by value. Apply this everywhere
    pub fn spanning_cubes_of(&self, pipe: &Pipe) -> (&Cube, &Cube) {
        let (idx1, idx2) = self
            .graph
            .edge_endpoints(*self.edge_indices.get(&pipe).unwrap())
            .unwrap();
        let cube1 = self.graph.node_weight(idx1).unwrap();
        let cube2 = self.graph.node_weight(idx2).unwrap();
        (cube1, cube2)
    }

    pub fn get_pipe(&self, pos1: Position3D, pos2: Position3D) -> &Pipe {
        let (n1, n2) = (
            *self.node_indices.get(&pos1).expect("Node1"),
            *self.node_indices.get(&pos2).expect("Node2"),
        );
        let e_idx = self.graph.find_edge(n1, n2).expect("Edge");
        self.graph.edge_weight(e_idx).expect("pipe")
    }

    pub fn num_y_half_cubes(&self) -> usize {
        self.node_indices
            .values()
            .filter(|idx| self.graph.node_weight(**idx).unwrap().kind() == CubeKind::YHalfCube)
            .count()
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
        ((self.num_cubes() - self.num_ports() - self.num_y_half_cubes()) as f64) / 2.0
    }

    pub fn degree(&self, cube_pos: &Position3D) -> usize {
        let idx: Option<&NodeIndex> = self.node_indices.get(cube_pos);
        match idx {
            Some(val) => self.graph.neighbors(*val).count(),
            None => 0,
        }
    }

    pub fn leaves(&self) -> impl Iterator<Item = Position3D> {
        self.node_indices
            .keys()
            .filter(|pos| self.degree(pos) == 1)
            .cloned()
    }

    pub fn to_zx_graph(&self) -> PositionedZX {
        PositionedZX::from_block_graph(self)
    }

    pub fn find_correlation_surfaces(&self) -> Vec<CorrelationSurface> {
        self.to_zx_graph().find_correlation_surfaces().unwrap()
    }

    pub fn cube_at(&self, position: Position3D) -> &Cube {
        self.graph
            .node_weight(*self.node_indices.get(&position).expect("Invalid position"))
            .expect("Cube missing for position")
    }

    pub fn has_pipe_between(&self, pos1: Position3D, pos2: Position3D) -> bool {
        return self.graph.contains_edge(
            *self.node_indices.get(&pos1).expect("pos1"),
            *self.node_indices.get(&pos2).expect("pos2"),
        );
    }

    pub fn validate(&self) -> Result<String, String> {
        for cube in self.cubes() {
            match self.validate_locally_at(cube) {
                Err(msg) => {
                    return Err(msg);
                }
                _ => {}
            }
        }
        Ok("".to_string())
    }

    fn validate_locally_at(&self, cube: &Cube) -> Result<String, String> {
        let pipes = self.pipes_at(&cube.position());
        match cube.kind() {
            CubeKind::PortCube => {
                if pipes.len() != 1 {
                    return Err("Port does not have exactly one pipe connected".to_string());
                } else {
                    return Ok("".to_string());
                }
            }
            CubeKind::YHalfCube => {
                if pipes.len() != 1 {
                    return Err("YHalfCube does not have exactly one pipe connected".to_string());
                } else if pipes.len() > 1 && pipes[0].direction() == Direction3D::Z {
                    return Err("YHalfCube has non-timelike pipes connected".to_string());
                } else {
                    return Ok("".to_string());
                }
            }
            CubeKind::ZX(zx_cube) => {
                let mut pipes_by_direction: HashMap<Direction3D, Vec<Pipe>> = HashMap::new();
                pipes.iter().for_each(|pipe| {
                    pipes_by_direction
                        .entry(pipe.direction())
                        .and_modify(|v| v.push(*pipe))
                        .or_default();
                });
                for direction in Direction3D::all() {
                    match pipes_by_direction.get(&direction) {
                        Some(v) => {
                            if v.len() == 2 {
                                continue;
                            }
                        }
                        _ => {}
                    }
                    let cube_color = zx_cube.get_basis_along(direction);
                    for ortho_dir in direction.orthogonal_directions() {
                        match pipes_by_direction.get(&direction) {
                            Some(_pipes) => {
                                for pipe in _pipes {
                                    let pipe_color = pipe.get_basis_along(
                                        direction,
                                        self.pipe_at_head(*pipe, cube.position())?,
                                    )?;
                                    if pipe_color != cube_color {
                                        return Err("Cube has mismatched colors".to_string());
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                }
                return Ok("".to_string());
            }
        }
    }

    // this is a method of Pipe in tqec-py, but my design choice was to store the pipe endpoints in the graph and not
    // assign them to the pipe object itself, to have that as the source of truth.
    // I haven't been punished for this design choice, yet.
    fn pipe_at_head(&self, pipe: Pipe, position: Position3D) -> Result<bool, String> {
        let (u, v) = self.spanning_cubes_of(&pipe);
        if position == u.position() {
            return Ok(true);
        }
        if position == v.position() {
            return Ok(false);
        }
        Err("".to_string())
    }

    pub fn pipes_at(&self, pos: &Position3D) -> Vec<Pipe> {
        let idx = self.node_indices.get(pos);
        match idx {
            Some(node_index) => {
                let neighbors = self.graph.neighbors(*node_index);
                neighbors
                    .map(|node| {
                        self.graph
                            .edges_connecting(node, *node_index)
                            .map(|e| *e.weight())
                            .chain(
                                self.graph
                                    .edges_connecting(*node_index, node)
                                    .map(|e| *e.weight()),
                            )
                    })
                    .flatten()
                    .collect::<Vec<Pipe>>()
            }
            None => {
                todo!("error in fn BlockGraph::pipes_at")
            }
        }
    }
}
