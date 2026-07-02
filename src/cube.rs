use std::{error::Error, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Basis {
  X,
  Z,
}

impl FromStr for Basis {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "X" => Ok(Basis::X),
            "Z" => Ok(Basis::Z),
            _ => Err("invalid basis"),
        }
    }
}

impl Basis {
  pub fn flipped(&self) -> Basis {
    match self {
      Basis::X => Basis::Z,
      Basis::Z => Basis::X
    }
  }
}

pub struct ZXCube {
  x: Basis,
  y: Basis,
  z: Basis,
}

pub static ALLOWED_CUBES: &[&str]  = &["ZXZ", "XZZ", "ZXX", "XZX", "XXZ", "ZZX"];

impl ZXCube {
  pub fn as_tuple(&self) -> (Basis, Basis, Basis) {
    (self.x, self.y, self.z)
  }
  pub fn is_spatial(&self) -> bool {
    self.x == self.y
  }
  pub fn from_str(rep: &str) -> Result<Self, &'static str> {
    if ALLOWED_CUBES.contains(&rep) {
      let mut chars = rep.chars();
      Ok(Self {
        x: Basis::from_str(&chars.next().unwrap().to_string())?,
        y: Basis::from_str(&chars.next().unwrap().to_string())?,
        z: Basis::from_str(&chars.next().unwrap().to_string())?
      })
    } else {
      Err("invalid")
    }
  }
}

pub struct Port {

}

pub struct YHalfCube {

}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum CubeKind {
  ZXCube, Port, YHalfCube
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Cube {
  kind: CubeKind,
  label: String
}

impl Cube {
  pub fn new(kind: CubeKind, label: String) -> Cube {
    Self {
      kind, label
    }
  }
  pub fn is_zx_cube(&self) -> bool {
    matches!(self.kind, CubeKind::ZXCube)
  }
  pub fn is_port(&self) -> bool {
    matches!(self.kind, CubeKind::Port)
  }
  pub fn is_y_half_cube(&self) -> bool {
    matches!(self.kind, CubeKind::YHalfCube)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeKind {
    x: Basis,
    y: Basis,
    z: Basis,
    has_hadamard: bool
}

impl FromStr for PipeKind {
    type Err = &'static str;

    fn from_str(from: &str) -> Result<Self, Self::Err>  {
      let uppercase = from.to_uppercase();
      let mut _from = uppercase.chars();
      let x = Basis::from_str(&_from.next().unwrap().to_string()).unwrap();
      let y = Basis::from_str(&_from.next().unwrap().to_string()).unwrap();
      let z = Basis::from_str(&_from.next().unwrap().to_string()).unwrap();
      let has_hadamard = _from.next().unwrap().to_string().eq("H");
      Ok(Self {x: x, y: y, z: z, has_hadamard: has_hadamard})
   }
}
