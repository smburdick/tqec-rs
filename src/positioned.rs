use std::{cell::RefCell, collections::{HashMap, HashSet}, iter::{Peekable, once}, rc::Rc};

use itertools::Itertools;
use quizx::{
    graph::{EType, GraphLike, V, VData, VType},
    phase::Phase,
    vec_graph::Graph,
};

use crate::{
    block_graph::BlockGraph,
    correlation::{
        CorrelationSurface, HalfEdgeCorrelationSurface, ValidationResult, ZXEdge, ZXNode,
        expand_correlation_surface_to_node, find_correlation_surfaces_from_leaf,
        reform_correlation_surface_generators,
    },
    cube::{Basis, Cube, CubeKind, Position3D},
    pauli::Pauli,
    utils::{solve_linear_system, zx_to_pauli},
};

pub struct PositionedZX {
    /// Conversion of BlockGraph into PyZX structures
    graph: Graph,
    positions: HashMap<V, Cube>, // V is alias of usize
}

type SharedSurface = Rc<RefCell<HalfEdgeCorrelationSurface>>;

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
        let pipes = block_graph.pipes();
        for pipe in pipes {
            let edge_type = if pipe.has_hadamard() {
                EType::H
            } else {
                EType::N
            };
            let (u, v) = block_graph.spanning_cubes_of(pipe);
            graph.add_edge_with_type(*bg2zx.get(u).unwrap(), *bg2zx.get(v).unwrap(), edge_type);
        }
        Self {
            graph: graph,
            positions: zx2bg,
        }
    }

    pub fn edges(&self) -> impl Iterator<Item = (V, V, EType)> {
        self.graph.edges()
    }

    pub fn edge_is_hadamard(&self, edge: (V, V)) -> bool {
        self.graph.edge_type(edge.0, edge.1) == EType::H
    }

    pub fn get_cube_at(&self, v: V) -> Option<&Cube> {
        self.positions.get(&v)
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
            }
            CubeKind::PortCube => (VType::B, phase),
            CubeKind::YHalfCube => (VType::Z, Phase::from_f64(0.5)),
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
            let basis = vertex_type_to_pauli(vtype, phase)
                .unwrap()
                .to_basis()
                .unwrap();
            let node = ZXNode::new(pos, basis);
            let mut edges = HashSet::new();
            let edge = ZXEdge::new(node, node.clone());
            edges.insert(edge);
            toReturn.push(CorrelationSurface::new(edges));
            return Ok(toReturn);
        }
        let leaves: Vec<V> = self
            .graph
            .vertices()
            .filter(|v| self.graph.degree(*v) == 1)
            .collect();
        if leaves.len() == 0 {
            return Err(
                "The graph must contain at least one leaf node to find correlation surfaces.",
            );
        }

        let components: Vec<(Graph, V)> = self
            .as_connected_components()
            .iter()
            .map(|component: &Graph| {
                (
                    component.clone(),
                    component
                        .vertices()
                        .filter(|v| component.degree(*v) == 1)
                        .min()
                        .unwrap(),
                )
            })
            .collect();

        let result: Vec<CorrelationSurface> = components
            .iter()
            .map(|(g, v)| find_correlation_surfaces_from_leaf(g, *v).into_iter())
            .multi_cartesian_product()
            .flatten()
            .map(|cs| cs.to_immutable_public_representation(self))
            .collect();

        Ok(result)
    }

    fn as_connected_components(&self) -> Vec<Graph> {
        let mut visited: HashSet<V> = HashSet::new();
        let mut components: Vec<Graph> = Vec::new();
        for start_vertex in self.graph.vertices() {
            // start_vertex: V
            if visited.contains(&start_vertex) {
                continue;
            }
            let mut component_vertices: HashSet<V> = HashSet::new();
            let mut stack: Vec<V> = vec![start_vertex];
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
            let (graphs, _) = self.partition_graph_from_vertices(
                vec![component_vertices.iter().cloned().collect()],
                false,
            );
            components.push(graphs.get(0).unwrap().clone());
        }
        components
    }

    fn partition_graph_from_vertices(
        &self,
        vertices_list: Vec<Vec<V>>,
        add_cut_edge_as_boundary_node: bool,
    ) -> (Vec<Graph>, Vec<AddableVertices>) {
        let mut subgraphs: Vec<Graph> = Vec::new();
        // let mut cut_edges: = HashMap::new();
        for vertices in vertices_list {
            let mut subgraph = Graph::new();
            // let mut input_vertices = HashMap::new();
            // let mut output_vertices = HashMap::new();
            for v in vertices.iter().sorted() {
                let mut data: VData = VData::default();
                data.phase = self.graph.phase(*v);
                data.ty = self.graph.vertex_type(*v);
                subgraph.add_vertex_with_data(data);
                // subgraph.add_vertex_with_phase(self.graph.vertex_type(*v), self.graph.phase(*v));
            }
            for v in vertices.iter() {
                for u in self.graph.neighbor_vec(*v).iter() {
                    if vertices.contains(&u) {
                        if !subgraph.connected(*u, *v) {
                            subgraph.add_edge_with_type(*u, *v, self.graph.edge_type(*u, *v));
                        } else if add_cut_edge_as_boundary_node {
                            todo!(
                                "Implement this use case futher down in compilation pipeline, which includes adding input/out"
                            )
                        }
                    }
                }
            }
            subgraphs.push(subgraph);
        }
        (subgraphs, Vec::new()) // TODO: add input/output vertices
    }

    pub fn find_correlation_surface_generating_set_from_leaf(
        graph: &Graph,
        leaf: V,
    ) -> Vec<HalfEdgeCorrelationSurface> {
        let neighbor = graph.neighbors(leaf).next().unwrap();
        // correlation_surfaces owns the data of each surface so the rest have to be borrowed via references
        // other vectors used here are temporary.
        let mut correlation_surfaces: Box<dyn Iterator<Item = SharedSurface>> = Box::new([Pauli::X, Pauli::Z]
            .into_iter()
            .map(|pauli: Pauli| {
                let mut cs: HalfEdgeCorrelationSurface = HalfEdgeCorrelationSurface::new();
                cs.add_pauli_to_edge(
                    (leaf, neighbor),
                    pauli,
                    Self::is_hadamard(graph, (leaf, neighbor)),
                );
                Rc::new(RefCell::new(cs))
            }));

        if graph.degree(neighbor) == 1 {
            return correlation_surfaces.map(|cs| cs.borrow().clone()).collect();
        }

        let mut frontier: Vec<V> = vec![neighbor];
        let mut explored_leaves: Vec<V> = vec![leaf];
        let mut explored_nodes: HashSet<V> = HashSet::new();
        explored_nodes.insert(leaf);

                   // These vectors gain ownership of the surfaces
        let mut vector_basis: HashMap<usize, (usize, usize)> = HashMap::new();

        let mut syndrome_basis: HashMap<usize, (usize, usize)> = HashMap::new();
        let mut basis_surfaces: Vec<SharedSurface> = Vec::new();


        let pauli_value = |p: Pauli| p.value(); // used in sub functions.
        while frontier.len() > 0 {

            let _correlation_surface = correlation_surfaces.next();
            if _correlation_surface.is_none() {
                return Vec::new();
            }
            let correlation_surface = _correlation_surface.unwrap();

            let current_node = frontier.remove(0);
            let map = &correlation_surface.borrow().mapping;

            let connected_neighbors: Vec<V> = map
                .get(&current_node)
                .expect(&format!("Connected neighbors at vertex {}", current_node))
                .keys()
                .map(|&a| a)
                .collect::<Vec<V>>();

            let unconnected_neighbors: Vec<V> = graph
                .neighbors(current_node)
                .filter(|v| !connected_neighbors.contains(v))
                .collect();

            let mut boundary_nodes: Vec<V> = explored_leaves
                .iter()
                .chain(frontier.iter())
                .copied()
                .collect();

            if unconnected_neighbors.len() > 0 {
                boundary_nodes.push(current_node);
            }

            let generating_set_sz: usize = boundary_nodes
                .iter()
                .map(|n|  map.get(&n).map_or(0, |inner| inner.len()))
                .sum();

            let unexplored_neighbors: Vec<V> = unconnected_neighbors
                .iter()
                .filter(|n| !map.contains_key(n))
                .copied()
                .collect();

            let passthrough_basis: Pauli =
                vertex_type_to_pauli(graph.vertex_type(current_node), graph.phase(current_node))
                    .expect(&format!("passthru parity of node {}", current_node));

            // check if each correlation surface candidate satisfies broadcast and passthrough rules
            // on the current node and is not a product of previously checked valid correlation surfaces

            let mut invalid_surfaces: Vec<(SharedSurface, usize)> = Vec::new();
            let mut valid_surfaces: Vec<(SharedSurface, Pauli, bool)> = Vec::new();


            for cs in once(Rc::clone(&correlation_surface)).chain(correlation_surfaces) {
                match cs.borrow().validate_node(
                    current_node,
                    passthrough_basis,
                    unconnected_neighbors.len() > 0,
                ) {
                    ValidationResult::Single(u) => {
                        invalid_surfaces.push((Rc::clone(&cs), u));
                        continue;
                    }
                    ValidationResult::Pair(pauli, parity) => {
                        let x = cs.borrow().signature_at_nodes(
                            boundary_nodes.clone().into_iter(),
                            pauli_value,
                            2,
                        );

                        if solve_linear_system(&mut vector_basis, x, true).is_err() {
                            valid_surfaces.push((Rc::clone(&cs), pauli, parity));

                            if vector_basis.len() == generating_set_sz {
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }

            // try to fix local constraint violations by XORing with other invalid surfaces

            for (cs, syndrome) in invalid_surfaces {
                if vector_basis.len() == generating_set_sz {
                    break;
                }

                let all_one = (1 << connected_neighbors.len()) - 1;
                for (j, target) in [syndrome ^ all_one, syndrome].iter().enumerate() {
                    let indices = solve_linear_system(&mut syndrome_basis, *target, j != 0);

                    if indices.is_err() {
                        if j == 1 {
                            basis_surfaces.push(cs.clone());
                        }
                        continue;
                    }

                    if indices.is_ok() {

                        let borrowed: Vec<_> = indices
                            .unwrap()
                            .iter()
                            .map(|k| basis_surfaces.get(*k).unwrap().borrow())
                            .chain(once(correlation_surface.borrow()))
                            .collect();

                        let input = borrowed.iter().map(|r| &**r).collect();

                        let new_correlation_surface = HalfEdgeCorrelationSurface::xor(input);

                        if solve_linear_system(
                            &mut vector_basis,
                            new_correlation_surface.signature_at_nodes(
                                boundary_nodes.clone().into_iter(),
                                pauli_value,
                                1,
                            ),
                            true,
                        )
                        .is_ok()
                        {
                            match new_correlation_surface.validate_node(
                                current_node,
                                passthrough_basis,
                                unconnected_neighbors.len() > 0,
                            ) {
                                ValidationResult::Pair(pauli, parity) => {
                                    valid_surfaces.push((Rc::new(RefCell::new(new_correlation_surface)), pauli, parity));
                                }
                                _ => {}
                            }
                            break;
                        }
                    }
                }
            }

            let edges_are_hadamard: Vec<bool> = unconnected_neighbors
                .iter()
                .map(|n| Self::is_hadamard(graph, (current_node, *n)))
                .collect();

            correlation_surfaces = Box::new(valid_surfaces
                .into_iter()
                .map(move |(cs, broadcast, parity)| {
                    expand_correlation_surface_to_node(
                        cs,
                        broadcast,
                        parity,
                        current_node,
                        passthrough_basis,
                        unconnected_neighbors.clone(),
                        edges_are_hadamard.clone(),
                        true,
                        false,
                    )
                })
                .flatten());


            unexplored_neighbors
                .iter()
                .filter(|n| !explored_nodes.contains(*n) && graph.degree(**n) > 1)
                .for_each(|n| frontier.push(*n));

            unexplored_neighbors
                .iter()
                .filter(|n| graph.degree(**n) == 1)
                .for_each(|n| explored_leaves.push(*n));

            explored_nodes.insert(current_node);

            vector_basis.clear();
            syndrome_basis.clear();
            basis_surfaces.clear();

        }

        return reform_correlation_surface_generators(
            correlation_surfaces.map(|cs| cs.borrow().clone()),
            |cs| {
                cs.signature_at_nodes(
                    graph.vertices().filter(|v| graph.degree(*v) == 1),
                    pauli_value,
                    2,
                )
            },
            &mut HashMap::new(),
            Vec::new(),
            false,
            graph.vertices().filter(|v| graph.degree(*v) == 1).count(),
            0,
        )
        .0;
    }

    fn is_hadamard(graph: &Graph, edge: (V, V)) -> bool {
        graph.edge_type(edge.0, edge.1) == EType::H
    }

    pub fn correlation_surface_is_valid(
        &self,
        correlation_surface: &CorrelationSurface,
    ) -> Result<String, String> {
        let mut p2v: HashMap<Position3D, usize> = HashMap::new();
        self.positions.iter().for_each(|(v, p)| {
            p2v.insert(p.position(), *v);
        });

        // check if surface's vertices are in the graph
        let binding = p2v.keys().map(|pos| *pos).collect::<HashSet<Position3D>>();
        let positions = correlation_surface.positions();
        let diff = positions.difference(&binding);
        if diff.count() > 0 {
            panic!("Diffs are wrong")
        }

        // check if correlation surface has all the edges in the graph
        let edges: Vec<(V, V)> = self.graph.edges().map(|(u, v, _)| (u, v)).collect();
        for edge in correlation_surface.span() {
            let (_u, _v) = edge.nodes();
            let (u, v) = (p2v.get(_u.position()), p2v.get(_v.position()));
            if u.is_none() || v.is_none() {
                panic!("Invalid edge")
            }
            if !edges.contains(&(*u.unwrap(), *v.unwrap()))
                && !edges.contains(&(*v.unwrap(), *u.unwrap()))
            {
                panic!("Cannot find edge")
            }
        }

        // check parity around each vertex
        for pos in correlation_surface.positions() {
            let v = *p2v
                .get(&pos)
                .expect(&format!("Missing position at {:?}", pos));
            let pauli = zx_to_pauli(&self.graph, v);
            let edges = correlation_surface.edges_at(pos);
            match pauli {
                Pauli::I => {
                    continue;
                }
                Pauli::Y => {
                    if edges.len() != 2 || correlation_surface.bases_at(pos).len() != 2 {
                        panic!("")
                    }
                }
                _ => {
                    let mut counts: HashMap<Basis, usize> = HashMap::new();
                    edges
                        .iter()
                        .for_each(|edge| *counts.entry(edge.get_basis(pos)).or_insert(0) += 1);
                    let v_basis = pauli.to_basis().expect("basis");
                    let t = 0..self.graph.incident_edges(v).count();
                    if !t.contains(counts.get(&v_basis.flipped()).unwrap()) {
                        panic!(
                            "X (Z) type vertex should have Pauli Z (X) Pauli supported on all or no edges, vertex at <pos> violates the rule."
                        )
                    }
                    if counts.get(&v_basis).unwrap() % 2 != 0 {
                        panic!(
                            "X (Z) type vertex should have even number of Pauli X (Z) supported on the edges, vertex at <pos> violates the rule."
                        )
                    }
                }
            }
        }

        Ok("Valid surface for this".to_string())
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
        (VType::B, _) => Ok(Pauli::I),
        _ => Err("Invalid vtype and phase {} {}"),
    }
}
