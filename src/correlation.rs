use quizx::detection_webs::PauliWeb;

use crate::{cube::{Basis, CubePosition}, positioned::PositionedZX};
use std::collections::HashSet;

pub struct ZXNode {
  position: CubePosition,
  basis: Basis
}

pub struct ZXEdge {
  u: ZXNode,
  v: ZXNode
}

pub struct CorrelationSurface {
  edges: HashSet<ZXEdge> // TODO: freeze it
}

