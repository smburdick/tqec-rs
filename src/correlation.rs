use quizx::{
    graph::{GraphLike, V},
    vec_graph::Graph,
};

use crate::{
    block_graph::BlockGraph,
    cube::{Basis, Position3D},
    pauli::Pauli,
    positioned::PositionedZX,
    types::coord,
    utils::{concat_ints_as_bits, int_to_bit_indices, solve_linear_system, zx_to_pauli},
};
use core::fmt;
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    iter::{self, repeat},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd)]
pub struct ZXNode {
    position: Position3D,
    basis: Basis,
}

impl fmt::Display for ZXNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.position, self.basis)
    }
}

impl ZXNode {
    pub fn new(position: Position3D, basis: Basis) -> Self {
        Self {
            position: position,
            basis: basis,
        }
    }
    pub fn position(&self) -> &Position3D {
        &self.position
    }
    pub fn basis(&self) -> Basis {
        self.basis
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ZXEdge {
    u: ZXNode,
    v: ZXNode,
}

impl ZXEdge {
    pub fn new(u: ZXNode, v: ZXNode) -> Self {
        Self { u: u, v: v }
    }

    pub fn sorted(&self) -> Self {
        if self.u < self.v {
            *self
        } else {
            ZXEdge::new(self.v, self.u)
        }
    }

    pub fn is_self_loop(&self) -> bool {
        return self.u.position == self.v.position;
    }

    pub fn nodes(&self) -> (&ZXNode, &ZXNode) {
        (&self.u, &self.v)
    }

    pub fn get_basis(&self, position: Position3D) -> Basis {
        let u_pos = self.u.position;
        let v_pos = self.v.position;
        match position {
            u_pos => self.u.basis,
            v_pos => self.v.basis,
            _ => panic!("Invalid basis"),
        }
    }
}

impl fmt::Display for ZXEdge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}, {}>", self.u, self.v)
    }
}

#[derive(Debug, Clone)]
pub struct CorrelationSurface {
    edges: HashSet<ZXEdge>, // TODO: rename this to 'span'
    graph_view: (
        HashMap<Position3D, HashMap<Position3D, Vec<ZXEdge>>>,
        HashMap<Position3D, HashSet<Basis>>,
    ), // used in helper functions
}

impl CorrelationSurface {
    pub fn new(edges: HashSet<ZXEdge>) -> Self {
        Self {
            edges: edges.clone(),
            graph_view: Self::graph_view(edges),
        }
    }

