use rust_3d::Point3D;

#[derive(Debug, Clone, Copy)]
pub enum Basis {
  X,
  Z,
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
}

pub struct Port {

}

pub struct YHalfCube {

}

enum CubeKind {
  ZXCube, Port, YHalfCube
}

pub struct Cube {
  position: Point3D,
  kind: CubeKind,
  label: String
}
