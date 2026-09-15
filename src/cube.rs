use crate::types::coord;
use rand::random;
use std::{fmt, iter::once, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
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
            _ => Err("Invalid basis"),
        }
    }
}

impl Basis {
    pub fn flipped(&self) -> Basis {
        match self {
            Basis::X => Basis::Z,
            Basis::Z => Basis::X,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Basis::X => String::from("X"),
            Basis::Z => String::from("Z"),
        }
    }
}

impl fmt::Display for Basis {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", if *self == Basis::X { "X" } else { "Z" })
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Ord, PartialOrd)]
pub struct Position3D {
    x: coord,
    y: coord,
    z: coord,
}

impl Position3D {
    pub fn new(x: coord, y: coord, z: coord) -> Position3D {
        Self { x: x, y: y, z: z }
    }
    pub fn x(&self) -> coord {
        self.x
    }
    pub fn y(&self) -> coord {
        self.y
    }
    pub fn z(&self) -> coord {
        self.z
    }
    pub fn shift_by(&self, dx: coord, dy: coord, dz: coord) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            z: self.z + dz,
        }
    }
}

impl fmt::Display for Position3D {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[derive(Hash, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Cube {
    kind: CubeKind,
    position: Position3D,
}

impl Cube {
    pub fn new(kind: CubeKind, position: Position3D) -> Cube {
        Self {
            kind: kind,
            position: position,
        }
    }

    pub fn kind(&self) -> CubeKind {
        self.kind
    }

    pub fn position(&self) -> Position3D {
        self.position
    }

    pub fn eq(&self, other: &Cube) -> bool {
        self.kind == other.kind && self.position == other.position
    }

    pub fn is_spatial(&self) -> bool {
        match self.kind {
            CubeKind::ZX(zx_cube) => zx_cube.is_spatial(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ZXCube {
    x: Basis,
    y: Basis,
    z: Basis,
}

pub static ALLOWED_CUBES: &[&str] = &["ZXZ", "XZZ", "ZXX", "XZX", "XXZ", "ZZX"];

impl ZXCube {
    pub fn as_tuple(&self) -> (Basis, Basis, Basis) {
        (self.x, self.y, self.z)
    }

    pub fn is_spatial(&self) -> bool {
        self.x == self.y
    }

    pub fn from_str(rep: &str) -> Result<Self, String> {
        if ALLOWED_CUBES.contains(&rep) {
            let mut chars = rep.chars();
            Ok(Self {
                x: Basis::from_str(&chars.next().expect("X basis").to_string())?,
                y: Basis::from_str(&chars.next().expect("Y basis").to_string())?,
                z: Basis::from_str(&chars.next().expect("Z basis").to_string())?,
            })
        } else {
            Err(format!(
                "Cube with representation {r} is invalid",
                r = rep.to_string()
            ))
        }
    }

    pub fn num_z_boundaries(&self) -> usize {
        vec![self.x, self.y, self.z]
            .iter()
            .filter(|b| **b == Basis::Z)
            .count()
    }

    pub fn normal_basis(&self) -> Basis {
        match [self.x, self.y, self.z]
            .iter()
            .filter(|b| **b == Basis::Z)
            .count()
        {
            1 => Basis::Z,
            _ => Basis::X,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct PortCube {}

// pub struct YHalfCube {

// }

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum CubeKind {
    ZX(ZXCube),
    PortCube, // TODO: add implementations of port/yhalf
    YHalfCube,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pipe {
    id: u64, // Ensure uniqueness of pipes in graph. TODO: find a better way to do this.
    x: Option<Basis>,
    y: Option<Basis>,
    z: Option<Basis>,
    has_hadamard: bool,
}

impl FromStr for Pipe {
    type Err = &'static str;

    fn from_str(from: &str) -> Result<Self, Self::Err> {
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
    pub fn to_string(&self) -> String {
        [self.x, self.y, self.z]
            .iter()
            .map(|basis| {
                if basis.is_some() {
                    basis.expect("msg").to_string()
                } else {
                    String::from("0")
                }
            })
            .chain(once(String::from(if self.has_hadamard { "H" } else { "" })))
            .collect::<String>()
    }
    pub fn has_hadamard(&self) -> bool {
        self.has_hadamard
    }
    pub fn direction(&self) -> Option<Direction3D> {
        Direction3D::from_int(
            self.to_string()
                .find("0")
                .expect("Pipe has invalid direction"),
        )
    }
    pub fn as_tuple(&self) -> (Option<Basis>, Option<Basis>, Option<Basis>) {
        (self.x, self.y, self.z)
    }
}

#[repr(usize)]
#[derive(PartialEq)]
pub enum Direction3D {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Direction3D {
    pub fn from_int(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::X),
            1 => Some(Self::Y),
            2 => Some(Self::Z),
            _ => None,
        }
    }
}
