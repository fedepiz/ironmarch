use macros::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use slotmap::Key;
use util::arena::Arena;
use util::tagged::TaggedCollection;

use crate::sites::SiteId;
use crate::{RGB, entities::*};
use crate::{simulation::*, sites::SiteData};

pub(crate) fn init(sim: &mut Simulation, arena: &Arena, seed: u64) {
    let rng = &mut SmallRng::seed_from_u64(seed);
    sim.turn_number = 1;
    init_cultures(sim);
    init_sites(sim);
    init_factions(sim, arena, rng);
    init_locations(sim, arena, rng);
    init_people(sim, arena, rng);

    sim.tick(crate::TickRequest::default(), arena);
}

macro_rules! lookup_or_continue {
    ($sim:expr, $tag:expr) => {{
        let tag: &str = $tag;
        let x = ($sim).entities.lookup(tag);
        if x.is_null() {
            continue;
        }
        x
    }};
    ($sim:expr, $tag:expr, $kind:literal) => {{
        let tag: &str = $tag;
        let x = ($sim).entities.lookup(tag);
        if x.is_null() {
            println!("Unknown {} '{tag}'", $kind);
            continue;
        }
        x
    }};
}

fn init_cultures(sim: &mut Simulation) {
    struct Desc<'a> {
        tag: &'a str,
        name: &'a str,
        names: &'a [&'a str],
    }

    const DESC: &[Desc] = &[
        Desc {
            tag: "anglish",
            name: "Anglish",
            names: crate::names::ANGLO_SAXON_MALE_NAMES,
        },
        Desc {
            tag: "brythonic",
            name: "Brythonic",
            names: crate::names::BRYTHONIC_MALE_NAMES,
        },
    ];

    for desc in DESC {
        let entity = sim.entities.spawn_with_tag(desc.tag);
        entity.name = desc.name.to_string();
        entity.kind_name = "Culture";

        entity.name_lists = {
            let name_lists = NameLists::default().with(
                NameList::PersonalNames,
                desc.names.iter().map(|x| x.to_string()).collect(),
            );

            Some(Box::new(name_lists))
        };
    }
}

fn init_sites(sim: &mut Simulation) {
    struct Desc {
        tag: &'static str,
        pos: (f32, f32),
    }

    const DESCS: &[Desc] = &[
        Desc {
            tag: "caer_ligualid",
            pos: (0., 0.),
        },
        Desc {
            tag: "din_drust",
            pos: (-7., -9.),
        },
        Desc {
            tag: "anava",
            pos: (7., -5.),
        },
        Desc {
            tag: "llan_heledd",
            pos: (3., 12.),
        },
        Desc {
            tag: "caer_ligualid-din_drust",
            pos: (-4., -4.),
        },
        Desc {
            tag: "caer_ligualid_south",
            pos: (0., 8.),
        },
        Desc {
            tag: "isura",
            pos: (-13., -8.),
        },
        Desc {
            tag: "isura_west",
            pos: (-19.5, -10.),
        },
        Desc {
            tag: "din_rheged",
            pos: (-25., -8.4),
        },
        Desc {
            tag: "ad_candidam_casam",
            pos: (-19., -6.2),
        },
    ];

    for desc in DESCS {
        sim.sites.define(desc.tag, desc.pos.into());
    }

    const CONNECTIONS: &[(&str, &str)] = &[
        ("caer_ligualid", "anava"),
        ("din_drust", "anava"),
        ("caer_ligualid", "caer_ligualid_south"),
        ("caer_ligualid_south", "llan_heledd"),
        ("caer_ligualid", "caer_ligualid-din_drust"),
        ("din_drust", "caer_ligualid-din_drust"),
        ("din_drust", "isura"),
        ("isura", "isura_west"),
        ("isura_west", "din_rheged"),
        ("isura_west", "ad_candidam_casam"),
    ];

    for (tag1, tag2) in CONNECTIONS {
        let id1 = get_or_continue!(sim.sites.lookup(&tag1), "Unknown site '{tag1}'");
        let id2 = get_or_continue!(sim.sites.lookup(&tag2), "Unknown site '{tag2}'");
        sim.sites.graph.connect(id1, id2);
    }
}

