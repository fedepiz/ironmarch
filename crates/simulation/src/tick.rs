use util::arena::Arena;

use crate::object::*;
use crate::simulation::*;
use crate::view;
use spatial::geom::Extents;

#[derive(Default)]
pub struct TickRequest {
    pub num_ticks: usize,
    pub map_viewport: Extents,
    pub objects_to_extract: Vec<ObjectId>,
}

pub(super) fn tick(sim: &mut Simulation, request: TickRequest, arena: &Arena) -> view::SimView {
    // Extract view
    view::extract(sim, request.map_viewport, &request.objects_to_extract)
}
