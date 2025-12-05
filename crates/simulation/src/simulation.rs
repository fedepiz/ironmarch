use slotmap::{Key, SlotMap, new_key_type};
use strum::{EnumCount, EnumIter};
use util::arena::*;
use util::tagged::{TaggedCollection, Tags};

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
    pub bound_site: SiteId,
    // Relations
    pub hierarchies: HierarchyLinks,
}

pub(crate) struct Entities {
    entries: SlotMap<EntityId, EntityData>,
    tags: Tags<EntityId>,
    dummy: EntityId,
}

impl Default for Entities {
    fn default() -> Self {
        Self::new()
    }
}

impl Entities {
    pub(crate) fn new() -> Self {
        let mut this = Self {
            entries: Default::default(),
            tags: Tags::default(),
            dummy: Default::default(),
        };
        let dummy = this.spawn();
        dummy.name = "NULL".to_string();
        this.dummy = dummy.id;
        this
    }

    pub(crate) fn spawn(&mut self) -> &mut EntityData {
        self.spawn_with(EntityData::default())
    }

    pub(crate) fn spawn_with(&mut self, data: EntityData) -> &mut EntityData {
        let id = self.entries.insert(data);
        let data = &mut self.entries[id];
        data.id = id;
        data
    }

    // pub(crate) fn despawn(&mut self, id: EntityId) {
    //     if let Some(_) = self.entries.remove(id) {
    //         self.tags.remove(&id);
    //     }
    // }

    // pub(crate) fn iter<'a>(&'a self) -> slotmap::basic::Iter<'a, EntityId, EntityData> {
    //     self.entries.iter()
    // }

    // pub(crate) fn iter_mut<'a>(&'a mut self) -> slotmap::basic::IterMut<'a, EntityId, EntityData> {
    //     self.entries.iter_mut()
    // }

    pub(crate) fn values<'a>(&'a self) -> slotmap::basic::Values<'a, EntityId, EntityData> {
        self.entries.values()
    }

    pub(crate) fn get_mut(&mut self, id: EntityId) -> Option<&mut EntityData> {
        self.entries.get_mut(id)
    }
}

impl std::ops::Index<EntityId> for Entities {
    type Output = EntityData;

    fn index(&self, index: EntityId) -> &Self::Output {
        if !index.is_null() {
            &self.entries[index]
        } else {
            &self.entries[self.dummy]
        }
    }
}

impl std::ops::IndexMut<EntityId> for Entities {
    fn index_mut(&mut self, index: EntityId) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl<'a> TaggedCollection for &'a Entities {
    type Output = &'a EntityData;
    fn lookup(&self, tag: &str) -> Option<Self::Output> {
        self.tags.lookup(tag).map(|id| &self.entries[id])
    }
}

// Parent-child relationships

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, EnumCount)]
pub(crate) enum HierarchyName {
    Faction,
}

#[derive(Default)]
pub(crate) struct HierarchyLink {
    pub parent: EntityId,
    pub children: Vec<EntityId>,
}

#[derive(Default)]
pub(crate) struct HierarchyLinks([HierarchyLink; HierarchyName::COUNT]);

impl HierarchyLinks {
    pub(crate) fn parent(&self, rel: HierarchyName) -> EntityId {
        self.0
            .get(rel as usize)
            .map(|x| x.parent)
            .unwrap_or_default()
    }

    pub(crate) fn children(&self, rel: HierarchyName) -> &[EntityId] {
        self.0
            .get(rel as usize)
            .map(|x| x.children.as_slice())
            .unwrap_or_default()
    }
}

/// Recursively search for the root of the entity in the hierarchy, returns a null entity if
/// the entity is not part of the hierarchy at all
pub(crate) fn root_of(entities: &Entities, rel: HierarchyName, entity: EntityId) -> EntityId {
    let mut id = entity;
    loop {
        let parent = entities[id].hierarchies.parent(rel);
        if parent.is_null() || parent == id {
            return parent;
        } else {
            id = parent;
        }
    }
}

pub(crate) fn set_root(entities: &mut Entities, rel: HierarchyName, entity: EntityId) {
    set_parent(entities, rel, entity, entity);
}

pub(crate) fn set_parent(
    entities: &mut Entities,
    rel: HierarchyName,
    child: EntityId,
    parent: EntityId,
) {
    // We can't operate on a null child
    if child.is_null() {
        return;
    }
    // Remove current parent
    unparent(entities, rel, child);

    // Add new parent
    let idx = rel as usize;
    let child_data = &mut entities[child];
    child_data.hierarchies.0[idx].parent = parent;

    // If parent is self or is a null, then stop here -- nothing to do
    if child == parent || parent.is_null() {
        return;
    }

    // Otherwise get the parent and update its child list
    let parent = entities.get_mut(parent).unwrap();
    let children = &mut parent.hierarchies.0[idx].children;
    match children.binary_search(&child) {
        Ok(idx) => children[idx] = child,
        Err(idx) => children.insert(idx, child),
    }
}

fn unparent(entities: &mut Entities, rel: HierarchyName, child: EntityId) {
    assert!(!child.is_null());
    let child_data = &mut entities[child];

    // Get the parent, give up if there is nothing to do
    let parent = child_data.hierarchies.parent(rel);
    if parent.is_null() {
        return;
    }

    // Neutralize child and parent fields
    let idx = rel as usize;
    child_data.hierarchies.0[idx].parent = EntityId::default();

    let parent = entities.get_mut(parent).unwrap();
    parent.hierarchies.0[idx].children.retain(|&x| x != child);
}
