use std::{collections::HashSet, io::pipe};

use crate::{
    block_graph::BlockGraph,
    cube::{Basis, Cube, Pipe},
    direction::Direction3D,
};

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum SpatialArms {
    NONE,
    UP,
    RIGHT,
    DOWN,
    LEFT,
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

struct AbstractObservable {
    top_readout_cubes: HashSet<CubeWithArms>,
    top_readout_pipes: HashSet<PipeWithArms>,
    bottom_stabilizer_cubes: HashSet<CubeWithArms>,
    bottom_stabilizer_pipes: HashSet<PipeWithArms>,
    temporal_hadamard_pipes: HashSet<PipeWithObservableBasis>,
}

impl AbstractObservable {
    pub fn new(
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
