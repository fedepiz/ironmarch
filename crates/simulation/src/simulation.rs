use util::arena::*;

use crate::entities::{Entities, EntityId};
use crate::sites::*;
use crate::tick::TickRequest;

#[derive(Default)]
pub struct Simulation {
    pub(crate) turn_number: usize,
    pub(crate) sites: Sites,
    pub(crate) entities: Entities,
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
