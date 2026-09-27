use stim::Circuit;


pub struct Moment {
    circuit: Circuit,
    used_qubits: Vec<usize>,
    avoid_checks: bool
}

impl Moment {
    pub fn new(circuit: Circuit, used_qubits: Vec<usize>, avoid_checks: bool) {

    }

    pub fn check_is_valid(circuit: Circuit) -> Result<String, String> {
        if circuit.num_ticks() > 0 {
            Err("")
        }
        
    }

}
