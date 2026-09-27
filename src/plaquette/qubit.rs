use std::collections::HashMap;

use crate::types::coord;
use derive_new::new;

#[derive(new)]
pub struct GridQubit {
    x: coord,
    y: coord,
}

#[derive(new)]
pub struct PlaquetteQubits {
    data_qubits: Vec<GridQubit>,
    syndrome_qubits: Vec<GridQubit>,
}

// pub fn count_qubit_accesses(circuit: stim::Circuit) -> HashMap<usize, usize> {
//     let counter = HashMap::new();
//     for instruction in circuit {
//         if instruction
//     }
// }
