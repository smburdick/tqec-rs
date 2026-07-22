use petgraph::graph::Frozen;
use quizx::detection_webs::PauliWeb;

use crate::{cube::{Basis, CubePosition}, positioned::PositionedZX};
use std::collections::HashSet;
use frozenset::{FrozenSet, Freeze};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZXNode {
  position: CubePosition,
  basis: Basis
}

impl ZXNode {
  pub fn new(position: CubePosition, basis: Basis) -> Self {
    Self { position: position, basis: basis }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZXEdge {
  u: ZXNode,
  v: ZXNode
}


impl ZXEdge {
  pub fn new(u: ZXNode, v: ZXNode) -> Self {
    Self { u: u, v: v}
  }
}

pub struct CorrelationSurface {
  edges: FrozenSet<ZXEdge>
}

impl CorrelationSurface {

  pub fn new(edges: HashSet<ZXEdge>) -> Self {
    Self { edges: edges.clone().freeze() }
  }

  pub fn find_correlation_surfaces_with_vertex_ordering() {

  }

}

