use crate::block_graph::BlockGraph;

mod cube;
mod block_graph;
mod positioned;
mod correlation;
mod pauli;
mod utils;

fn main() {
  let parse_res = BlockGraph::from_bgraph_file("bgraphs/cnot.bgraph");
  match parse_res {
    Ok(bg) => {
      bg.find_correlation_surfaces()
        .into_iter()
        .for_each(|cs: correlation::CorrelationSurface| {
          println!("{}", cs.external_stabilizer_on_graph(bg.clone()));
        });
    },
    Err(msg) => println!("{}", msg)
  }
}
