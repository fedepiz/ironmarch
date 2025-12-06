use macros::*;
use rand::rngs::SmallRng;
use slotmap::*;
use strum::{EnumCount, EnumIter, IntoEnumIterator};
use tinybitset::TinyBitSet;
use util::arena::*;
use util::misc::VecExt;
use util::tagged::*;

use crate::RGB;
use crate::sites::SiteId;

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
    // Appearence
    pub sprite: &'static str,
    pub size: f32,
    pub color: RGB,
    /// Set of flags
    pub flags: Flags,
    pub links: Links,
    pub name_lists: Option<Box<NameLists>>,
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
        self.spawn_with_tag("")
    }

    pub(crate) fn spawn_with_tag(&mut self, tag: &str) -> &mut EntityData {
        let id = self.entries.insert(EntityData::default());
        let data = &mut self.entries[id];
        data.id = id;
        data.kind_name = "UNKNOWN_KIND";

        if !tag.is_empty() {
            self.tags.unbind(tag);
            self.tags.insert(tag, data.id);
        }

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

    pub(crate) fn lookup(&self, tag: &str) -> EntityId {
        self.tags.lookup(tag).unwrap_or_default()
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
    /// Links a faction to the location that act as its capital
    Capital,
    /// Links a faction to its member entities
    Faction,
    /// Links a location to the entities therein located
    PlaceOf,
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

    pub(crate) fn singular_child(&self, rel: HierarchyName) -> EntityId {
        let slice = self.children(rel);
        assert!(slice.len() < 2);
        slice.first().copied().unwrap_or_default()
    }
}

impl Entities {
    /// Recursively search for the root of the entity in the hierarchy, returns a null entity if
    /// the entity is not part of the hierarchy at all
    pub(crate) fn root_of(&self, rel: HierarchyName, entity: EntityId) -> EntityId {
        let mut this = entity;
        loop {
            let parent = self[this].hierarchies.parent(rel);
            if parent.is_null() || parent == this {
                break;
            }
            this = parent;
        }
        this
    }

    pub(crate) fn ancestry<'a>(
        &self,
        arena: &'a Arena,
        rel: HierarchyName,
        entity: EntityId,
    ) -> &'a [EntityId] {
        let mut out = arena.new_vec();

        let mut this = entity;
        while !this.is_null() {
            out.push(this);
            let parent = self[this].hierarchies.parent(rel);
            if parent.is_null() || parent == this {
                break;
            }
            this = parent;
        }

        out.into_bump_slice()
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

    pub(crate) fn make_sibling(&mut self, rel: HierarchyName, child: EntityId, sibling: EntityId) {
        let parent = self.entries[sibling].hierarchies.parent(rel);
        self.set_parent(rel, child, parent);
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
        // Get out the children array
        let children = std::mem::take(&mut parent.hierarchies.0[rel as usize].children);
        let parent = parent.id;
        // And remove the parent one-by-one, via resetting it to EntityId::null
        for child in children {
            let child = self.get_mut(child).unwrap();
            let p = std::mem::take(&mut child.hierarchies.0[rel as usize].parent);
            assert!(p == parent);
        }
    }

    pub(crate) fn children<'a>(
        &'a self,
        rel: HierarchyName,
        parent: &'a EntityData,
    ) -> impl Iterator<Item = &'a EntityData> + ExactSizeIterator + DoubleEndedIterator + use<'a>
    {
        parent
            .hierarchies
            .children(rel)
            .iter()
            .map(|&id| &self.entries[id])
    }

    pub fn children_with_flags<'a>(
        &'a self,
        root: &EntityData,
        hierarchy: HierarchyName,
        flags: &[Flag],
    ) -> impl Iterator<Item = &'a EntityData> {
        root.hierarchies
            .children(hierarchy)
            .iter()
            .map(|&id| &self[id])
            .filter(|entity| entity.flags.check_all(flags))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, EnumIter, EnumCount)]
pub(crate) enum Flag {
    IsFaction,
    IsLocation,
    IsPerson,
    IsPlace,
}

const FLAG_BACKING_SIZE: usize = Flag::COUNT / 8 + Flag::COUNT % 8;
#[derive(Default)]
pub(crate) struct Flags(TinyBitSet<u8, FLAG_BACKING_SIZE>);

impl Flags {
    #[inline]
    pub fn set(&mut self, flag: Flag, value: bool) {
        self.0.assign(flag as usize, value);
    }

    #[inline]
    pub fn get(&self, flag: Flag) -> bool {
        self.0[flag as usize]
    }

    #[inline]
    pub fn set_all(&mut self, flags: &[Flag], value: bool) {
        for &flag in flags {
            self.set(flag, value);
        }
    }

    #[inline]
    pub fn check_all(&self, flags: &[Flag]) -> bool {
        flags.iter().all(|flag| self.get(*flag))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, EnumIter, EnumCount)]
pub(crate) enum LinkName {
    Culture,
}

#[derive(Default)]
pub(crate) struct Links([EntityId; LinkName::COUNT]);

impl Links {
    #[inline]
    pub fn get(&self, link: LinkName) -> EntityId {
        self.0[link as usize]
    }

    #[inline]
    pub fn set(&mut self, link: LinkName, entity: EntityId) {
        self.0[link as usize] = entity;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, EnumIter, EnumCount)]
pub(crate) enum NameList {
    PersonalNames,
}

#[derive(Default)]
pub(crate) struct NameLists(pub [Vec<String>; NameList::COUNT]);

impl NameLists {
    #[inline]
    pub fn get(&self, list: NameList) -> &[String] {
        &self.0[list as usize]
    }

    #[inline]
    pub fn with(mut self, list: NameList, value: Vec<String>) -> Self {
        self.0[list as usize] = value;
        self
    }

    pub fn pick_randomly(&self, list: NameList, rng: &mut SmallRng) -> &str {
        let list = self.get(list);
        let picked = rand::seq::SliceRandom::choose(list, rng);
        picked.map(|x| x.as_str()).unwrap_or("NONAME")
    }
}
