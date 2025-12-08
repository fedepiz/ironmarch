use rand::SeedableRng;
use rand::rngs::SmallRng;
use slotmap::Key;
use spatial::geom::Extents;
use util::arena::Arena;
use util::tagged::TaggedCollection;

use crate::entities::*;
use crate::object::*;
use crate::simulation::*;
use crate::spawn::PrototypeArgs;
use crate::view;

#[derive(Default)]
pub struct TickRequest {
    pub view: ViewRequest,
    pub end_turn: bool,
    pub make_active: Option<ObjectId>,
    pub interacted_with_object: Option<ObjectId>,
}

#[derive(Default)]
pub struct ViewRequest {
    pub enabled: bool,
    pub map_viewport: Extents,
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
    determine_available_actions(sim);

    // Update interaction
    if let Some(object) = request.interacted_with_object {
        handle_interaction(sim, arena, object);
    }

    // Extract view
    if request.view.enabled {
        view::extract(
            sim,
            arena,
            request.view.map_viewport,
            sim.interaction.selected_entity,
        )
    } else {
        view::SimView::default()
    }
}

fn handle_interaction(sim: &mut Simulation, arena: &Arena, interacted_with: ObjectId) {
    let available_actions = std::mem::take(&mut sim.available_actions);

    let rng = &mut {
        let seed = (sim.turn_number as u64).wrapping_mul(13).wrapping_add(2732);
        SmallRng::seed_from_u64(seed)
    };
    // Update interaction
    match interacted_with.0 {
        ObjectHandle::Null => sim.interaction.selected_entity = EntityId::null(),
        ObjectHandle::Entity(id) => {
            sim.interaction.selected_entity = id;
        }
        ObjectHandle::AvailableAction(idx) => {
            let action = available_actions.list.into_iter().nth(idx).unwrap();

            println!("Performing action {}", action.name);
            if let Some((proto, args)) = action.spawn_prototype.as_ref() {
                proto.spawn(sim, arena, rng, args);
            }
        }
        _ => {}
    };
}

fn determine_available_actions(sim: &mut Simulation) {
    let subject = &sim.entities[sim.active_agent];
    let target = &sim.entities[sim.interaction.selected_entity];

    let mut actions = std::mem::take(&mut sim.available_actions);
    let has_subject_and_object = !subject.id.is_null() && !target.id.is_null();
    actions.has_any = has_subject_and_object;
    actions.list.clear();

    if has_subject_and_object {
        actions.list.push(Action {
            name: "Test Action",
            ..Default::default()
        });

        if target.flags.get(Flag::IsPerson) {
            actions.list.push(Action {
                name: "Kiss",
                ..Default::default()
            });
        }

        if target.flags.get(Flag::IsLocation) {
            let prototype = sim.prototypes.lookup("bonheddwr").unwrap_or_default();
            let args = PrototypeArgs {
                location: target.id,
                ..Default::default()
            };

            actions.list.push(Action {
                name: "Recruit",
                spawn_prototype: Some((prototype, args)),
            })
        }
    }
    sim.available_actions = actions;
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
