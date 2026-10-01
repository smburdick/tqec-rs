use crate::circuit::moment::Moment;
use crate::circuit::qubit_map::QubitMap;
use crate::circuit::schedule::Schedule;

pub struct ScheduledCircuit {
    moments: Vec<Moment>,
    schedule: Schedule,
    qubit_map: QubitMap
}
