use rand::rngs::SmallRng;
use slotmap::Key;
use slotmap::SlotMap;
use slotmap::new_key_type;
use util::arena::Arena;
use util::tagged::TaggedCollection;
use util::tagged::Tags;

use crate::entities::*;
use crate::simulation::*;
use crate::sites::*;
use crate::view::RGB;

new_key_type! { pub struct PrototypeId; }

#[derive(Default)]
pub(crate) struct Prototypes {
    entries: SlotMap<PrototypeId, Prototype>,
    tags: Tags<PrototypeId>,
}

impl Prototypes {
    pub fn define(&mut self, tag: impl Into<String>, proto: Prototype) -> PrototypeId {
        let id = self.entries.insert(proto);
        self.tags.insert(tag, id);
        id
    }
}

#[derive(Default, Clone, Copy)]
pub(crate) struct Prototype {
    pub name: &'static str,
    pub kind: &'static str,
    pub flags: &'static [Flag],
    pub has_location: bool,
    pub has_faction: bool,
}

impl std::ops::Index<PrototypeId> for Prototypes {
    type Output = Prototype;

    fn index(&self, index: PrototypeId) -> &Self::Output {
        &self.entries[index]
    }
}

impl TaggedCollection for Prototypes {
    type Output = Prototype;

    fn lookup(&self, tag: &str) -> Option<Self::Output> {
        self.tags
            .lookup(tag)
            .and_then(|id| self.entries.get(id))
            .copied()
    }
}

#[derive(Default)]
pub(crate) struct PrototypeArgs {
    pub tag: &'static str,
    pub location: EntityId,
    pub faction: EntityId,
}

impl Prototype {
    pub fn spawn(
        self,
        sim: &mut Simulation,
        arena: &Arena,
        rng: &mut SmallRng,
        args: &PrototypeArgs,
    ) -> EntityId {
        let mut parents = arena.new_vec();

        if self.has_location {
            assert!(!args.location.is_null());
            parents.push((HierarchyName::PlaceOf, args.location));
        }

        if self.has_faction {
            assert!(!args.faction.is_null());
            parents.push((HierarchyName::Faction, args.faction));
        }

        let spawn = SpawnEntity {
            tag: args.tag,
            name: Name::Fixed(self.name),
            kind: self.kind,
            looks: Looks::default(),
            site: SiteId::null(),
            flags: self.flags,
            links: &[],
            parents: parents.into_bump_slice(),
            children: &[],
        };

        spawn_entity(sim, spawn, rng)
    }
}

#[derive(Default)]
pub(crate) struct SpawnEntity<'a> {
    pub tag: &'a str,
    pub name: Name<'a>,
    pub kind: &'static str,
    pub looks: Looks,
    pub site: SiteId,
    pub flags: &'a [Flag],
    pub links: &'a [(LinkName, EntityId)],
    pub parents: &'a [(HierarchyName, EntityId)],
    pub children: &'a [(HierarchyName, EntityId)],
}

impl SpawnEntity<'_> {
    pub fn spawn(self, sim: &mut Simulation, rng: &mut SmallRng) -> EntityId {
        self::spawn_entity(sim, self, rng)
    }
}

pub(crate) enum Name<'a> {
    Fixed(&'a str),
    FromList(EntityId, NameList),
}

impl Default for Name<'_> {
    fn default() -> Self {
        Self::Fixed("")
    }
}

#[derive(Default)]
pub(crate) struct Looks {
    pub sprite: &'static str,
    pub size: f32,
    pub color: Color,
}

pub(crate) enum Color {
    Fixed(RGB),
    Dynamic,
}

impl Default for Color {
    fn default() -> Self {
        Self::Fixed(Default::default())
    }
}

fn spawn_entity(sim: &mut Simulation, info: SpawnEntity, rng: &mut SmallRng) -> EntityId {
    let name = match info.name {
        Name::Fixed(x) => x.to_string(),
        Name::FromList(source, list) => sim.entities[source]
            .name_lists
            .as_ref()
            .unwrap()
            .pick_randomly(list, rng)
            .to_string(),
    };

    let entity = sim.entities.spawn_with_tag(info.tag);
    entity.name = name;
    entity.kind_name = info.kind;

    entity.sprite = info.looks.sprite;
    entity.size = info.looks.size;
    entity.color = match info.looks.color {
        Color::Fixed(rgb) => EntityColor {
            current: rgb,
            dirty: false,
        },
        Color::Dynamic => EntityColor {
            current: Default::default(),
            dirty: true,
        },
    };

    entity.flags.set_all(info.flags, true);

    for &(link, tgt) in info.links {
        entity.links.set(link, tgt);
    }

    if !info.site.is_null() {
        bind_entity_to_site(entity, &mut sim.sites.data[info.site]);
    }

    let entity = entity.id;

    for &(rel, parent) in info.parents {
        sim.entities.set_parent(rel, entity, parent);
    }

    for &(rel, child) in info.children {
        sim.entities.set_parent(rel, child, entity);
    }

    entity
}

#[inline]
fn bind_entity_to_site(entity: &mut EntityData, site: &mut SiteData) {
    assert!(entity.bound_site.is_null());
    assert!(site.bound_entity.is_null());
    entity.bound_site = site.id;
    site.bound_entity = entity.id;
}