fn init_factions(sim: &mut Simulation, arena: &Arena, rng: &mut SmallRng) {
    struct Desc {
        tag: &'static str,
        name: &'static str,
        parent: &'static str,
        color: (u8, u8, u8),
    }

    const DESCS: &[Desc] = &[
        Desc {
            tag: "rheged",
            name: "Rheged",
            parent: "",
            color: (200, 40, 30),
        },
        Desc {
            tag: "clan_drust",
            name: "Clan Drust",
            parent: "rheged",
            color: (0, 0, 0),
        },
        Desc {
            tag: "clan_heledd",
            name: "Clan Heledd",
            parent: "rheged",
            color: (0, 0, 0),
        },
    ];

    for desc in DESCS {
        let color = if desc.color == (0, 0, 0) {
            random_color(rng)
        } else {
            let (r, g, b) = desc.color;
            RGB { r, g, b }
        };

        let parent = if desc.parent.is_empty() {
            EntityId::null()
        } else {
            lookup_or_continue!(sim, desc.parent)
        };

        let info = SpawnEntity {
            tag: desc.tag,
            name: SpawnName::Fixed(desc.name),
            kind: "Faction",
            looks: SpawnLooks {
                color: SpawnColor::Fixed(color),
                ..Default::default()
            },
            flags: &[Flag::IsFaction],
            parents: arena.alloc_slice([(HierarchyName::Faction, parent)]),
            ..Default::default()
        };
        spawn_entity(sim, info, rng);
    }
}

fn init_locations(sim: &mut Simulation, arena: &Arena, rng: &mut SmallRng) {
    struct Desc {
        name: &'static str,
        site: &'static str,
        culture: &'static str,
        kind: Kind,
        faction: &'static str,
    }

    enum Kind {
        Town,
        Village,
        Hillfort,
    }

    const DESCS: &[Desc] = &[
        Desc {
            name: "Caer Ligualid",
            site: "caer_ligualid",
            culture: "brythonic",
            kind: Kind::Town,
            faction: "rheged",
        },
        Desc {
            name: "Anava",
            site: "anava",
            culture: "brythonic",
            kind: Kind::Village,
            faction: "rheged",
        },
        Desc {
            name: "Din Drust",
            site: "din_drust",
            culture: "brythonic",
            kind: Kind::Hillfort,
            faction: "clan_drust",
        },
        Desc {
            name: "Llan Heledd",
            site: "llan_heledd",
            culture: "brythonic",
            kind: Kind::Village,
            faction: "clan_heledd",
        },
    ];

    for desc in DESCS {
        let faction = lookup_or_continue!(sim, desc.faction, "faction");
        let culture = lookup_or_continue!(sim, desc.culture, "culture");
        let site = get_or_continue!(sim.sites.lookup_data_mut(desc.site), "Unknown site").id;

        struct KindData {
            name: &'static str,
            image: &'static str,
            size: f32,
        }

        let kind = match desc.kind {
            Kind::Town => KindData {
                name: "Town",
                image: "town",
                size: 2.,
            },
            Kind::Village => KindData {
                name: "Village",
                image: "village",
                size: 1.4,
            },
            Kind::Hillfort => KindData {
                name: "Hillfort",
                image: "hillfort",
                size: 1.75,
            },
        };

        let info = SpawnEntity {
            tag: desc.site,
            name: SpawnName::Fixed(desc.name),
            kind: kind.name,
            looks: SpawnLooks {
                sprite: kind.image,
                size: kind.size,
                color: SpawnColor::Dynamic,
            },
            site,
            flags: &[Flag::IsLocation, Flag::IsPlace],
            links: arena.alloc_slice([(LinkName::Culture, culture)]),
            parents: &[(HierarchyName::Faction, faction)],
            children: &[(HierarchyName::Capital, faction)],
        };
        spawn_entity(sim, info, rng);
    }
}

