use rust_3d::Point3D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Basis {
  X,
  Z,
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

impl ZXCube {
  pub fn as_tuple(&self) -> (Basis, Basis, Basis) {
    return (self.x, self.y, self.z);
  }
  pub fn is_spatial(&self) -> bool {
    return self.x == self.y;
  }
}

pub struct Port {

}

pub struct YHalfCube {

}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
enum CubeKind {
  ZXCube, Port, YHalfCube
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Cube {
  position: Point3D,
  kind: CubeKind,
  label: String
}

impl Cube {
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
