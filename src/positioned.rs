use quizx::{graph::GraphLike, hash_graph::Graph};

use crate::block_graph::BlockGraph; // TODO: decide which kind of graph to use (vec or hash)

pub struct PositionedZX {
  graph: Graph
}

impl PositionedZX {
  pub fn new(block_graph: &BlockGraph) -> Self {
    Self {
      graph: Graph::new() // TODO:
    }
  }
}
