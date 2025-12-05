use crate::sites::SiteId;
use slotmap::*;
use strum::{EnumCount, EnumIter, IntoEnumIterator};
use tinybitset::TinyBitSet;
use util::{
    arena::ArenaSafe,
    tagged::{TaggedCollection, Tags},
};

macro_rules! get_or_return {
    ($e:expr) => {
        match $e {
            Some(x) => x,
            _ => {
                return;
            }
        }
    };
}

trait VecExt<T> {
    fn sorted_insert(&mut self, item: T);
}

impl<T: Ord> VecExt<T> for Vec<T> {
    #[inline]
    fn sorted_insert(&mut self, element: T) {
        match self.binary_search(&element) {
            Ok(idx) => {
                self[idx] = element;
            }
            Err(idx) => {
                self.insert(idx, element);
            }
        }
    }
}

new_key_type! { pub(crate) struct EntityId; }

impl ArenaSafe for EntityId {}

#[derive(Default)]
pub(crate) struct EntityData {
    pub id: EntityId,
    pub name: String,
    pub kind_name: &'static str,
    /// Some entities are bound to a given site. This records the site
    /// the said entity is linked to
    pub bound_site: SiteId,
    // Relations
    /// Parent-child relations
    pub hierarchies: HierarchyLinks,
    /// Sprite
    pub sprite: &'static str,
    /// Set of flags
    pub flags: Flags,
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
        let id = self.entries.insert(EntityData::default());
        let data = &mut self.entries[id];
        data.id = id;
        data.kind_name = "UNKNOWN_KIND";
        data
    }

    pub(crate) fn despawn(&mut self, id: EntityId) {
        for hierarchy in HierarchyName::iter() {
            self.remove_all_children(hierarchy, id);
            self.unparent(hierarchy, id);
        }

        if let Some(_) = self.entries.remove(id) {
            self.tags.remove(&id);
        }
    }

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

impl Entities {
    /// Recursively search for the root of the entity in the hierarchy, returns a null entity if
    /// the entity is not part of the hierarchy at all
    pub(crate) fn root_of(&self, rel: HierarchyName, entity: EntityId) -> EntityId {
        let mut id = entity;
        loop {
            let parent = self[id].hierarchies.parent(rel);
            if parent.is_null() || parent == id {
                return parent;
            } else {
                id = parent;
            }
        }
    }

    /// Mark the entity as being a root of the given hierarchy
    pub(crate) fn set_root(&mut self, rel: HierarchyName, entity: EntityId) {
        self.set_parent(rel, entity, entity);
    }

    pub(crate) fn set_parent(&mut self, rel: HierarchyName, child: EntityId, parent: EntityId) {
        // We can't operate on a null child
        if child.is_null() {
            return;
        }
        // Remove current parent
        self.unparent(rel, child);

        // Add new parent
        let idx = rel as usize;
        let child_data = self.get_mut(child).unwrap();
        child_data.hierarchies.0[idx].parent = parent;

        // If parent is self or is a null, then stop here -- nothing to do
        if child == parent || parent.is_null() {
            return;
        }

        // Otherwise get the parent and update its child list
        let parent = self.get_mut(parent).unwrap();
        let children = &mut parent.hierarchies.0[idx].children;
        children.sorted_insert(child);
    }

    pub(crate) fn unparent(&mut self, rel: HierarchyName, child: EntityId) {
        let child_data = get_or_return!(self.get_mut(child));

        // Get the parent, give up if there is nothing to do
        let parent = child_data.hierarchies.parent(rel);
        if parent.is_null() {
            return;
        }

        // Neutralize child and parent fields
        let idx = rel as usize;
        child_data.hierarchies.0[idx].parent = EntityId::default();

        let parent = self.get_mut(parent).unwrap();
        parent.hierarchies.0[idx].children.retain(|&x| x != child);
    }

    #[inline]
    pub(crate) fn remove_all_children(&mut self, rel: HierarchyName, parent: EntityId) {
        let parent = get_or_return!(self.get_mut(parent));
        let children = std::mem::take(&mut parent.hierarchies.0[rel as usize].children);
        let parent = parent.id;
        for child in children {
            let child = self.get_mut(child).unwrap();
            let p = std::mem::take(&mut child.hierarchies.0[rel as usize].parent);
            assert!(p == parent);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, EnumIter, EnumCount)]
pub(crate) enum Flag {
    IsFaction,
    IsLocation,
}

const FLAG_BACKING_SIZE: usize = Flag::COUNT / 8 + Flag::COUNT % 8;
#[derive(Default)]
pub(crate) struct Flags(TinyBitSet<u8, FLAG_BACKING_SIZE>);

impl Flags {
    pub fn set(&mut self, flag: Flag, value: bool) {
        self.0.assign(flag as usize, value);
    }

    pub fn get(&self, flag: Flag) -> bool {
        self.0[flag as usize]
    }
}
