use spatial::geom::Extents;
use util::arena::Arena;

use crate::object::*;
use crate::simulation::*;
use crate::view;

#[derive(Default)]
pub struct TickRequest {
    pub map_viewport: Extents,
    pub selected_object: ObjectId,
    pub end_turn: bool,
}

pub(super) fn tick(sim: &mut Simulation, request: TickRequest, arena: &Arena) -> view::SimView {
    if request.end_turn {
        sim.turn_number += 1;
    }
    // Extract view
    view::extract(sim, arena, request.map_viewport, request.selected_object)
}
