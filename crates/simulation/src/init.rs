use macros::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use slotmap::Key;
use util::tagged::TaggedCollection;

use crate::{RGB, entities::*};
use crate::{simulation::*, sites::SiteData};

pub(crate) fn init(sim: &mut Simulation, seed: u64) {
    let rng = &mut SmallRng::seed_from_u64(seed);
    sim.turn_number = 1;
    init_cultures(sim);
    init_sites(sim);
    init_factions(sim, rng);
    init_locations(sim);
    init_people(sim, rng);
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

    const DESC: &[Desc] = &[Desc {
        tag: "anglish",
        name: "Anglish",
        names: &[
            "Alden",
            "Aldwin",
            "Alfred",
            "Athelstan",
            "Bede",
            "Brand",
            "Cerdic",
            "Cuthbert",
            "Dunstan",
            "Eadwig",
            "Earl",
            "Edgar",
            "Edmund",
            "Edward",
            "Edwin",
            "Egbert",
            "Eldred",
            "Elmer",
            "Ethelbert",
            "Ethelwulf",
            "Godric",
            "Godwin",
            "Grim",
            "Harold",
            "Hereward",
            "Kenelm",
            "Leofric",
            "Leofwin",
            "Offa",
            "Osbert",
            "Osborn",
            "Osmund",
            "Oswald",
            "Oswin",
            "Redwald",
            "Sigebert",
            "Siward",
            "Stigand",
            "Thurston",
            "Wada",
            "Wilfred",
            "Wulfric",
            "Wulfstan",
            "Wynstan",
        ],
    }];

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

fn init_factions(sim: &mut Simulation, rng: &mut SmallRng) {
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
    ];

    for desc in DESCS {
        let entity = sim.entities.spawn_with_tag(desc.tag);
        entity.name = desc.name.to_string();
        entity.kind_name = "Faction";
        entity.flags.set(Flag::IsFaction, true);

        entity.color = if desc.color == (0, 0, 0) {
            random_color(rng)
        } else {
            let (r, g, b) = desc.color;
            RGB { r, g, b }
        };
    }

    for desc in DESCS {
        let entity = lookup_or_continue!(sim, desc.tag, "faction");
        let parent = lookup_or_continue!(sim, desc.parent);
        sim.entities
            .set_parent(HierarchyName::Faction, entity, parent);
    }
}

fn init_locations(sim: &mut Simulation) {
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
            culture: "anglish",
            kind: Kind::Town,
            faction: "rheged",
        },
        Desc {
            name: "Anava",
            site: "anava",
            culture: "anglish",
            kind: Kind::Village,
            faction: "rheged",
        },
        Desc {
            name: "Din Drust",
            site: "din_drust",
            culture: "anglish",
            kind: Kind::Hillfort,
            faction: "clan_drust",
        },
    ];

    for desc in DESCS {
        let faction = lookup_or_continue!(sim, desc.faction, "faction");
        let culture = lookup_or_continue!(sim, desc.culture, "culture");
        let site = get_or_continue!(sim.sites.lookup_data_mut(desc.site), "Unknown site");

        let color = sim.entities[faction].color;

        let entity = sim.entities.spawn_with_tag(desc.site);
        entity.name = desc.name.to_string();

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

        entity.kind_name = kind.name;
        entity.sprite = kind.image;
        entity.size = kind.size;
        entity.color = color;

        let flags = &[Flag::IsLocation, Flag::IsPlace];
        entity.flags.set_all(flags, true);

        entity.links.set(LinkName::Culture, culture);

        bind_entity_to_site(entity, site);

        let entity = entity.id;

        for (rel, child, parent) in [
            (HierarchyName::Faction, entity, faction),
            (HierarchyName::Capital, entity, faction),
        ] {
            sim.entities.set_parent(rel, child, parent);
        }
    }
}

#[inline]
fn bind_entity_to_site(entity: &mut EntityData, site: &mut SiteData) {
    assert!(entity.bound_site.is_null());
    assert!(site.bound_entity.is_null());
    entity.bound_site = site.id;
    site.bound_entity = entity.id;
}

fn init_people(sim: &mut Simulation, rng: &mut SmallRng) {
    struct Desc<'a> {
        name: &'a str,
        location: &'a str,
        culture: &'a str,
        repeats: usize,
    }

    const DESCS: &[Desc] = &[Desc {
        name: "",
        location: "caer_ligualid",
        culture: "",
        repeats: 4,
    }];

    let mut spawns = vec![];

    for desc in DESCS {
        for _ in 0..desc.repeats {
            let location = lookup_or_continue!(sim, desc.location, "location");
            let culture = if desc.culture.is_empty() {
                EntityId::null()
            } else {
                lookup_or_continue!(sim, desc.culture, "culture")
            };
            spawns.push(SpawnPerson {
                name: desc.name.to_string(),
                location,
                culture,
            });
        }
    }

    for info in spawns {
        spawn_person(sim, info, rng);
    }
}

#[derive(Default)]
struct SpawnPerson {
    name: String,
    location: EntityId,
    culture: EntityId,
}

fn spawn_person(sim: &mut Simulation, info: SpawnPerson, rng: &mut SmallRng) -> EntityId {
    let culture = if info.culture.is_null() {
        sim.entities[info.location].links.get(LinkName::Culture)
    } else {
        info.culture
    };

    let name = if info.name.is_empty() {
        let name_list = sim.entities[culture].name_lists.as_ref();
        name_list
            .map(|nm| nm.pick_randomly(NameList::PersonalNames, rng))
            .unwrap()
            .to_string()
    } else {
        info.name.to_string()
    };

    let entity = sim.entities.spawn();
    entity.name = name;
    entity.kind_name = "Person";

    entity.flags.set(Flag::IsPerson, true);

    entity.links.set(LinkName::Culture, culture);

    let entity = entity.id;

    sim.entities
        .set_parent(HierarchyName::PlaceOf, entity, info.location);

    sim.entities
        .make_sibling(HierarchyName::Faction, entity, info.location);

    entity
}

fn random_color(rng: &mut SmallRng) -> RGB {
    RGB {
        r: rng.gen_range(u8::MIN..=u8::MAX),
        g: rng.gen_range(u8::MIN..=u8::MAX),
        b: rng.gen_range(u8::MIN..=u8::MAX),
    }
}
