use quizx::{
    graph::{GraphLike, V},
    vec_graph::Graph,
};

use crate::{
    block_graph::BlockGraph,
    cube::{Basis, CubePosition},
    pauli::Pauli,
    positioned::PositionedZX,
    utils::{concat_ints_as_bits, solve_linear_system, zx_to_pauli},
};
use core::fmt;
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    iter::{self, repeat},
    mem::take,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd)]
pub struct ZXNode {
    position: CubePosition,
    basis: Basis,
}

impl fmt::Display for ZXNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.position, self.basis)
    }
}

impl ZXNode {
    pub fn new(position: CubePosition, basis: Basis) -> Self {
        Self {
            position: position,
            basis: basis,
        }
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
}

impl fmt::Display for ZXEdge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}, {}>", self.u, self.v)
    }
}

#[derive(Debug)]
pub struct CorrelationSurface {
    edges: HashSet<ZXEdge>, // TODO: rename this to 'span'
}

impl CorrelationSurface {
    pub fn new(edges: HashSet<ZXEdge>) -> Self {
        Self {
            edges: edges.clone(),
        }
    }

    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }

    pub fn is_single_node(&self) -> bool {
        self.num_edges() == 1 && self.edges.iter().next().unwrap().is_self_loop()
    }

    pub fn external_stabilizer_on_graph(&self, graph: BlockGraph) -> String {
        let supports: Vec<CubePosition>;
        if graph.is_open() {
            todo!("");
        } else {
            supports = graph.leaves().collect();
        }
        self.external_stabilizer(supports)
    }

    pub fn external_stabilizer(&self, io_ports: Vec<CubePosition>) -> String {
        let mut view: HashMap<CubePosition, HashSet<Basis>> = self.graph_view().1;
        io_ports
            .iter()
            .map(|port| {
                let bases = view.entry(*port).or_default();
                Pauli::from_basis_set(take(bases)).to_string()
            })
            .collect::<String>()
    }

    pub fn graph_view(
        &self,
    ) -> (
        HashMap<CubePosition, HashMap<CubePosition, Vec<ZXEdge>>>,
        HashMap<CubePosition, HashSet<Basis>>,
    ) {
        let mut edges = HashMap::new();
        let mut bases = HashMap::new();
        if self.is_single_node() {
            let edge = self
                .edges
                .iter()
                .next()
                .expect("Single node CS missing edge");
            let pos = edge.u.position;
            edges.insert(pos, HashMap::from([(pos, vec![edge.clone()])]));
            bases.insert(pos, HashSet::from([edge.u.basis]));
        } else {
            for edge in self.edges.iter() {
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

    // FIXME: this is returning incorrect results. :-/
    pub fn validate_node(
        &self,
        node: V,
        basis: Pauli,
        has_unconnected_neighbors: bool,
    ) -> (Option<Pauli>, Option<bool>, Option<usize>) {
        let paulis: Vec<Pauli> = self.paulis_at_nodes(iter::once(node)).collect();
        if paulis.len() == 0 {
            return (None, None, None);
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
            return (Some(broadcast_pauli), Some(passthru_parity), None);
        } else {
            return (
                None,
                None,
                Some(concat_ints_as_bits(
                    syndrome.iter().map(|&b| b as usize),
                    repeat(1),
                )),
            );
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

    // pub fn product_of_disconnected_surfaces(cs_list: Vec<Vec<HalfEdgeCorrelationSurface>>) -> Vec<CorrelationSurfaceView> {
    //   // Find Cartesian product of
    // }

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
            CorrelationSurface::new(set);
        }
        let mut span: Vec<ZXEdge> = Vec::new();
        let mut zx_nodes: HashMap<(usize, Basis), ZXNode> = HashMap::new();
        let bases = vec![Basis::X, Basis::Z];
        for (u, v, _) in graph.edges() {
            // TODO: use alternatives to unwrap.
            // FIXME: program crashes at this line.
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
            let _vec = Pauli::vec_ixyz();
            let product: Vec<(Pauli, Pauli)> = _vec
                .iter()
                .flat_map(|&x| _vec.iter().map(move |&y| (x, y)))
                .collect();
            for (xz_u, xz_v) in product {
                if (edge_is_hadamard ^ (xz_u == xz_v)) && xz_u == pauli_u && xz_v == pauli_v {
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

// FIXME: check this function ... seems like results have been different.
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
    for (i, out_paulis) in generate_valid_local_paulis(
        node_basis,
        broadcast_pauli,
        passthrough_parity,
        unconnected_neighbors.len(),
        generate_all,
    )
    .iter()
    .enumerate()
    {
        // FIXME: this method doesn't really make a lot of sense, I am probably misunderstanding it
        let mut new_correlation_surface = correlation_surface.clone();
        // if i != 0 || always_copy {
        //   new_correlation_surface.mapping.insert(node, new_correlation_surface.mapping.get(&node).unwrap().clone());
        // }
        for (n, pauli, edge_is_hadamard) in unconnected_neighbors
            .iter()
            .zip(out_paulis.iter())
            .zip(edges_are_hadamard.iter())
            .map(|((x, y), z)| (x, y, z))
        {
            if (i > 0 || always_copy) && correlation_surface.mapping.contains_key(n) {
                new_correlation_surface
                    .mapping
                    .insert(*n, correlation_surface.mapping.get(n).unwrap().clone());
            }
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
    println!("CSes = {:#?}", correlation_surfaces);
    let mut leaves: HashMap<Pauli, Vec<V>> = HashMap::new();
    for p in Pauli::vec_ixyz() {
        leaves.insert(p, Vec::new());
    }
    let vertices: Vec<V> = zx_graph
        .vertices()
        .filter(|v| zx_graph.degree(*v) == 1)
        .collect();
    for v in vertices.iter().sorted() {
        let mut key: Pauli = zx_to_pauli(zx_graph, *v).flipped(true);
        leaves.get_mut(&key).unwrap().push(*v);
    }

    let open_leaves: bool = leaves.get(&Pauli::I).unwrap().len() > 0;
    leaves.remove_entry(&Pauli::I);

    if leaves.values().map(|m| m.len()).sum::<usize>() > 0 {
        let sigfunc = |cs: &HalfEdgeCorrelationSurface| {
            concat_ints_as_bits(
                leaves.clone().iter().map(|(pauli, _leaves)| {
                    cs.signature_at_nodes(
                        _leaves.iter().sorted().map(|l| *l),
                        |p: Pauli| (p != *pauli && p != Pauli::I) as usize,
                        1,
                    )
                }),
                leaves.values().sorted().map(|l| l.len() as usize),
            )
        };
        // FIXME: basis surfaces being [] means they can't be referenced, need to check against the python version.
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

    if open_leaves {
        todo!("")
    }
    correlation_surfaces
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
