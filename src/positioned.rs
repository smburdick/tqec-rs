use std::collections::{HashMap, HashSet};

use quizx::{graph::{EType, GraphLike, V, VType}, hash_graph::Graph, phase::Phase};

use crate::{block_graph::BlockGraph, correlation::{CorrelationSurface, ZXNode, ZXEdge}, cube::{Cube, CubeKind}, pauli::Pauli}; // TODO: decide which kind of graph to use (vec or hash)

pub struct PositionedZX {
  /// Conversion of BlockGraph into PyZX structures
  graph: Graph,
  positions: HashMap<V, Cube> // V is alias of usize
}

impl PositionedZX {

  pub fn from_block_graph(block_graph: &BlockGraph) -> Self {
    let mut graph = Graph::new();
    let mut zx2bg: HashMap<V, Cube> = HashMap::new();
    let mut bg2zx: HashMap<Cube, V> = HashMap::new();
    for cube in block_graph.cubes() {
      let (vt, phase) = PositionedZX::cube_to_zx(cube);
      let v: V = graph.add_vertex_with_phase(vt, phase);
      zx2bg.insert(v, cube.clone());
      bg2zx.insert(cube.clone(), v);
    }
    for pipe in block_graph.pipes() {
      let edge_type = if pipe.has_hadamard() { EType::H } else { EType::N };
      let (u, v) = block_graph.spanning_cubes_of(pipe);
      graph.add_edge_with_type(*bg2zx.get(u).unwrap(), *bg2zx.get(v).unwrap(), edge_type);
    }
    Self {
      graph: graph,
      positions: zx2bg
    }
  }

  pub fn cube_to_zx(cube: &Cube) -> (VType, Phase) {
    let phase: Phase = Phase::from_f64(0.0);
    match cube.kind() {
        CubeKind::ZX(zx_cube) => {
          if zx_cube.num_z_boundaries() == 1 {
            (VType::Z, phase)
          } else {
            (VType::X, phase)
          }
        },
        CubeKind::Port => (VType::B, phase),
        CubeKind::YHalfCube => (VType::Z, Phase::from_f64(0.5))
    }
  }

  pub fn supports_spiders(&self) -> bool {
    for v in self.graph.vertices() {
      let vt = self.graph.vertex_type(v);
      let phase = self.graph.phase(v);
      let pauli = vertex_type_to_pauli(vt, phase);
      if pauli.is_err() {
        return false;
      }
      let degree = self.graph.degree(v);
      let pres = pauli.unwrap();
      if degree != 1 && (pres == Pauli::I || pres == Pauli::Y) {
        return false;
      }
    }
    true
  }

  pub fn find_correlation_surfaces(&self) -> Result<Vec<CorrelationSurface>, &'static str> {
      if !self.supports_spiders() {
        return Err("Must support spiders");
      }
      let mut toReturn = Vec::new();
      // TODO: check if graph is single node
      if self.graph.num_vertices() == 1 {
        let v: V = self.graph.vertices().next().unwrap();
        let pos = self.positions.get(&v).unwrap().position();
        let vtype = self.graph.vertex_type(v);
        let phase = self.graph.vertex_data(v).phase;
        let basis = vertex_type_to_pauli(vtype, phase).unwrap().to_basis().unwrap();
        let node = ZXNode::new(pos, basis);
        let mut edges = HashSet::new();
        let edge = ZXEdge::new(node, node.clone());
        edges.insert(edge);
        toReturn.push(CorrelationSurface::new(edges));
      }
      // TODO: find correlation surfaces with vertex ordering
      Ok(toReturn)
  }

}

pub fn vertex_type_to_pauli(vtype: VType, phase: Phase) -> Result<Pauli, &'static str> {
  let zero = Phase::from(0);
  let half = Phase::from_f64(0.5);
  match (vtype, phase) {
    (VType::X, phase) if phase == zero => Ok(Pauli::X),
    (VType::Z, phase) if phase == zero => Ok(Pauli::Z),
    (VType::Z, phase) if phase == half => Ok(Pauli::Y),
    (VType::B, _) => Ok(Pauli::I), // QuiZX doesn't have identity
    _ => Err("")
  }
}
