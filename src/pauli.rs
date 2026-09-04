use crate::cube::Basis;


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

  pub fn xor(self, other: Pauli) -> Self {
    match (self as usize) ^ (other as usize) {
      0b00 => Pauli::I,
      0b01 => Pauli::X,
      0b10 => Pauli::Z,
      0b11 => Pauli::Y,
      _ => unreachable!(),
    }
  }

  pub fn value(&self) -> usize {
    *self as usize
  }

}
