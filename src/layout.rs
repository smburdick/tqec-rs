use crate::types::coord;

pub struct LayoutPosition2D {
    x: coord,
    y: coord,
}

pub struct LayoutPosition3D {
    spatial_position: LayoutPosition2D,
    z: coord,
}