    pub fn span(&self) -> impl Iterator<Item = &ZXEdge> {
        self.edges.iter()
    }

    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }

    pub fn is_single_node(&self) -> bool {
        self.num_edges() == 1 && self.edges.iter().next().unwrap().is_self_loop()
    }

    pub fn bases_at(&self, position: Position3D) -> HashSet<Basis> {
        self.graph_view
            .1
            .get(&position)
            .expect("Failed to find position")
            .iter()
            .cloned()
            .collect::<HashSet<Basis>>()
    }

    pub fn external_stabilizer_on_graph(&self, graph: &BlockGraph) -> String {
        let supports: Vec<Position3D>;
        if graph.is_open() {
            supports = graph.ordered_port_positions()
        } else {
            supports = graph.leaves().collect();
        }
        self.external_stabilizer(supports)
    }

    pub fn external_stabilizer(&self, io_ports: Vec<Position3D>) -> String {
        let view: &HashMap<Position3D, HashSet<Basis>> = &self.graph_view.1;
        io_ports
            .iter()
            .map(|port| {
                let _bases = view.get(port);
                let bases: HashSet<Basis>;
                if _bases.is_none() {
                    bases = HashSet::new();
                } else {
                    bases = (*_bases.unwrap()).clone();
                }
                Pauli::from_basis_set(bases).to_string()
            })
            .collect::<String>()
    }

    pub fn positions(&self) -> HashSet<Position3D> {
        return self
            .graph_view
            .0
            .keys()
            .cloned()
            .collect::<HashSet<Position3D>>();
    }

    pub fn edges_at(&self, pos: Position3D) -> HashSet<ZXEdge> {
        self.graph_view
            .0
            .get(&pos)
            .expect("position")
            .values()
            .cloned()
            .flatten()
            .into_iter()
            .collect::<HashSet<ZXEdge>>()
    }

    pub fn shift_by(&self, dx: coord, dy: coord, dz: coord) -> Self {
        let mut nodes: HashMap<ZXNode, ZXNode> = HashMap::new();
        for position in self.positions() {
            let new_position =
                Position3D::new(position.x() + dx, position.y() + dy, position.z() + dz);
            for basis in self.bases_at(position) {
                let old_node = ZXNode::new(position, basis);
                let new_node = ZXNode::new(new_position, basis);
                nodes.insert(old_node, new_node);
            }
        }
        let edges = self
            .edges
            .iter()
            .map(|edge| {
                ZXEdge::new(
                    *nodes.get(&edge.u).expect("msg"),
                    *nodes.get(&edge.v).expect("msg"),
                )
                .sorted()
            })
            .collect::<HashSet<ZXEdge>>();
        CorrelationSurface::new(edges)
    }

    fn graph_view(
        _edges: HashSet<ZXEdge>,
    ) -> (
        HashMap<Position3D, HashMap<Position3D, Vec<ZXEdge>>>,
        HashMap<Position3D, HashSet<Basis>>,
    ) {
        let mut edges = HashMap::new();
        let mut bases = HashMap::new();
        if _edges.len() == 1 {
            //&& _edges.iter().next().is_self_loop() {
            let edge = _edges.iter().next().expect("Single node CS missing edge");
            let pos = edge.u.position;
            edges.insert(pos, HashMap::from([(pos, vec![edge.clone()])]));
            bases.insert(pos, HashSet::from([edge.u.basis]));
        } else {
            for edge in _edges.iter() {
                let (u, v) = (edge.u.position, edge.v.position);
                edges
                    .entry(u)
                    .or_default()
                    .entry(v)
                    .or_default()
                    .push(edge.clone());
                edges
                    .entry(v)
                    .or_default()
                    .entry(u)
                    .or_default()
                    .push(edge.clone());
                bases.entry(u).or_default().insert(edge.u.basis);
                bases.entry(v).or_default().insert(edge.v.basis);
            }
        }
        (edges, bases)
    }
}

impl fmt::Display for CorrelationSurface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.edges)
    }
}

#[derive(Clone, Debug)]
pub struct HalfEdgeCorrelationSurface {
    pub mapping: HashMap<V, HashMap<V, Pauli>>,
}

pub enum ValidationResult {
    None,
    Single(usize),
    Pair(Pauli, bool),
}

impl HalfEdgeCorrelationSurface {
    pub fn new() -> Self {
        Self {
            mapping: HashMap::new(),
        }
    }

    pub fn is_single_node(&self) -> bool {
        self.mapping.keys().len() == 1 && self.mapping.values().len() == 1
    }

    pub fn add_pauli_to_edge(&mut self, edge: (V, V), pauli: Pauli, edge_is_hadamard: bool) {
        let (u, v) = edge;
        for (from, to, p) in [(u, v, pauli), (v, u, pauli.flipped(edge_is_hadamard))] {
            self.mapping.entry(from).or_default().insert(to, p);
        }
    }

