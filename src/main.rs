use crate::{
    abstract_observable::compile_correlation_surface_to_abstract_observable,
    block_graph::BlockGraph, correlation::CorrelationSurface,
};

mod abstract_observable;
mod block_graph;
mod correlation;
mod cube;
mod cube_spec;
mod layout;
mod pauli;
mod positioned;
mod types;
mod utils;

fn main() {
    // basic_all_test();
    benchmark();
}

fn basic_all_test() {
    for file in [
        "stability",
        "move_rotation",
        "cnot",
        "three_cnots",
        "cz",
        "steane",
    ] {
        let parse_res = BlockGraph::from_bgraph_file(format!("bgraphs/{}.bgraph", file));
        match parse_res {
            Ok(bg) => {
                println!("Validating {}", file);
                let validation = bg.validate();
                if validation.is_err() {
                    println!("{}", validation.unwrap_err());
                }
                bg.find_correlation_surfaces()
                    .into_iter()
                    .for_each(|cs: CorrelationSurface| {
                        // println!("{:?}", cs);
                        println!("{}", cs.external_stabilizer_on_graph(&bg));
                        //let ao = compile_correlation_surface_to_abstract_observable(&bg, &cs, false);
                    });
                println!();
            }
            Err(msg) => println!("{}", msg),
        }
    }
}

fn benchmark() {
    for file in [
        "big_memory"
    ] {
        let parse_res = BlockGraph::from_bgraph_file(format!("bgraphs/{}.bgraph", file));
        match parse_res {
            Ok(bg) => {

                bg.find_correlation_surfaces();
            }
            Err(msg) => println!("{}", msg),
        }
    }
}
