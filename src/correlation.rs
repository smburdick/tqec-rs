use petgraph::graph::Frozen;
use quizx::{detection_webs::PauliWeb, graph::V, vec_graph::Graph};

use crate::{cube::{Basis, CubePosition}, pauli::Pauli, positioned::PositionedZX, utils::{concat_ints_as_bits, solve_linear_system}};
use std::{collections::{HashMap, HashSet}, iter::{self, repeat}};
use frozenset::{FrozenSet, Freeze};
use itertools::{Combinations, Itertools};

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

pub fn xor(cses: Vec<&Self>) -> Self {
    let mut result = Self::new();

    let (first, others) = cses
        .split_first()
        .expect("xor requires at least one circuit");

    for (v, neighbors) in &first.mapping {
        let mut val = HashMap::new();

        for (n, pauli) in neighbors {
            let mut res_pauli = pauli.clone();

            for cs in others {
                let neighbor_row = cs
                    .mapping
                    .get(v)
                    .expect("vertex missing from mapping");

                let other_pauli = neighbor_row
                    .get(n)
                    .expect("neighbor missing from mapping");

                res_pauli = res_pauli.xor(*other_pauli);
            }
            val.insert(*n, res_pauli);
        }
        result.mapping.insert(*v, val);
    }
    result
}

}

pub fn generate_valid_local_paulis(
  node_basis: Pauli,
  broadcast_pauli: Pauli,
  passthrough_parity: bool,
  num_unconnected_neighbors: usize,
  generate_all: bool
) -> Vec<Vec<Pauli>> {
  let mut result: Vec<Vec<Pauli>> = Vec::new();
  let unconnected_neighbors = 1..num_unconnected_neighbors;
  let combined_pauli = broadcast_pauli.xor(node_basis);
  if generate_all {
    let passthru_nodes = ((passthrough_parity as usize)..(unconnected_neighbors.len() + 1))
      .step_by(2)
      .flat_map(|n| unconnected_neighbors.clone().combinations(n));
    result = passthru_nodes.map(|p| unconnected_neighbors.clone().map(|n| if p.contains(&n) {combined_pauli} else {broadcast_pauli}).collect()).collect();
  } else {
    todo!("Not implemented yet")
  }
  result
}

pub fn expand_correlation_surface_to_node(
  correlation_surface: HalfEdgeCorrelationSurface,
  broadcast_pauli: Pauli,
  passthrough_parity: bool,
  node: V,
  node_basis: Pauli,
  unconnected_neighbors: Vec<V>,
  edges_are_hadamard: Vec<bool>,
  generate_all: bool,
  always_copy: bool
) -> Vec<HalfEdgeCorrelationSurface> { // TODO: python version uses generator instead, consider using that.
  let mut new_correlation_surfaces: Vec<HalfEdgeCorrelationSurface> = Vec::new();
  let cs = correlation_surface.clone();
  for (i, out_paulis) in generate_valid_local_paulis(node_basis, broadcast_pauli, passthrough_parity, unconnected_neighbors.len(), generate_all).iter().enumerate() {
    let mut new_correlation_surface: HalfEdgeCorrelationSurface = correlation_surface.clone();
    if i != 0 || always_copy {
      new_correlation_surface.mapping.insert(node, new_correlation_surface.mapping.get(&node).unwrap().clone());
    }
    for (n, pauli, edge_is_hadamard) in unconnected_neighbors.iter().zip(out_paulis.iter()).zip(edges_are_hadamard.iter()).map(|((x, y), z)| (x, y, z)) {
      if (i > 0 || always_copy) && cs.mapping.contains_key(n) {
        new_correlation_surface.mapping.insert(*n, cs.mapping.get(n).unwrap().clone());
      }
      new_correlation_surface.add_pauli_to_edge((node, *n), *pauli, *edge_is_hadamard);
    }
    new_correlation_surfaces.push(new_correlation_surface);
  }
  new_correlation_surfaces
}

pub fn reform_correlation_surface_generators<F>(
    correlation_surfaces: Vec<HalfEdgeCorrelationSurface>,
    signature_func: F,
    stabilizer_basis: &mut HashMap<u32, (u32, u32)>,
    basis_surfaces: Vec<HalfEdgeCorrelationSurface>,
    construct_new_surfaces: bool,// = True,
    num_new_surfaces_needed: usize, // | None = None,
    num_basis_surfaces_needed: usize //int | None = None,
) -> (
    Vec<HalfEdgeCorrelationSurface>,
    Vec<HalfEdgeCorrelationSurface>,
)
where
    F: Fn(HalfEdgeCorrelationSurface) -> u32,
{
  let mut new_basis_surfaces = basis_surfaces.clone();
  for cs in correlation_surfaces {
    let indices = solve_linear_system(stabilizer_basis, signature_func(cs.clone()), true);
    if indices.is_err() {
      new_basis_surfaces.push(cs);
      if num_basis_surfaces_needed > 0 && basis_surfaces.len() > num_basis_surfaces_needed {
        break;
      }
      continue;
    }
    if construct_new_surfaces {
      todo!("")
    }
  }
  (new_basis_surfaces, Vec::new())
}

pub fn find_correlation_surfaces_from_leaf(zx_graph: PositionedZX, leaf: V) -> Vec<HalfEdgeCorrelationSurface> {
  let correlation_surfaces = zx_graph.find_correlation_surface_generating_set_from_leaf(leaf);
  todo!("")
}

