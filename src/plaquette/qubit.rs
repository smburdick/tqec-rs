use crate::types::Coord;
use derive_new::new;

#[derive(new)]
pub struct GridQubit {
    x: Coord,
    y: Coord,
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
