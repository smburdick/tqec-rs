use rust_3d::Point3D;

enum Basis {
  X,
  Z,
}

pub struct ZXCube {
  x: Basis,
  y: Basis,
  z: Basis,
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
