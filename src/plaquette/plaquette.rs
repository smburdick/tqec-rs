use std::collections::HashSet;

use crate::{circuit::circuit::ScheduledCircuit, plaquette::qubit::PlaquetteQubits};

struct Plaquette {
    name: String,
    qubits: PlaquetteQubits,
    circuit: ScheduledCircuit,
    mergeable_info: HashSet<String>,
    // debug_info: PlaquetteDebugInfo
}