    pub fn validate_node(
        &self,
        node: V,
        basis: Pauli,
        has_unconnected_neighbors: bool,
    ) -> ValidationResult {
        let paulis: Vec<Pauli> = self.paulis_at_nodes(iter::once(node)).collect();
        if paulis.len() == 0 {
            return ValidationResult::None;
        }

        let passthru_parity = paulis
            .iter()
            .copied()
            .reduce(|acc, p| acc.xor(p))
            .expect("Passthru parity")
            .contains(basis);

        let mut valid = true;
        let broadcast_basis = basis.flipped(true);
        let mut syndrome: Vec<bool> = paulis.iter().map(|p| p.contains(broadcast_basis)).collect();
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
            return ValidationResult::Pair(broadcast_pauli, passthru_parity);
        } else {
            return ValidationResult::Single(concat_ints_as_bits(
                syndrome.iter().map(|&b| b as usize),
                repeat(1),
            ));
        }
    }

    pub fn paulis_at_nodes(&self, nodes: impl Iterator<Item = V>) -> impl Iterator<Item = Pauli> {
        nodes
            .into_iter()
            .map(|v| {
                self.mapping
                    .get(&v)
                    .expect(&format!("Missing mapping for vertex {}", v))
                    .values()
            })
            .flatten()
            .map(|&p| p)
    }

    pub fn signature_at_nodes<F>(
        &self,
        nodes: impl Iterator<Item = V>,
        func: F,
        bit_length: usize,
    ) -> usize
    where
        F: Fn(Pauli) -> usize,
    {
        let paulis = self.paulis_at_nodes(nodes);
        let ints = paulis.map(|x| func(x));
        concat_ints_as_bits(ints, repeat(bit_length))
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
                    let neighbor_row = cs.mapping.get(v).expect("vertex missing from mapping");

                    let other_pauli = neighbor_row.get(n).expect("neighbor missing from mapping");

                    res_pauli = res_pauli.xor(*other_pauli);
                }
                val.insert(*n, res_pauli);
            }
            result.mapping.insert(*v, val);
        }
        result
    }

    pub fn to_immutable_public_representation(&self, graph: &PositionedZX) -> CorrelationSurface {
        if self.is_single_node() {
            let u_id = self.mapping.iter().next().unwrap().0;
            let (v_id, pauli) = self.mapping.get(u_id).unwrap().iter().next().unwrap();
            assert!(u_id == v_id);
            let cube = graph.get_cube_at(*u_id).unwrap();
            let node = ZXNode::new(cube.position(), pauli.to_basis().unwrap());
            let edge: ZXEdge = ZXEdge::new(node, node);
            let mut set: HashSet<ZXEdge> = HashSet::new();
            set.insert(edge);
            return CorrelationSurface::new(set);
        }
        let mut span: Vec<ZXEdge> = Vec::new();
        let mut zx_nodes: HashMap<(usize, Basis), ZXNode> = HashMap::new();
        let bases = vec![Basis::X, Basis::Z];
        for (u, v, _) in graph.edges() {
            let pauli_u = *self
                .mapping
                .get(&u)
                .expect(&format!("Pauli map of node {}", u))
                .get(&v)
                .expect(&format!("Pauli corresponding to {} -> {}", u, v));
            let pauli_v = *self.mapping.get(&v).unwrap().get(&u).unwrap();
            let edge_is_hadamard = graph.edge_is_hadamard((u, v));
            let pos_u = graph.get_cube_at(u).unwrap().position();
            let pos_v = graph.get_cube_at(v).unwrap().position();
            let _vec = [Pauli::X, Pauli::Z];
            let product: Vec<(Pauli, Pauli)> = _vec
                .iter()
                .flat_map(|&x| _vec.iter().map(move |&y| (x, y)))
                .collect();
            for (xz_u, xz_v) in product {
                if (edge_is_hadamard ^ (xz_u == xz_v))
                    && pauli_u.contains(xz_u)
                    && pauli_v.contains(xz_v)
                {
                    let basis_u = bases[(xz_u.value() >> 1) as usize];
                    let basis_v = bases[(xz_v.value() >> 1) as usize];

                    let node_u = zx_nodes
                        .entry((u, basis_u))
                        .or_insert_with(|| ZXNode::new(pos_u, basis_u))
                        .clone();

                    let node_v = zx_nodes
                        .entry((v, basis_v))
                        .or_insert_with(|| ZXNode::new(pos_v, basis_v))
                        .clone();

                    span.push(ZXEdge::new(node_u, node_v).sorted());
                }
            }
        }
        CorrelationSurface::new(span.into_iter().collect::<HashSet<ZXEdge>>())
    }
}

