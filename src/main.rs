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
    basic_all_test();
}

fn basic_all_test() {

    for file in ["move_rotation", "three_cnots", "cz", "steane"] {
        let parse_res = BlockGraph::from_bgraph_file(format!("bgraphs/{}.bgraph", file));
        match parse_res {
            Ok(bg) => {
                println!("{}", file);
                bg.find_correlation_surfaces().into_iter().for_each(
                    |cs: correlation::CorrelationSurface| {
                        // println!("{:?}", cs);
                        println!("{}", cs.external_stabilizer_on_graph(&bg));
                        //let ao = compile_correlation_surface_to_abstract_observable(&bg, &cs, false);
                    },
                );
                println!();
            }
            Err(msg) => println!("{}", msg),
        }
    }

}
