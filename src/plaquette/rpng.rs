

pub enum PauliBasis {
    X, Y, Z
}

pub enum ExtendedBasis {
    Pauli(PauliBasis),
    H
}

pub struct RPNG {
    r: Option<ExtendedBasis>,
    p: Option<PauliBasis>,
    n: Option<usize>,
    g: Option<ExtendedBasis>
}