pub fn generate_valid_local_paulis(
    node_basis: Pauli,
    broadcast_pauli: Pauli,
    passthrough_parity: bool,
    num_unconnected_neighbors: usize,
    generate_all: bool,
) -> Vec<Vec<Pauli>> {
    let mut result: Vec<Vec<Pauli>> = Vec::new();
    let unconnected_neighbors = 0..num_unconnected_neighbors;
    let combined_pauli = broadcast_pauli.xor(node_basis);
    if generate_all {
        let passthru_nodes = ((passthrough_parity as usize)..(unconnected_neighbors.len() + 1))
            .step_by(2)
            .flat_map(|n| unconnected_neighbors.clone().combinations(n));

        result = passthru_nodes
            .map(|p| {
                unconnected_neighbors
                    .clone()
                    .map(|n| {
                        if p.contains(&n) {
                            combined_pauli
                        } else {
                            broadcast_pauli
                        }
                    })
                    .collect()
            })
            .collect();
    } else {
        todo!("Not implemented yet")
    }
    result
}

pub fn expand_correlation_surface_to_node(
    correlation_surface: &HalfEdgeCorrelationSurface,
    broadcast_pauli: Pauli,
    passthrough_parity: bool,
    node: V,
    node_basis: Pauli,
    unconnected_neighbors: &Vec<V>,
    edges_are_hadamard: &Vec<bool>,
    generate_all: bool,
    always_copy: bool,
) -> Vec<HalfEdgeCorrelationSurface> {
    // TODO: python version uses generator instead, consider using that.
    let mut new_correlation_surfaces: Vec<HalfEdgeCorrelationSurface> = Vec::new();
    for out_paulis in generate_valid_local_paulis(
        node_basis,
        broadcast_pauli,
        passthrough_parity,
        unconnected_neighbors.len(),
        generate_all,
    )
    .iter()
    {
        let mut new_correlation_surface = correlation_surface.clone();
        for ((n, pauli), edge_is_hadamard) in unconnected_neighbors
            .iter()
            .zip(out_paulis.iter())
            .zip(edges_are_hadamard.iter())
        {
            new_correlation_surface.add_pauli_to_edge((node, *n), *pauli, *edge_is_hadamard);
        }
        new_correlation_surfaces.push(new_correlation_surface);
    }
    new_correlation_surfaces
}

pub fn reform_correlation_surface_generators<F>(
    correlation_surfaces: Vec<&HalfEdgeCorrelationSurface>,
    signature_func: F,
    stabilizer_basis: &mut HashMap<usize, (usize, usize)>,
    basis_surfaces: Vec<&HalfEdgeCorrelationSurface>,
    construct_new_surfaces: bool,     // = True,
    num_new_surfaces_needed: usize,   // | None = None,
    num_basis_surfaces_needed: usize, //int | None = None,
) -> (
    Vec<HalfEdgeCorrelationSurface>,
    Vec<HalfEdgeCorrelationSurface>,
)
where
    F: Fn(&HalfEdgeCorrelationSurface) -> usize,
{
    let mut new_basis_surfaces: Vec<HalfEdgeCorrelationSurface> =
        basis_surfaces.iter().map(|cs| (*cs).clone()).collect();

    let mut new_surfaces: Vec<HalfEdgeCorrelationSurface> = Vec::new();
    for cs in correlation_surfaces {
        let x = signature_func(cs);
        let indices = solve_linear_system(stabilizer_basis, x, true);
        if indices.is_err() {
            new_basis_surfaces.push(cs.clone());
            if num_basis_surfaces_needed > 0 && basis_surfaces.len() > num_basis_surfaces_needed {
                break;
            }
            continue;
        }
        if construct_new_surfaces {
            let _bscs = indices
                .unwrap()
                .iter()
                .map(|k| &new_basis_surfaces[*k])
                .chain(std::iter::once(cs))
                .collect();
            let _new_cs = HalfEdgeCorrelationSurface::xor(_bscs);
            new_surfaces.push(_new_cs);
            if num_new_surfaces_needed > 0 && new_surfaces.len() >= num_new_surfaces_needed {
                break;
            }
        }
    }
    (new_basis_surfaces, new_surfaces)
}

