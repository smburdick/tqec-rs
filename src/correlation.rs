use petgraph::graph::Frozen;
use quizx::{detection_webs::PauliWeb, graph::V, vec_graph::Graph};

use crate::{cube::{Basis, CubePosition}, pauli::Pauli, positioned::PositionedZX, utils::concat_ints_as_bits};
use std::{collections::{HashMap, HashSet}, iter::{self, repeat}};
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
  pub fn sorted(u: ZXNode, v: ZXNode) -> Self {
    todo!()
  }
}

pub struct CorrelationSurface {
  edges: FrozenSet<ZXEdge>
}

impl CorrelationSurface {

  pub fn new(edges: HashSet<ZXEdge>) -> Self {
    Self { edges: edges.clone().freeze() }
  }

}

#[derive(Clone)]
pub struct HalfEdgeCorrelationSurface {
 pub mapping: HashMap<V, HashMap<V, Pauli>>
}

impl HalfEdgeCorrelationSurface {

  pub fn new() -> Self {
    Self { mapping: HashMap::new() }
  }

  pub fn is_single_node(&self) -> bool {
    self.mapping.keys().len() == 1 && self.mapping.values().len() == 1
  }

  pub fn to_immutable_public_representation(&self, graph: PositionedZX) -> CorrelationSurface {
    todo!("")
  }

  pub fn add_pauli_to_edge(&mut self, edge: (V, V), pauli: Pauli, edge_is_hadamard: bool) {
    let (u, v) = edge;
    for (from, to, p) in [
      (u, v, pauli),
      (v, u, pauli.flipped(edge_is_hadamard))] {
        self.mapping.entry(from)
          .or_default()
          .insert(to, p);
    }
  }

  pub fn validate_node(&self, node: V, basis: Pauli, has_unconnected_neighbors: bool) -> (Option<Pauli>, Option<bool>, Option<u32>) {
    let paulis: Vec<Pauli> = self.paulis_at_nodes(iter::once(node)).collect();
    let passthru_parity = paulis.iter().copied().reduce(|acc, p| acc.xor(p)).unwrap() == basis;
    let mut valid = true;
    let broadcast_basis = basis.flipped(true);
    let mut syndrome: Vec<bool> = paulis.iter().copied().map(|p| p == broadcast_basis).collect();
    let mut broadcast_pauli: Pauli = Pauli::I;
    if syndrome.iter().all(|&b| b) {
      broadcast_pauli = broadcast_basis;
    } else if syndrome.iter().all(|&b| !b) {
      broadcast_pauli = Pauli::I;
    } else {
      valid = false;
    }
    if !has_unconnected_neighbors {
      syndrome.push(passthru_parity);
      if passthru_parity {
        valid = false;
      }
    }
    if valid {
      return (Some(broadcast_pauli), Some(passthru_parity), None);
    } else {
      return (None, None, Some(concat_ints_as_bits(syndrome.iter().map(|&b| b as u32), 1..)))
    }
  }

  pub fn paulis_at_nodes(&self, nodes: impl Iterator<Item = V>) -> impl Iterator<Item = Pauli> {
    nodes.map(|v| self.mapping.get(&v).unwrap().values()).flatten().map(|p| *p)
  }

  pub fn signature_at_nodes<F>(&self, nodes: impl Iterator<Item = V>, func: F, bit_length: u32) -> u32 where F: Fn(Pauli) -> u32 {
    let paulis = self.paulis_at_nodes(nodes);
    let ints = paulis.map(|x|  func(x));
    concat_ints_as_bits(ints, bit_length..)
  }

}

pub fn find_correlation_surfaces_from_leaf(zx_graph: Graph, leaf: V) -> Vec<HalfEdgeCorrelationSurface> {
  todo!("")
}

