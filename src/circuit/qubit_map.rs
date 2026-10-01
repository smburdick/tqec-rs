use std::collections::HashMap;

use crate::plaquette::qubit::GridQubit;


pub struct QubitMap {
    i2q: HashMap<usize, GridQubit>
}
