use crate::block_graph::BlockGraph;

mod cube;
mod block_graph;
mod positioned;
mod correlation;
mod pauli;
mod utils;

fn main() {
  let bg = BlockGraph::from_bgraph_file("bgraphs/cnot.bgraph").unwrap();
  assert!(bg.num_cubes() == 10);
  assert!(bg.num_pipes() == 9);
  let cs = bg.find_correlation_surfaces();
  println!("# Correlation surfaces: {}", cs.len());
}
