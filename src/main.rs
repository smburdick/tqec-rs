use crate::block_graph::BlockGraph;

mod block_graph;
mod correlation;
mod cube;
mod pauli;
mod positioned;
mod utils;

fn main() {
    let file = "bgraphs/cnot.bgraph";
    // let file = "bgraphs/3_cnots.bgraph";
    // let file = "bgraphs/scene.bgraph";
    // let file = "bgraphs/move_rotation.bgraph";

    let parse_res = BlockGraph::from_bgraph_file(file);
    match parse_res {
        Ok(bg) => {
            bg.find_correlation_surfaces().into_iter().for_each(
                |cs: correlation::CorrelationSurface| {
                    println!("{:?}", cs);
                    println!("{}", cs.external_stabilizer_on_graph(bg.clone()));
                },
            );
        }
        Err(msg) => println!("{}", msg),
    }
}
