use std::{collections::HashMap};

use quizx::{graph::{EType, GraphLike, V, VType}, hash_graph::Graph, phase::Phase};

use crate::{block_graph::BlockGraph, cube::{Cube, CubeKind, CubePosition, ZXCube}}; // TODO: decide which kind of graph to use (vec or hash)

pub struct PositionedZX {
  graph: Graph,
  positions: HashMap<V, Cube>
}

impl PositionedZX {
  pub fn from_block_graph(block_graph: &BlockGraph) -> Self {
    let mut graph = Graph::new();
    let mut zx2bg: HashMap<V, Cube> = HashMap::new();
    let mut bg2zx: HashMap<Cube, V> = HashMap::new();
    for cube in block_graph.cubes() {
      let (vt, phase) = PositionedZX::cube_to_zx(cube);
      let v: V = graph.add_vertex_with_phase(vt, phase);
      zx2bg.insert(v, cube.clone());
      bg2zx.insert(cube.clone(), v);
    }
    for pipe in block_graph.pipes() {
      let edge_type = if pipe.has_hadamard() { EType::H } else { EType::N };
      let (u, v) = block_graph.spanning_cubes_of(pipe);
      graph.add_edge_with_type(*bg2zx.get(u).unwrap(), *bg2zx.get(v).unwrap(), edge_type);
    }
    Self {
      graph: graph,
      positions: zx2bg
    }
  }

  pub fn cube_to_zx(cube: &Cube) -> (VType, Phase) {
    match cube.kind() {
        CubeKind::ZX(zx_cube) => {
          let phase: Phase = Phase::from_f64(0.0);
          if zx_cube.num_z_boundaries() == 1 {
            (VType::Z, phase)
          } else {
            (VType::X, phase)
          }
        }
      } // TODO: implement port/yhalf
  }
}
