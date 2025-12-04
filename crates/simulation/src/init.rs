use util::tagged::TaggedCollection;

use crate::simulation::*;

pub(crate) fn init(sim: &mut Simulation) {
    sim.turn_number = 1;
    // Init sites
    {
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
            let id1 = match sim.sites.lookup(&tag1) {
                Some(id) => id,
                None => {
                    println!("Unknown site '{tag1}'");
                    continue;
                }
            };
            let id2 = match sim.sites.lookup(&tag2) {
                Some(id) => id,
                None => {
                    println!("Unknown site '{tag2}'");
                    continue;
                }
            };
            sim.sites.graph.connect(id1, id2);
        }
    }
}
