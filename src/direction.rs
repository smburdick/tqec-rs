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
