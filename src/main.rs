use crate::{
    abstract_observable::compile_correlation_surface_to_abstract_observable,
    block_graph::BlockGraph,
};

mod abstract_observable;
mod block_graph;
mod correlation;
mod cube;
mod pauli;
mod positioned;
mod utils;

fn main() {
    let file = "bgraphs/cnot.bgraph";
    // let file = "bgraphs/3_cnots.bgraph"; // FIXME: doesn't work because of unfinished invalid_surfaces implementation
    // let file = "bgraphs/cz.bgraph";
    // let file = "bgraphs/move_rotation.bgraph";

    let parse_res = BlockGraph::from_bgraph_file(file);
    match parse_res {
        Ok(bg) => {
            let minz = bg
                .cubes()
                .iter()
                .map(|cube| cube.position().z())
                .min()
                .unwrap();
            if minz != 0 {}
            bg.find_correlation_surfaces().into_iter().for_each(
                |cs: correlation::CorrelationSurface| {
                    println!("{:?}", cs);
                    println!("{}", cs.external_stabilizer_on_graph(&bg));
                    //let ao = compile_correlation_surface_to_abstract_observable(&bg, &cs, false);
                },
            );
        }
        Err(msg) => println!("{}", msg),
    }
}
