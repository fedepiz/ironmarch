use util::arena::*;

use crate::sites::*;
use crate::tick::TickRequest;

#[derive(Default)]
pub struct Simulation {
    pub(crate) turn_number: usize,
    pub(crate) sites: Sites,
}

impl Simulation {
    pub fn new() -> Simulation {
        let mut sim = Simulation::default();
        crate::init::init(&mut sim);
        sim
    }

    pub fn tick(&mut self, request: TickRequest, arena: &Arena) -> crate::view::SimView {
        crate::tick::tick(self, request, arena)
    }
}
