use slotmap::*;
use std::collections::*;
use util::arena::*;

use crate::sites::*;
use crate::tick::TickRequest;

#[derive(Default)]
pub struct Simulation {
    pub(crate) turn_number: usize,
    pub(crate) sites: Sites,
}

impl Simulation {
    pub fn new() -> Simulation {
        let mut sim = Simulation::default();
        init(&mut sim);
        sim
    }

    pub fn tick(&mut self, request: TickRequest, arena: &Arena) -> crate::view::SimView {
        crate::tick::tick(self, request, arena)
    }
}

pub(crate) trait Tagged {
    fn tag(&self) -> &str;
}

pub(crate) trait TaggedCollection {
    type Output;

    fn lookup(&self, tag: &str) -> Option<Self::Output>;
}

impl<K: slotmap::Key, V: Tagged> TaggedCollection for SlotMap<K, V> {
    type Output = K;

    fn lookup(&self, tag: &str) -> Option<Self::Output> {
        self.iter()
            .find(|(_, data)| data.tag() == tag)
            .map(|(id, _)| id)
    }
}

pub(crate) struct Tags<T: Copy + Ord + std::hash::Hash> {
    string_to_id: HashMap<String, T>,
    id_to_string: HashMap<T, String>,
}

impl<T: Copy + Ord + std::hash::Hash> Default for Tags<T> {
    fn default() -> Self {
        Self {
            string_to_id: HashMap::default(),
            id_to_string: HashMap::default(),
        }
    }
}

impl<T: Copy + Ord + std::hash::Hash> Tags<T> {
    pub fn insert(&mut self, tag: impl Into<String>, id: T) {
        let str = tag.into();
        self.string_to_id.insert(str.clone(), id);
        self.id_to_string.insert(id, str);
    }

    pub fn unbind(&mut self, tag: &str) {
        if let Some(id) = self.string_to_id.remove(tag) {
            self.id_to_string.remove(&id);
        }
    }

    pub fn remove(&mut self, id: &T) {
        if let Some(tag) = self.id_to_string.remove(id) {
            self.string_to_id.remove(&tag);
        }
    }

    pub fn lookup(&self, tag: &str) -> Option<T> {
        self.string_to_id.get(tag).copied()
    }

    pub fn reverse_lookup(&self, id: &T) -> Option<&str> {
        self.id_to_string.get(id).map(|x| x.as_str())
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl V2 {
    pub const MIN: V2 = V2::splat(f32::MIN);
    pub const MAX: V2 = V2::splat(f32::MAX);

    pub const ZERO: V2 = V2::splat(0.);

    pub const fn splat(v: f32) -> Self {
        Self::new(v, v)
    }

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: V2) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl From<V2> for (f32, f32) {
    fn from(value: V2) -> Self {
        (value.x, value.y)
    }
}

impl From<(f32, f32)> for V2 {
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x, y)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Extents {
    pub top_left: V2,
    pub bottom_right: V2,
}

impl Default for Extents {
    fn default() -> Self {
        Self {
            top_left: V2::MIN,
            bottom_right: V2::MAX,
        }
    }
}

impl Extents {
    pub(crate) fn contains(&self, point: V2) -> bool {
        point.x >= self.top_left.x
            && point.y >= self.top_left.y
            && point.x <= self.bottom_right.x
            && point.y <= self.bottom_right.y
    }
}

fn init(sim: &mut Simulation) {
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
                Some((id, _)) => id,
                None => {
                    println!("Unknown site '{tag1}'");
                    continue;
                }
            };
            let id2 = match sim.sites.lookup(&tag2) {
                Some((id, _)) => id,
                None => {
                    println!("Unknown site '{tag2}'");
                    continue;
                }
            };
            sim.sites.connect(id1, id2);
        }
    }
}
