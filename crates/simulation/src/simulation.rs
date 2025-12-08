use strum::{EnumCount, EnumIter};
use util::arena::*;

use crate::entities::{Entities, EntityId};
use crate::sites::*;
use crate::tick::TickRequest;

#[derive(Default)]
pub struct Simulation {
    pub(crate) turn_number: usize,
    pub(crate) interaction: Interaction,
    pub(crate) sites: Sites,
    pub(crate) entities: Entities,
    pub(crate) active_agent: EntityId,
    pub(crate) available_actions: AvailableActions,
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
}

pub(crate) struct Action {
    pub kind: ActionKind,
    pub name: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, EnumIter, EnumCount)]
pub(crate) enum ActionKind {
    Null,
}

impl Default for ActionKind {
    fn default() -> Self {
        ActionKind::Null
    }
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
                kind: ActionKind::default(),
                name: "Dummy Action",
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
