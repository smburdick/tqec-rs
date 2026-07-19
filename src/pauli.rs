
#[repr(u8)]
#[derive(PartialEq)]
pub enum Pauli {
  I = 0b00,
  X = 0b01,
  Z = 0b10,
  Y = 0b11 // X | Z
}
