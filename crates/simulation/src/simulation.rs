use util::arena::*;

use crate::aspects::Aspects;
use crate::entities::{Entities, EntityId};
use crate::sites::*;
use crate::spawn::*;
use crate::tick::TickRequest;

#[derive(Default)]
pub struct Simulation {
    pub(crate) turn_number: usize,
    pub(crate) aspects: Aspects,
    pub(crate) sites: Sites,
    pub(crate) prototypes: Prototypes,
    pub(crate) entities: Entities,
    pub(crate) interaction: Interaction,
    pub(crate) active_agent: EntityId,
}

impl Simulation {
    pub fn new(arena: &Arena) -> Simulation {
        let mut sim = Simulation::default();

        crate::init::init(&mut sim, arena, 2704);
        sim
    }

    pub fn tick(&mut self, request: TickRequest, arena: &Arena) -> crate::view::SimView {
        crate::tick::tick(self, request, arena)
    }
}

#[derive(Default)]
pub(crate) struct Interaction {
    pub selected_entity: EntityId,
    pub available_actions: AvailableActions,
}

#[derive(Default)]
pub(crate) struct Action {
    pub name: &'static str,
    pub spawn_prototype: Option<(Prototype, PrototypeArgs)>,
}

pub(crate) struct AvailableActions {
    pub has_any: bool,
    pub list: Vec<Action>,
    pub dummy: Action,
}

impl Default for AvailableActions {
    fn default() -> Self {
        Self {
            has_any: false,
            list: vec![],
            dummy: Action {
                name: "Dummy Action",
                ..Default::default()
            },
        }
    }
}

impl std::ops::Index<usize> for AvailableActions {
    type Output = Action;
    fn index(&self, index: usize) -> &Self::Output {
        self.list.get(index).unwrap_or(&self.dummy)
    }
}
