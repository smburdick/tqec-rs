use crate::block_graph::BlockGraph;

pub mod cube;
pub mod block_graph;

fn main() {
  let bg = BlockGraph::from_bgraph_file("logical_cnot.dae");
}
