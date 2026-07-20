use crate::cube::Basis;


#[repr(u8)]
#[derive(PartialEq)]
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

}
