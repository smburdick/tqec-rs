use std::{collections::{HashMap, HashSet}, iter};

use quizx::{graph::{EType, GraphLike, V, VType, VData}, vec_graph::Graph, phase::Phase};
use rust_3d::add;

use crate::{block_graph::BlockGraph, correlation::{CorrelationSurface, HalfEdgeCorrelationSurface, ZXEdge, ZXNode, expand_correlation_surface_to_node, reform_correlation_surface_generators}, cube::{Cube, CubeKind}, pauli::Pauli, utils::solve_linear_system}; // TODO: decide which kind of graph to use (vec or hash)

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
        return Ok(toReturn);
      }
      let leaves: Vec<V> = self.graph.vertices().filter(|v| self.graph.degree(*v) == 1).collect();
      if leaves.len() == 0 {
        return Err("The graph must contain at least one leaf node to find correlation surfaces.");
      }
      // TODO: find correlation surfaces for each connected component in the graph
      // We don't support vertex ordering yet!
      let mut components: Vec<(Graph, V)> = Vec::new();
      for component in self.as_connected_components() {
        if let Some(leaf) = component.vertices().filter(|v| component.degree(*v) == 1).min() {
          components.push((component, leaf));
        }
      }

      Ok(toReturn)
  }

  fn as_connected_components(&self) -> Vec<Graph> {
    let mut visited: HashSet<V> = HashSet::new();
    let mut components: Vec<Graph> = Vec::new();
    for start_vertex in self.graph.vertices() { // start_vertex: V
      if visited.contains(&start_vertex) {
        continue;
      }
      let mut component_vertices: HashSet<V> = HashSet::new();
      let mut stack: Vec<V> = Vec::new();
      while stack.len() > 0 {
        let vertex = stack.pop().unwrap();
        if visited.contains(&vertex) {
          continue;
        }
        visited.insert(vertex);
        component_vertices.insert(vertex);
        for n_can in self.graph.neighbor_vec(vertex) {
          if !visited.contains(&n_can) {
            stack.push(n_can);
          }
        }
      }
      let (graphs, _) = self.partition_graph_from_vertices(vec![component_vertices.iter().cloned().collect()], false);
      components.push(graphs.get(0).unwrap().clone());
    }
    components
  }

  fn partition_graph_from_vertices(&self, vertices_list: Vec<Vec<V>>,  add_cut_edge_as_boundary_node: bool) -> (Vec<Graph>, Vec<AddableVertices>) {
    let mut subgraphs: Vec<Graph> = Vec::new();
    // let mut cut_edges: = HashMap::new();
    for vertices in vertices_list {
      let mut subgraph = Graph::new();
      // let mut input_vertices = HashMap::new();
      // let mut output_vertices = HashMap::new();
      for v in vertices.iter() {
        let mut data: VData = VData::default();
        data.phase = self.graph.phase(*v);
        data.ty = self.graph.vertex_type(*v);
        subgraph.add_vertex_with_data(data);
      }
      for v in vertices.iter() {
        for u in self.graph.neighbor_vec(*v).iter() {
          if vertices.contains(&u) {
            if !subgraph.connected(*u, *v) {
              subgraph.add_edge_with_type(*u ,*v, self.graph.edge_type(*u, *v));
            } else if add_cut_edge_as_boundary_node {
              todo!("Implement this use case futher down in compilation pipeline, which includes adding input/out")
            }
          }
        }
      }
      subgraphs.push(subgraph);
    }
    (subgraphs, Vec::new())
  }

  // TODO: this should take a ZXGraph (a connected subgraph component) instead of a positionedZX object
  pub fn find_correlation_surface_generating_set_from_leaf(&self, leaf: V) -> Vec<HalfEdgeCorrelationSurface> {
    let neighbor = self.graph.neighbors(leaf).next().unwrap();
    // correlation_surfaces owns the data of each surface so the rest have to be borrowed via references
    // other vectors used here are temporary.
    let mut correlation_surfaces: Vec<HalfEdgeCorrelationSurface> = Pauli::vec_ixyz()
      .into_iter()
      .map(|pauli: Pauli| {
          let mut cs: HalfEdgeCorrelationSurface = HalfEdgeCorrelationSurface::new();
          cs.add_pauli_to_edge((leaf, neighbor), pauli, self.is_hadamard((leaf, neighbor)));
          cs
        } 
      ).collect();

    if self.graph.degree(neighbor) == 1 {
      return correlation_surfaces;
    }

    let mut frontier: Vec<V> = vec![neighbor];
    let mut explored_leaves: Vec<V> = vec![leaf];
    let mut explored_nodes: HashSet<V> = HashSet::new();
    explored_nodes.insert(leaf);

    // let mut correlation_surface: HalfEdgeCorrelationSurface;

    let func = |p: Pauli| u32::from(p.value());

    while frontier.len() > 0 {
      if let Some(current_node) = frontier.pop() {
        if correlation_surfaces.len() == 0 {
          break;
        }
        let mut correlation_surface = correlation_surfaces.pop().unwrap();
        let map =  correlation_surface.clone().mapping;
        let connected_neighbors = map.get(&current_node).unwrap();
        let unconnected_neighbors: Vec<V> = self.graph.neighbors(current_node)
          .filter(|v| !connected_neighbors.contains_key(v))
          .collect();
        let mut boundary_nodes: Vec<V> = explored_leaves.iter()
          .chain(frontier.iter()).copied().collect();

        if unconnected_neighbors.len() > 0 {
          boundary_nodes.push(current_node);
        }

        let generating_set_sz: usize = boundary_nodes.iter() 
          .map(|n| map.get(&n).unwrap().keys().len())
          .sum();
        let unexplored_neighbors: Vec<V> = unconnected_neighbors.clone().into_iter().filter(|n| !map.contains_key(n)).collect();
        let passthrough_basis: Pauli = vertex_type_to_pauli(self.graph.vertex_type(current_node), self.graph.phase(current_node)).unwrap();
        // check if each correlation surface candidate satisfies broadcast and passthrough rules
        // on the current node and is not a product of previously checked valid correlation surfaces
        let mut valid_surfaces: Vec<(HalfEdgeCorrelationSurface, Pauli, bool)> = Vec::new();
        let mut invalid_surfaces: Vec<HalfEdgeCorrelationSurface> = Vec::new();
        let mut syndromes: Vec<u32> = Vec::new();
        let mut vector_basis = HashMap::new();
        let mut cs_refs = correlation_surfaces.clone();
        cs_refs.push(correlation_surface.clone());
        let sole_cs = [&correlation_surface];

        for cs in cs_refs {
          let (p, b, u) = cs.validate_node(current_node, passthrough_basis, unconnected_neighbors.len() > 0);
          if u.is_some() {
            invalid_surfaces.push(correlation_surface.clone());
            syndromes.push(u.unwrap());
            continue;
          }
          let bd = boundary_nodes.clone();
          let x = correlation_surface.signature_at_nodes(bd.into_iter(), func, 2);
          if solve_linear_system(&mut vector_basis, x, false).unwrap().len() == 0 {
            // TODO: replace cs.clone() with Rc::clone() invocations for better scalability.
            valid_surfaces.push((correlation_surface.clone(), p.unwrap(), b.unwrap()));
          }
        }
        // TODO: try to fix local constraint violations by XORing with other invalid surfaces
        let mut syndrome_basis: HashMap<u32, (u32, u32)> = HashMap::new();
        let mut basis_surfaces: Vec<HalfEdgeCorrelationSurface> = Vec::new();
        for (cs, syndrome) in invalid_surfaces.iter().zip(syndromes.iter()) {
          if vector_basis.len() == generating_set_sz {
            break;
          }
          // python: iterate over enumerate((syndrome ^ all_one, syndrome))
          let _s = *syndrome;
          for (j, target) in [!_s, _s].iter().enumerate() {
            let indices = solve_linear_system(&mut syndrome_basis, *target, j != 0);
            if indices.is_ok() {
              let _indices = indices.unwrap();
              if _indices.len() == 0 {
                if j == 1 {
                  basis_surfaces.push(correlation_surface.clone());
                }
                continue;
              }
              let input: Vec<&HalfEdgeCorrelationSurface> = _indices
                  .iter()
                  .map(|k| basis_surfaces.get(*k as usize).unwrap())
                  .chain(sole_cs.into_iter())
                  .collect();
              let new_correlation_surface = HalfEdgeCorrelationSurface::xor(input);
              if solve_linear_system(&mut vector_basis, new_correlation_surface.signature_at_nodes(boundary_nodes.clone().into_iter(), func, 1), true).is_ok() {
                let (u, b, _) = new_correlation_surface.validate_node(current_node, passthrough_basis, unconnected_neighbors.len() > 0);
                valid_surfaces.push((new_correlation_surface, u.unwrap(), b.unwrap()));
                break;
              }
            }
          }
        }

        let edges_are_hadamard: Vec<bool> = unconnected_neighbors.iter().map(|n| self.is_hadamard((current_node, *n))).collect();

        correlation_surfaces = valid_surfaces.iter().map(|tup| expand_correlation_surface_to_node(
          // TODO: arguments
            tup.0.clone(),
            tup.1,
            tup.2,
            current_node,
            passthrough_basis,
            unconnected_neighbors.clone(),
            edges_are_hadamard.clone(),
            true,
            false
        ).into_iter()).flatten().collect();

        let _cs = correlation_surfaces.pop();
        if _cs.is_none() {
          return Vec::new();
        } else {
          correlation_surface = _cs.unwrap();
          unconnected_neighbors.iter().filter(|n| !explored_nodes.contains(*n) && self.graph.degree(**n) > 1).for_each(|n| frontier.push(*n));
          unexplored_neighbors.iter().filter(|n| self.graph.degree(**n) == 1).for_each(|n| explored_leaves.push(*n));
          explored_nodes.insert(current_node);
        }
        if frontier.len() == 0 { // final loop iteration
          correlation_surfaces.push(correlation_surface);
        }
      }
    }

    return reform_correlation_surface_generators(
      correlation_surfaces,
      |cs| cs.signature_at_nodes(
        self.graph.vertices().filter(|v| self.graph.degree(*v) == 1), func, 2
      ),
      &mut HashMap::new(),
      Vec::new(),
      false,
      self.graph.vertices().filter(|v| self.graph.degree(*v) == 1).count(),
      0
    ).0;

  }

  pub fn is_hadamard(&self, edge: (V, V)) -> bool {
    self.graph.edge_type(edge.0, edge.1) == EType::H
  }

}

pub struct AddableVertices {
  // adapt from list[tuple[dict[int, tuple[int, int]], dict[int, tuple[int, int]]]] in tqec
  // TODO: structure the data returned from partition_graph_from_vertices
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
