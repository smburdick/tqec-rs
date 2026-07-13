use std::{str::FromStr};
use rand::random;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct CubePosition {
    x: i32,
    y: i32,
    z: i32
}

impl CubePosition {
    pub fn new(x: i32, y: i32, z: i32) -> CubePosition {
        Self {
            x: x, y: y, z: z
        }
    }
}

#[derive(Hash, Debug, Clone, Eq, PartialEq)]
pub struct Cube {
  kind: CubeKind,
  position: CubePosition
}

impl Cube {
  pub fn new(kind: CubeKind, position: CubePosition) -> Cube {
    Self {
      kind: kind,
      position: position
    }
  }
  pub fn kind(&self) -> CubeKind {
   self.kind
  }
  pub fn position(&self) -> CubePosition {
    self.position
  }
  pub fn eq(&self, other: &Cube) -> bool {
    self.kind == other.kind && self.position == other.position
  }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
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
  pub fn num_z_boundaries(&self) -> usize {
    vec!(self.x, self.y, self.z).iter().filter(|b| **b == Basis::Z)
      .count()
  }
}

// TODO:
// pub struct Port {

// }

// pub struct YHalfCube {

// }

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum CubeKind {
  ZX(ZXCube)
  // Port(Port),
  // YHalf(YHalfCube)
}

// impl Cube {
//   pub fn new(kind: CubeKind, label: String) -> Cube {
//     Self {
//       kind, label
//     }
//   }
//   pub fn is_zx_cube(&self) -> bool {
//     matches!(self.kind, CubeKind::ZXCube)
//   }
//   pub fn is_port(&self) -> bool {
//     matches!(self.kind, CubeKind::Port)
//   }
//   pub fn is_y_half_cube(&self) -> bool {
//     matches!(self.kind, CubeKind::YHalfCube)
//   }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pipe {
    id: u64, // Ensure uniqueness of pipes in graph.
    x: Option<Basis>,
    y: Option<Basis>,
    z: Option<Basis>,
    has_hadamard: bool
}

impl FromStr for Pipe {
    type Err = &'static str;

    fn from_str(from: &str) -> Result<Self, Self::Err>  {
      let chars: Vec<char> = from.chars().collect();

      if chars.len() < 3 {
          return Err("Pipe must contain axial metadata (x, y, z, has_hadamard)");
      }

      let parse_basis = |c: char| -> Result<Option<Basis>, Self::Err> {
          if c == 'O' {
              Ok(None)
          } else {
              Ok(Some(Basis::from_str(&c.to_string())?))
          }
      };

      Ok(Self {
          id: random(),
          x: parse_basis(chars[0])?,
          y: parse_basis(chars[1])?,
          z: parse_basis(chars[2])?,
          has_hadamard: chars.get(3) == Some(&'H'),
      })
   }
}

impl Pipe {
  pub fn has_hadamard(&self) -> bool {
    self.has_hadamard
  }
}
