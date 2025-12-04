use util::arena::Arena;

use crate::object::*;
use crate::simulation::*;
use crate::view;
use crate::view::*;

#[derive(Default)]
pub struct TickRequest {
    pub num_ticks: usize,
    pub map_viewport: Extents,
    pub objects_to_extract: Vec<ObjectId>,
}

pub(super) fn tick(sim: &mut Simulation, request: TickRequest, arena: &Arena) -> SimView {
    // Extract view
    let mut view = SimView::default();
    view.map_items = view::map_view_items(sim, request.map_viewport);
    view.map_lines = view::map_view_lines(sim, request.map_viewport);
    view.objects = request
        .objects_to_extract
        .iter()
        .map(|&id| view::extract_object(sim, id))
        .collect();
    view
}
