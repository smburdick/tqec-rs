use crate::block_graph::BlockGraph;

mod cube;
mod block_graph;
mod positioned;

fn main() {
  let bg = BlockGraph::from_bgraph_file("bgraphs/cnot.bgraph").unwrap();
  assert!(bg.num_cubes() == 10);
  assert!(bg.num_pipes() == 9);
}