#[inline]
fn bind_entity_to_site(entity: &mut EntityData, site: &mut SiteData) {
    assert!(entity.bound_site.is_null());
    assert!(site.bound_entity.is_null());
    entity.bound_site = site.id;
    site.bound_entity = entity.id;
}

fn init_people(sim: &mut Simulation, arena: &Arena, rng: &mut SmallRng) {
    // #[derive(Default)]
    // struct Desc<'a> {
    //     name: &'a str,
    //     location: &'a str,
    //     culture: &'a str,
    //     faction: &'a str,
    //     repeats: usize,
    // }

    // let descs = [Desc {
    //     location: "caer_ligualid",
    //     repeats: 4,
    //     ..Default::default()
    // }];

    // For each location, generate 5 characters of that culture

    struct Create {
        location: EntityId,
        repeats: usize,
    }

    let creates: Vec<_> = sim
        .entities
        .iter()
        .filter(|entity| entity.flags.get(Flag::IsLocation))
        .map(|location| Create {
            location: location.id,
            repeats: 5,
        })
        .collect();

    let mut spawns = vec![];

    for desc in creates {
        for _ in 0..desc.repeats {
            let location = &sim.entities[desc.location];
            let culture = location.links.get(LinkName::Culture);
            let faction = location.hierarchies.parent(HierarchyName::Faction);

            let location = location.id;
            let name = SpawnName::FromList(culture, NameList::PersonalNames);

            spawns.push(SpawnEntity {
                name,
                kind: "Person",
                looks: SpawnLooks::default(),
                site: Default::default(),
                flags: &[Flag::IsPerson],
                links: arena.alloc_slice([(LinkName::Culture, culture)]),
                parents: arena.alloc_slice([
                    (HierarchyName::PlaceOf, location),
                    (HierarchyName::Faction, faction),
                ]),
                children: &[],
                ..Default::default()
            });
        }
    }

    for info in spawns {
        spawn_entity(sim, info, rng);
    }
}

enum SpawnName<'a> {
    Fixed(&'a str),
    FromList(EntityId, NameList),
}
impl Default for SpawnName<'_> {
    fn default() -> Self {
        Self::Fixed("")
    }
}

#[derive(Default)]
struct SpawnLooks {
    sprite: &'static str,
    size: f32,
    color: SpawnColor,
}

enum SpawnColor {
    Fixed(RGB),
    Dynamic,
}

impl Default for SpawnColor {
    fn default() -> Self {
        Self::Fixed(Default::default())
    }
}

#[derive(Default)]
struct SpawnEntity<'a> {
    tag: &'a str,
    name: SpawnName<'a>,
    kind: &'static str,
    looks: SpawnLooks,
    site: SiteId,
    flags: &'a [Flag],
    links: &'a [(LinkName, EntityId)],
    parents: &'a [(HierarchyName, EntityId)],
    children: &'a [(HierarchyName, EntityId)],
}

fn spawn_entity(sim: &mut Simulation, info: SpawnEntity, rng: &mut SmallRng) {
    let name = match info.name {
        SpawnName::Fixed(x) => x.to_string(),
        SpawnName::FromList(source, list) => sim.entities[source]
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
        SpawnColor::Fixed(rgb) => EntityColor {
            current: rgb,
            dirty: false,
        },
        SpawnColor::Dynamic => EntityColor {
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
}

fn random_color(rng: &mut SmallRng) -> RGB {
    RGB {
        r: rng.gen_range(u8::MIN..=u8::MAX),
        g: rng.gen_range(u8::MIN..=u8::MAX),
        b: rng.gen_range(u8::MIN..=u8::MAX),
    }
}
