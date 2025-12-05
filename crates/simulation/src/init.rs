use macros::*;
use slotmap::Key;
use util::tagged::TaggedCollection;

use crate::entity::*;
use crate::{simulation::*, sites::SiteData};

pub(crate) fn init(sim: &mut Simulation) {
    sim.turn_number = 1;
    init_sites(sim);
    init_factions(sim);
    init_locations(sim);
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

fn init_factions(sim: &mut Simulation) {
    struct Desc {
        tag: &'static str,
        name: &'static str,
        parent: &'static str,
    }

    const DESCS: &[Desc] = &[
        Desc {
            tag: "rheged",
            name: "Rheged",
            parent: "",
        },
        Desc {
            tag: "clan_drust",
            name: "Clan Drust",
            parent: "rheged",
        },
    ];

    for desc in DESCS {
        let entity = sim.entities.spawn_with_tag(desc.tag);
        entity.name = desc.name.to_string();
        entity.kind_name = "Faction";
        entity.flags.set(Flag::IsFaction, true);
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
            kind: Kind::Town,
            faction: "rheged",
        },
        Desc {
            name: "Anava",
            site: "anava",
            kind: Kind::Village,
            faction: "rheged",
        },
        Desc {
            name: "Din Drust",
            site: "din_drust",
            kind: Kind::Hillfort,
            faction: "clan_drust",
        },
    ];

    for desc in DESCS {
        let faction = lookup_or_continue!(sim, desc.faction, "faction");
        let site = get_or_continue!(sim.sites.lookup_data_mut(desc.site), "Unknown site");

        let entity = sim.entities.spawn();
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

        entity.flags.set(Flag::IsLocation, true);

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
