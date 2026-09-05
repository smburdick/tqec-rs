use std::collections::HashSet;

use crate::{cube::Basis, pauli::Pauli::Y};


#[repr(usize)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Pauli {
  I = 0b00,
  X = 0b01,
  Z = 0b10,
  Y = 0b11 // X | Z
}

impl Pauli {

  pub fn to_basis(&self) -> Result<Basis, &'static str> {
    match self {
      Pauli::X => Ok(Basis::X),
      Pauli::Z => Ok(Basis::Z),
      _ => Err("Cannot convert to basis.")
    }
  }

  pub fn vec_ixyz() -> Vec<Self> {
    vec![Pauli::I, Pauli::X, Pauli::Y, Pauli::Z]
  }

  pub fn flipped(&self, condition: bool) -> Self {
    if condition { 
      match self {
        Pauli::X => Pauli::Z,
        Pauli::Z => Pauli::X,
        Pauli::Y => Pauli::I,
        Pauli::I => Pauli::Y
      }
    } else{ 
      return self.clone();
    }
  }

  pub fn to_string(&self) -> String {
    match self {
      Pauli::X => String::from("X"),
      Pauli::Z => String::from("Z"),
      Pauli::I => String::from("I"),
      Pauli::Y => String::from("Y")
    }
  }

  pub fn xor(self, other: Pauli) -> Self {
    Self::usize_to_pauli((self as usize) ^ (other as usize))
  }

  pub fn value(&self) -> usize {
    *self as usize
  }

  pub fn from_basis_set(bases: HashSet<Basis>) -> Self {
    Self::usize_to_pauli((bases.contains(&Basis::X) as usize) | ((bases.contains(&Basis::Z) as usize) << 1))
  }

  fn usize_to_pauli(u: usize) -> Pauli {
    match u {
      0b00 => Pauli::I,
      0b01 => Pauli::X,
      0b10 => Pauli::Z,
      0b11 => Pauli::Y,
      _ => panic!("Invalid Pauli match: {}", u)
    }
  }

}
