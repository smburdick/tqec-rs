use std::collections::HashMap;
use std::collections::HashSet;

use crate::types::Coord;
use derive_new::new;
use stim::Circuit;
use stim::CircuitItem;

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

pub const ANNOTATION_INSTRUCTIONS: [&str; 18] = [
    // Noise channels
    "CORRELATED_ERROR",
    "DEPOLARIZE1",
    "DEPOLARIZE2",
    "E",
    "ELSE_CORRELATED_ERROR",
    "HERALDED_ERASE",
    "HERALDED_PAULI_CHANNEL_1",
    "PAULI_CHANNEL_1",
    "PAULI_CHANNEL_2",
    "X_ERROR",
    "Y_ERROR",
    "Z_ERROR",
    // Annotations
    "DETECTOR",
    "MPAD",
    "OBSERVABLE_INCLUDE",
    "QUBIT_COORDS",
    "SHIFT_COORDS",
    "TICK",
];

pub fn count_qubit_accesses(circuit: &Circuit) -> HashMap<u64, u64> {
    let mut counter: HashMap<u64, u64> = HashMap::new();
    for instruction in circuit {
        match instruction {
            CircuitItem::RepeatBlock(block) => {
                for (qi, count) in count_qubit_accesses(block.body()) {
                    let item = counter.get(&qi).expect("");
                    counter.insert(qi, item + count * block.repeat_count());
                }
            }
            CircuitItem::Instruction(instruction) => {
                if ANNOTATION_INSTRUCTIONS.contains(&instruction.name()) {
                    continue;
                }
                for target in instruction.targets() {
                    if !target.is_qubit_target() {
                        continue;
                    }
                    let qi = target.qubit_value().expect("qubit value") as u64;
                    let item = counter.get(&qi).unwrap() + 1;
                    counter.insert(qi, item);
                }
            }
        }
    }
    counter
}

pub fn get_used_qubit_indices(circuit: &Circuit) -> HashSet<u64> {
    count_qubit_accesses(circuit).keys().copied().collect::<HashSet<u64>>()
}
