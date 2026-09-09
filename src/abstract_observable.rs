use std::{
    collections::{HashMap, HashSet},
    io::pipe,
};

use bitflags::bitflags;

use crate::{
    block_graph::BlockGraph,
    correlation::{CorrelationSurface, ZXEdge},
    cube::{Basis, Cube, CubeKind, Direction3D, Pipe, Position3D, ZXCube},
    positioned::PositionedZX,
};

bitflags! {
    #[derive(Hash, PartialEq, Eq, Clone, Copy)]
    struct SpatialArms: u8 {
        const NONE  = 0;
        const UP    = 1 << 0;
        const RIGHT = 1 << 1;
        const DOWN  = 1 << 2;
        const LEFT  = 1 << 3;
    }
}

impl SpatialArms {
    pub fn get_map_from_arm_shift() -> HashMap<SpatialArms, (i32, i32)> {
        HashMap::from([
            (Self::UP, (0, -1)),
            (Self::RIGHT, (1, 0)),
            (Self::DOWN, (0, 1)),
            (Self::LEFT, (-1, 0)),
        ])
    }
    pub fn single_arms() -> Vec<Self> {
        vec![Self::UP, Self::RIGHT, Self::DOWN, Self::LEFT]
    }

    pub fn length(&self) -> usize {
        Self::single_arms()
            .iter()
            .map(|t| if self.contains(*t) { 1 } else { 0 })
            .sum()
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct CubeWithArms {
    cube: Cube,
    arms: SpatialArms,
}

impl CubeWithArms {
    pub fn new(cube: Cube, arms: SpatialArms) -> Result<Self, String> {
        if arms != SpatialArms::NONE && !cube.is_spatial() {
            Err(
                "The `arms` attribute should be `SpatialArms::NONE` for non-spatial cubes."
                    .to_string(),
            )
        } else {
            Ok(Self {
                arms: arms,
                cube: cube,
            })
        }
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct PipeWithArms {
    pipe: Pipe,
    cube_arms: (SpatialArms, SpatialArms),
}

impl PipeWithArms {
    pub fn new(pipe: Pipe, cube_arms: (SpatialArms, SpatialArms)) -> Self {
        Self {
            pipe: pipe,
            cube_arms: cube_arms,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct PipeWithObservableBasis {
    pipe: Pipe,
    observable_basis: Basis,
}

impl PipeWithObservableBasis {
    pub fn new(pipe: Pipe, observable_basis: Basis) -> Result<Self, String> {
        if !pipe.has_hadamard() || pipe.direction().expect("Valid pipe direction") != Direction3D::Z
        {
            Err("Pipes must be temporal Hadamard.".to_string())
        } else {
            Ok(Self {
                pipe: pipe,
                observable_basis: observable_basis,
            })
        }
    }
}

pub struct AbstractObservable {
    top_readout_cubes: HashSet<CubeWithArms>,
    top_readout_pipes: HashSet<PipeWithArms>,
    bottom_stabilizer_cubes: HashSet<CubeWithArms>,
    bottom_stabilizer_pipes: HashSet<PipeWithArms>,
    temporal_hadamard_pipes: HashSet<PipeWithObservableBasis>,
}

impl AbstractObservable {
    pub fn new(
        // TODO: constructor may be redundant
        top_readout_cubes: HashSet<CubeWithArms>,
        top_readout_pipes: HashSet<PipeWithArms>,
        bottom_stabilizer_cubes: HashSet<CubeWithArms>,
        bottom_stabilizer_pipes: HashSet<PipeWithArms>,
        temporal_hadamard_pipes: HashSet<PipeWithObservableBasis>,
    ) -> Self {
        Self {
            top_readout_cubes: top_readout_cubes,
            top_readout_pipes: top_readout_pipes,
            bottom_stabilizer_cubes: bottom_stabilizer_cubes,
            bottom_stabilizer_pipes: bottom_stabilizer_pipes,
            temporal_hadamard_pipes: temporal_hadamard_pipes,
        }
    }

    // NB: in tqec-py, cube edges are an intrinsic part of the pipe objects themselves
    // I elected to encode this in the graph itself, so any reference to pipe objects
    // must include that.
    // Which is already the case when AbstractObservable is produced anyway ...
    pub fn slice_at_z(&self, graph: BlockGraph, z: i32) -> Self {
        let top_readout_cubes = self
            .top_readout_cubes
            .iter()
            .filter(|cube| cube.cube.position().z() == z)
            .clone()
            .map(|c| *c)
            .collect::<HashSet<CubeWithArms>>();
        let top_readout_pipes = self
            .top_readout_pipes
            .iter()
            .filter(|pipe| graph.spanning_cubes_of(&pipe.pipe).0.position().z() == z)
            .clone()
            .map(|c| *c)
            .collect::<HashSet<PipeWithArms>>();
        let bottom_stabilizer_cubes = self
            .bottom_stabilizer_cubes
            .iter()
            .filter(|cube| cube.cube.position().z() == z)
            .clone()
            .map(|c| *c)
            .collect::<HashSet<CubeWithArms>>();
        let bottom_stabilizer_pipes = self
            .bottom_stabilizer_pipes
            .iter()
            .filter(|pipe| graph.spanning_cubes_of(&pipe.pipe).0.position().z() == z)
            .clone()
            .map(|c| *c)
            .collect::<HashSet<PipeWithArms>>();
        let temporal_hadamard_pipes = self
            .temporal_hadamard_pipes
            .iter()
            .filter(|pipe| graph.spanning_cubes_of(&pipe.pipe).0.position().z() == z)
            .clone()
            .map(|c| *c)
            .collect::<HashSet<PipeWithObservableBasis>>();
        Self {
            top_readout_cubes: top_readout_cubes,
            top_readout_pipes: top_readout_pipes,
            bottom_stabilizer_cubes: bottom_stabilizer_cubes,
            bottom_stabilizer_pipes: bottom_stabilizer_pipes,
            temporal_hadamard_pipes: temporal_hadamard_pipes,
        }
    }
}

pub fn compile_correlation_surface_to_abstract_observable(
    block_graph: BlockGraph,
    correlation_surface: CorrelationSurface,
    include_temporal_hadamard_pipes: bool,
) -> AbstractObservable {
    if correlation_surface.is_single_node() {
        let cube = (*block_graph.cubes().get(0).expect("Cube")).clone();
        let cube_with_arms = CubeWithArms::new(cube, SpatialArms::NONE).expect("msg");
        if cube.is_spatial() {
            return AbstractObservable {
                top_readout_cubes: HashSet::new(),
                top_readout_pipes: HashSet::new(),
                bottom_stabilizer_cubes: HashSet::from([cube_with_arms]),
                bottom_stabilizer_pipes: HashSet::new(),
                temporal_hadamard_pipes: HashSet::new(),
            };
        } else {
            return AbstractObservable {
                top_readout_cubes: HashSet::from([cube_with_arms]),
                top_readout_pipes: HashSet::new(),
                bottom_stabilizer_cubes: HashSet::new(),
                bottom_stabilizer_pipes: HashSet::new(),
                temporal_hadamard_pipes: HashSet::new(),
            };
        }
    }

    let pg = block_graph.to_zx_graph();
    let valid = pg.correlation_surface_is_valid(&correlation_surface); // TODO: make use of this result ...

    let mut endpoints_to_edge: HashMap<[Position3D; 2], Vec<&ZXEdge>> = HashMap::new(); // FIXME: keys need to be hash sets for lookup later.

    correlation_surface.span().for_each(|edge| {
        let (u, v) = edge.nodes();
        let mut endpoints = [*u.position(), *v.position()];
        endpoints.sort();
        endpoints_to_edge.entry(endpoints).or_default().push(edge);
    });

    let mut top_readout_cubes: HashSet<CubeWithArms> = HashSet::new();
    let mut top_readout_pipes: HashSet<PipeWithArms> = HashSet::new();
    let mut bottom_stabilizer_pipes: HashSet<PipeWithArms> = HashSet::new();
    let mut temporal_hadamard_pipes: HashSet<PipeWithObservableBasis> = HashSet::new();

    for pos in correlation_surface.positions() {
        let cube = block_graph.cube_at(pos);
        if !cube.is_spatial() {
            continue;
        }
        let kind = cube.kind();
        match kind {
            CubeKind::ZX(zx_cube) => {
                let bases = correlation_surface.bases_at(pos);
                let normal_basis = zx_cube.normal_basis();
                if !(bases == HashSet::from([normal_basis.flipped()])) {
                    let mut arms = SpatialArms::NONE;
                    for (arm, (dx, dy)) in SpatialArms::get_map_from_arm_shift() {
                        let mut _key = [cube.position(), cube.position().shift_by(dx, dy, 0)];
                        _key.sort();
                        let edges = endpoints_to_edge.get(&_key);
                        if edges.is_some()
                            && edges
                                .unwrap()
                                .iter()
                                .map(|e| {
                                    let (u, v) = e.nodes();
                                    u.basis() == normal_basis && v.basis() == normal_basis
                                })
                                .any(|x| x)
                        {
                            arms |= arm;
                        }
                    }
                    assert!(arms.length() == 2 || arms.length() == 4);
                    if arms.length() == 4 {
                        top_readout_cubes.insert(
                            CubeWithArms::new(cube.clone(), SpatialArms::LEFT | SpatialArms::DOWN)
                                .unwrap(),
                        );
                        top_readout_cubes.insert(
                            CubeWithArms::new(cube.clone(), SpatialArms::RIGHT | SpatialArms::UP)
                                .unwrap(),
                        );
                    } else {
                        top_readout_cubes.insert(CubeWithArms::new(cube.clone(), arms).unwrap());
                    }
                }
            }
            _ => panic!("Cube kind must be ZX"),
        }
    }

    // TODO: handle the pipes


    AbstractObservable {
        top_readout_cubes: top_readout_cubes,
        top_readout_pipes: top_readout_pipes,
        bottom_stabilizer_cubes: HashSet::new(),
        bottom_stabilizer_pipes: bottom_stabilizer_pipes,
        temporal_hadamard_pipes: temporal_hadamard_pipes,
    }
}


fn has_obs_include(cube: Cube, correlation: Basis, block_graph: BlockGraph) -> bool {
  // Check if the top data qubit readout should be included in the observable.
  if cube.kind() == CubeKind::YHalfCube {
    return true;
  }
  // No pipe at the top
  match cube.kind() {
    CubeKind::ZX(zx_cube) => {
      if block_graph.has_pipe_between(cube.position(), cube.position().shift_by(0, 0, 1)) {
        return false;
      }
      return zx_cube.as_tuple().2 == correlation
    },
    _ => panic!("Must be ZX cube.")
  }
}
