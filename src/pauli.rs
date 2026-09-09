use std::collections::HashSet;
use bitflags::bitflags;

use crate::{cube::Basis};

bitflags! {

    #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    pub struct Pauli: u8 {
        const I = 0b00;
        const X = 0b01;
        const Z = 0b10;
        const Y = 0b11; // X | Z
    }

}


impl Pauli {

    pub fn to_basis(&self) -> Result<Basis, &'static str> {
        match *self {
            Pauli::X => Ok(Basis::X),
            Pauli::Z => Ok(Basis::Z),
            _ => Err("Cannot convert to basis."),
        }
    }

    pub fn vec_ixyz() -> Vec<Self> {
        vec![Pauli::I, Pauli::X, Pauli::Y, Pauli::Z]
    }

    pub fn flipped(&self, condition: bool) -> Self {
        if condition {
            match *self {
                Pauli::X => Pauli::Z,
                Pauli::Z => Pauli::X,
                _ => *self,
            }
        } else {
            return self.clone();
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            Pauli::X => String::from("X"),
            Pauli::Z => String::from("Z"),
            Pauli::I => String::from("I"),
            Pauli::Y => String::from("Y"),
            _ => panic!("Invalid pauli conversion")
        }
    }

    pub fn from_basis_set(bases: HashSet<Basis>) -> Self {
        ((bases.contains(&Basis::X) as u8) | ((bases.contains(&Basis::Z) as u8) << 1)) as Pauli
    }

}
