use spatial::geom::Extents;
use util::arena::Arena;

use crate::entities::*;
use crate::object::*;
use crate::simulation::*;
use crate::view;

#[derive(Default)]
pub struct TickRequest {
    pub view: ViewRequest,
    pub end_turn: bool,
    pub make_active: Option<ObjectId>,
}

#[derive(Default)]
pub struct ViewRequest {
    pub enabled: bool,
    pub map_viewport: Extents,
    pub selected_object: ObjectId,
}

pub(super) fn tick(sim: &mut Simulation, request: TickRequest, arena: &Arena) -> view::SimView {
    if request.end_turn {
        sim.turn_number += 1;
    }

    sim.active_agent = request
        .make_active
        .and_then(|x| x.as_entity())
        .unwrap_or(sim.active_agent);

    refresh_colours(sim);

    // Extract view
    if request.view.enabled {
        view::extract(
            sim,
            arena,
            request.view.map_viewport,
            request.view.selected_object,
        )
    } else {
        view::SimView::default()
    }
}

fn refresh_colours(sim: &mut Simulation) {
    let mut updates = Vec::with_capacity(sim.entities.len());

    for entity in sim.entities.iter() {
        if !entity.color.dirty {
            continue;
        }

        let source = &sim.entities[entity.hierarchies.parent(HierarchyName::Faction)];
        let next_colour = source.color.current;
        let is_still_dirty = source.color.dirty;
        updates.push((entity.id, next_colour, is_still_dirty));
    }

    for (id, value, is_dirty) in updates {
        let color = &mut sim.entities[id].color;
        color.current = value;
        color.dirty = is_dirty;
    }
}
