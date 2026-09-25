use crate::types::coord;
use derive_new::new;

#[derive(new)]
struct GridQubit {
    x: coord,
    y: coord
}


#[derive(new)]
struct PlaquetteQubits {
    data_qubits: Vec<GridQubit>,
    syndrome_qubits: Vec<GridQubit>
}