pub fn find_correlation_surfaces_from_leaf(
    zx_graph: &Graph,
    leaf: V,
) -> Vec<HalfEdgeCorrelationSurface> {
    let mut correlation_surfaces =
        PositionedZX::find_correlation_surface_generating_set_from_leaf(zx_graph, leaf);

    let mut leaves: HashMap<Pauli, Vec<V>> = HashMap::new();
    for p in Pauli::vec_ixyz() {
        leaves.insert(p, Vec::new());
    }

    let vertices: Vec<V> = zx_graph
        .vertices()
        .filter(|v| zx_graph.degree(*v) == 1)
        .collect();

    for v in vertices.iter().sorted() {
        let key: Pauli = zx_to_pauli(zx_graph, *v).flipped(true);
        leaves.get_mut(&key).unwrap().push(*v);
    }

    let open_leaves = leaves.get(&Pauli::I).expect("msg").clone();
    leaves.remove_entry(&Pauli::I);

    if leaves.values().map(|m| m.len()).sum::<usize>() > 0 {
        let sigfunc = |cs: &HalfEdgeCorrelationSurface| {
            concat_ints_as_bits(
                leaves.iter().map(|(pauli, _leaves)| {
                    cs.signature_at_nodes(
                        _leaves.iter().map(|v| *v),
                        |p: Pauli| (p != *pauli && p != Pauli::I) as usize,
                        1,
                    )
                }),
                leaves.values().map(|l| l.len() as usize),
            )
        };
        correlation_surfaces = reform_correlation_surface_generators(
            correlation_surfaces.iter().collect(),
            sigfunc,
            &mut HashMap::new(),
            Vec::new(),
            true,
            0,
            0,
        )
        .1
    }

    if !open_leaves.is_empty() {
        let mut basis: HashMap<usize, (usize, usize)> = HashMap::new();
        construct_basis(
            &mut basis,
            &correlation_surfaces,
            |cs: &HalfEdgeCorrelationSurface| {
                cs.signature_at_nodes(open_leaves.iter().map(|l| *l), |p: Pauli| p.value(), 2)
            },
        );
        normalize_basis(&mut basis, true);
        correlation_surfaces = basis
            .values()
            .map(|(_, mask)| {
                let indices = int_to_bit_indices(*mask);
                if indices.len() > 1 {
                    HalfEdgeCorrelationSurface::xor(
                        indices
                            .iter()
                            .map(|i| correlation_surfaces.get(*i).expect("msg"))
                            .collect(),
                    )
                } else {
                    correlation_surfaces
                        .get(*indices.get(0).expect("msg"))
                        .expect("msg")
                        .clone()
                }
            })
            .collect();
    }

    correlation_surfaces
}

pub fn construct_basis<F>(
    basis: &mut HashMap<usize, (usize, usize)>,
    correlation_surfaces: &Vec<HalfEdgeCorrelationSurface>,
    func: F,
) where
    F: Fn(&HalfEdgeCorrelationSurface) -> usize,
{
    correlation_surfaces.iter().for_each(|cs| {
        solve_linear_system(basis, func(cs), true);
    });
}

pub fn normalize_basis(basis: &mut HashMap<usize, (usize, usize)>, in_place: bool) {
    if !in_place {
        todo!("Not implemented yet.")
    }
    let highest_bits: Vec<usize> = basis.keys().sorted().rev().map(|u| *u).collect();
    for (i, key) in highest_bits.iter().enumerate() {
        let (_v, _m) = basis.get(key).expect("msg");
        let mut vector = *_v;
        let mut mask = *_m;
        for highest_bit in &highest_bits[i + 1..] {
            if (vector >> highest_bit) & 1 != 0 {
                let (pivot, pivot_mask) = basis.get(highest_bit).expect("msg");
                vector ^= *pivot;
                mask ^= *pivot_mask;
            }
        }
        basis.insert(*key, (vector, mask));
    }
}

// pub struct Basis {
//   contents: Vec<usize, (usize, usize)>
// }

// impl Basis {

//   pub fn new() -> Self {

//   }

//   pub fn construct_from_items<F>(&mut self, items: Vec<dyn Any>, func: F) where F: Fn(dyn Any) -> usize {
//     todo!("")
//   }

// }
