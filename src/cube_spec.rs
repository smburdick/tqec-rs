use std::collections::HashSet;

use crate::abstract_observable::SpatialArms;
use crate::block_graph::BlockGraph;
use crate::cube::{Cube, CubeKind};
use crate::types::coord;

pub struct CubeSpec {
    kind: CubeKind,
    spatial_arms: SpatialArms,
    has_spatial_up_or_down_pipe_in_timeslice: bool,
}

impl CubeSpec {
    pub fn from_cube(
        cube: Cube,
        graph: &BlockGraph,
        spatial_up_or_down_pipe_slices: HashSet<coord>,
    ) -> Self {
        Self {
            kind: cube.kind(),
            spatial_arms: if !cube.is_spatial() {
                SpatialArms::NONE
            } else {
                SpatialArms::from_cube_in_graph(&cube, graph)
            },
            has_spatial_up_or_down_pipe_in_timeslice: spatial_up_or_down_pipe_slices
                .contains(&cube.position().z()),
        }
    }
}
