use std::{collections::HashMap};

use fraction::Fraction;
use quizx::{graph::{GraphLike, VType}, hash_graph::Graph};

use crate::{block_graph::{BlockGraph, CubePosition}, cube::{Cube, ZXCube}}; // TODO: decide which kind of graph to use (vec or hash)

pub struct PositionedZX {
  graph: Graph, // use ZX graph
  positions: HashMap<i32, CubePosition> // TODO: may need different key that's ZXGraph firendlier
}

impl PositionedZX {
  pub fn from_block_graph(block_graph: &BlockGraph) -> Self {
    // TODO: iterate over bgraph cubes, convert to ZXGraph vertices

    // TODO: iterate over bgraph pipes, convert to ZXGraph edges
    Self {
      graph: Graph::new(),
      positions: HashMap::new()
    }
  }
  // TODO: use traits or generics to have different kinds of cubes?
  pub fn cube_to_zx(cube: &Cube) -> (VType, Fraction) {
    match cube {
        Cube::ZX(_cube) => {
          if _cube.num_z_boundaries() == 1 {
            (VType::Z, Fraction::new(0u32, 1u32))
          } else {
            (VType::X, Fraction::new(0u32, 1u32))
          }
        }
      } // TODO: implement port/yhalf
  }
}
