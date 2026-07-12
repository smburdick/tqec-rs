use std::{str::FromStr};

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

#[derive(Debug, Clone)]
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
  pub fn num_z_boundaries(&self) -> u8 {
    // TODO: apply DRY to this code.
    let mut n = 0;
    if matches!(self.x, Basis::Z) {
      n += 1;
    }
    if matches!(self.y, Basis::Z) {
      n += 1;
    }
    if matches!(self.z, Basis::Z) {
      n += 1;
    }
    n
  }
}

// pub struct Port {

// }

// pub struct YHalfCube {

// }

// #[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
// pub enum CubeKind {
//   ZXCube, Port, YHalfCube
// }


#[derive(Debug, Clone)]
pub enum Cube {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pipe {
    x: Option<Basis>,
    y: Option<Basis>,
    z: Option<Basis>,
    has_hadamard: bool
}

impl FromStr for Pipe {
    type Err = &'static str;

    fn from_str(from: &str) -> Result<Self, Self::Err>  {
      let uppercase: Vec<char> = from.chars().collect();

      // FIXME: make me less repetitve please..
      let x_char = uppercase[0];
      let mut x: Option<Basis>;
      if x_char == 'O' {
        x = None;
      } else {
        x = Some(Basis::from_str(&x_char.to_string()).unwrap());
      }
      let y_char = uppercase[1];
      let mut y: Option<Basis>;
      if y_char == 'O' {
        y = None;
      } else {
        y = Some(Basis::from_str(&y_char.to_string()).unwrap());
      }
      let z_char = uppercase[2];
      let mut z: Option<Basis>;
      if z_char == 'O' {
        z = None;
      } else {
        z = Some(Basis::from_str(&z_char.to_string()).unwrap());
      }
      let has_hadamard = uppercase.len() == 4 && uppercase[3] == 'H';
      Ok(Self {x: x, y: y, z: z, has_hadamard: has_hadamard})
   }
}
