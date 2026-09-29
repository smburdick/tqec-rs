use crate::types::Coord;

pub struct LayoutPosition2D {
    x: Coord,
    y: Coord,
}

pub struct LayoutPosition3D {
    spatial_position: LayoutPosition2D,
    z: Coord,
}
