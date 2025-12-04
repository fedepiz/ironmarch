use slotmap::{SlotMap, new_key_type};
use util::arena::*;
use util::tagged::Tags;

use crate::sites::*;
use crate::tick::TickRequest;

#[derive(Default)]
pub struct Simulation {
    pub(crate) turn_number: usize,
    pub(crate) sites: Sites,
    pub(crate) entities: Entities,
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

new_key_type! { pub(crate) struct EntityId; }

#[derive(Default)]
pub(crate) struct EntityData {
    pub id: EntityId,
    pub name: String,

    /// Some entities are bound to a given site. This records the site
    /// the said entity is linked to
    pub bound_site: Option<SiteId>,
}

#[derive(Default)]
pub(crate) struct Entities {
    entries: SlotMap<EntityId, EntityData>,
    tags: Tags<EntityId>,
}

impl Entities {
    pub(crate) fn spawn(&mut self) -> &mut EntityData {
        self.spawn_with(EntityData::default())
    }

    pub(crate) fn spawn_with(&mut self, data: EntityData) -> &mut EntityData {
        let id = self.entries.insert(data);
        let data = &mut self.entries[id];
        data.id = id;
        data
    }

    pub(crate) fn despawn(&mut self, id: EntityId) {
        if let Some(_) = self.entries.remove(id) {
            self.tags.remove(&id);
        }
    }

    pub(crate) fn iter<'a>(&'a self) -> slotmap::basic::Iter<'a, EntityId, EntityData> {
        self.entries.iter()
    }

    pub(crate) fn iter_mut<'a>(&'a mut self) -> slotmap::basic::IterMut<'a, EntityId, EntityData> {
        self.entries.iter_mut()
    }

    pub(crate) fn values<'a>(&'a self) -> slotmap::basic::Values<'a, EntityId, EntityData> {
        self.entries.values()
    }
}

impl std::ops::Index<EntityId> for Entities {
    type Output = EntityData;

    fn index(&self, index: EntityId) -> &Self::Output {
        &self.entries[index]
    }
}

impl std::ops::IndexMut<EntityId> for Entities {
    fn index_mut(&mut self, index: EntityId) -> &mut Self::Output {
        &mut self.entries[index]
    }
}
