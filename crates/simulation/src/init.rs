use macros::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use slotmap::Key;
use util::arena::Arena;
use util::tagged::TaggedCollection;

use crate::simulation::*;
use crate::spawn::{self, SpawnEntity};
use crate::{RGB, entities::*};

pub(crate) fn init(sim: &mut Simulation, arena: &Arena, seed: u64) {
    let rng = &mut SmallRng::seed_from_u64(seed);
    sim.turn_number = 1;
    init_cultures(sim);
    init_prototypes(sim);
    init_sites(sim);
    init_factions(sim, arena, rng);
    let init_locations = init_locations(sim, arena, rng);
    init_people(sim, arena, &init_locations.create_people, rng);
    init_cards(sim, arena, rng);

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

fn init_prototypes(sim: &mut Simulation) {
    sim.prototypes.define(
        "bonheddwr",
        spawn::Prototype {
            name: "Bonheddwr",
            kind: "Card",
            flags: &[Flag::IsCard],
            has_location: true,
            has_faction: false,
        },
    );
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

        let info = spawn::SpawnEntity {
            tag: desc.tag,
            name: spawn::Name::Fixed(desc.name),
            kind: "Faction",
            looks: spawn::Looks {
                color: spawn::Color::Fixed(color),
                ..Default::default()
            },
            flags: &[Flag::IsFaction],
            parents: arena.alloc_slice([(HierarchyName::Faction, parent)]),
            ..Default::default()
        };
        info.spawn(sim, rng);
    }
}

#[derive(Default)]
struct InitLocations {
    create_people: Vec<CreatePeople>,
}

fn init_locations(sim: &mut Simulation, arena: &Arena, rng: &mut SmallRng) -> InitLocations {
    let mut out = InitLocations::default();

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

    out.create_people.reserve(DESCS.len());

    for desc in DESCS {
        let faction = lookup_or_continue!(sim, desc.faction, "faction");
        let culture = lookup_or_continue!(sim, desc.culture, "culture");
        let site = get_or_continue!(sim.sites.lookup_data_mut(desc.site), "Unknown site").id;

        struct KindData {
            name: &'static str,
            image: &'static str,
            size: f32,
            create_n_people: usize,
        }

        let kind = match desc.kind {
            Kind::Town => KindData {
                name: "Town",
                image: "town",
                size: 2.,
                create_n_people: 5,
            },
            Kind::Village => KindData {
                name: "Village",
                image: "village",
                size: 1.4,
                create_n_people: 3,
            },
            Kind::Hillfort => KindData {
                name: "Hillfort",
                image: "hillfort",
                size: 1.75,
                create_n_people: 3,
            },
        };

        let info = SpawnEntity {
            tag: desc.site,
            name: spawn::Name::Fixed(desc.name),
            kind: kind.name,
            looks: spawn::Looks {
                sprite: kind.image,
                size: kind.size,
                color: spawn::Color::Dynamic,
            },
            site,
            flags: &[Flag::IsLocation, Flag::IsPlace],
            links: arena.alloc_slice([(LinkName::Culture, culture)]),
            parents: &[(HierarchyName::Faction, faction)],
            children: &[(HierarchyName::Capital, faction)],
        };
        let location = info.spawn(sim, rng);
        out.create_people.push(CreatePeople {
            location,
            num_people: kind.create_n_people,
        });
    }
    out
}

struct CreatePeople {
    location: EntityId,
    num_people: usize,
}

fn init_people(sim: &mut Simulation, arena: &Arena, sources: &[CreatePeople], rng: &mut SmallRng) {
    let mut spawns = vec![];

    for desc in sources {
        for _ in 0..desc.num_people {
            let location = &sim.entities[desc.location];
            let culture = location.links.get(LinkName::Culture);
            let faction = location.hierarchies.parent(HierarchyName::Faction);

            let location = location.id;
            let name = spawn::Name::FromList(culture, NameList::PersonalNames);

            spawns.push(SpawnEntity {
                name,
                kind: "Person",
                looks: spawn::Looks::default(),
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

    for spawn in spawns {
        spawn.spawn(sim, rng);
    }
}

fn init_cards(sim: &mut Simulation, arena: &Arena, rng: &mut SmallRng) {
    struct Desc {
        prototype: &'static str,
        location: &'static str,
    }

    const DESCS: &[Desc] = &[Desc {
        prototype: "bonheddwr",
        location: "caer_ligualid",
    }];

    for desc in DESCS {
        let location = lookup_or_continue!(sim, desc.location, "location");

        let prototype =
            get_or_continue!(sim.prototypes.lookup(desc.prototype), "Undefined prototype");

        prototype.spawn(
            sim,
            arena,
            rng,
            &spawn::PrototypeArgs {
                location,
                ..Default::default()
            },
        );
    }
}

fn random_color(rng: &mut SmallRng) -> RGB {
    RGB {
        r: rng.gen_range(u8::MIN..=u8::MAX),
        g: rng.gen_range(u8::MIN..=u8::MAX),
        b: rng.gen_range(u8::MIN..=u8::MAX),
    }
}
