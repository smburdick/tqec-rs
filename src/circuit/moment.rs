use stim::Circuit;

use crate::plaquette::qubit::count_qubit_accesses;


pub struct Moment {
    circuit: Circuit,
    used_qubits: Vec<usize>,
    avoid_checks: bool
}

impl Moment {

    pub fn check_is_valid(circuit: &Circuit) -> Result<String, String> {
        if circuit.num_ticks() > 0 {
            return Err("Cannot initialize a Moment with a stim.Circuit instance containing at least one TICK instruction.".to_string());
        }
        let qubit_usage = count_qubit_accesses(circuit);
        let multi_used_qubits: Vec<u64> = qubit_usage.iter().filter(|(_, usage_count)| **usage_count > 1).map(|(qi, _)| *qi).collect();
        if multi_used_qubits.len() > 0 {
            return Err("".to_string()); // TODO: report multiply used qubits
        }
        Ok("".to_string())
    }

}
