use std::collections::BTreeMap;

use slotmap::{SecondaryMap, SlotMap, new_key_type};
use util::arena::ArenaSafe;

use crate::simulation::*;

new_key_type! { pub(crate) struct SiteId; }

impl ArenaSafe for SiteId {}

#[derive(Default)]
pub(crate) struct SiteData {
    pub tag: String,
    pub pos: V2,
    pub neighbours: Vec<(SiteId, f32)>,
}

impl Tagged for SiteData {
    fn tag(&self) -> &str {
        &self.tag
    }
}

#[derive(Default)]
pub(crate) struct Sites {
    entries: SlotMap<SiteId, SiteData>,
    distances: BTreeMap<(SiteId, SiteId), f32>,
}

impl std::ops::Index<SiteId> for Sites {
    type Output = SiteData;

    fn index(&self, index: SiteId) -> &Self::Output {
        &self.entries[index]
    }
}

impl Sites {
    pub fn define(&mut self, tag: impl Into<String>, pos: V2) -> SiteId {
        self.entries.insert(SiteData {
            tag: tag.into(),
            pos,
            neighbours: vec![],
        })
    }

    pub fn make_secondary_map<T>(&self) -> SecondaryMap<SiteId, T> {
        SecondaryMap::with_capacity(self.entries.capacity())
    }

    pub fn connect(&mut self, id1: SiteId, id2: SiteId) {
        let distance = self.entries[id1].pos.distance(self.entries[id2].pos);
        Self::insert_no_repeat(&mut self.entries[id1].neighbours, id2, distance);
        Self::insert_no_repeat(&mut self.entries[id2].neighbours, id1, distance);

        // Record distance
        let min_id = id1.min(id2);
        let max_id = id1.max(id2);
        let p1 = self[min_id].pos;
        let p2 = self[max_id].pos;
        let distance = p1.distance(p2);
        self.distances.insert((min_id, max_id), distance);
    }

    fn insert_no_repeat(vs: &mut Vec<(SiteId, f32)>, id: SiteId, distance: f32) {
        if vs.iter().all(|x| x.0 != id) {
            vs.push((id, distance));
        }
    }

    pub fn lookup<'a>(&'a self, tag: &str) -> Option<(SiteId, &'a SiteData)> {
        self.entries
            .iter()
            .find(|(_, data)| data.tag.as_str() == tag)
            .map(|(id, data)| (id, data))
    }

    pub fn get(&self, id: SiteId) -> Option<&SiteData> {
        self.entries.get(id)
    }

    pub fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = (SiteId, &'a SiteData)> + ExactSizeIterator + use<'a> {
        self.entries.iter()
    }

    pub fn neighbours(&self, id: SiteId) -> &[(SiteId, f32)] {
        &self.entries[id].neighbours
    }

    pub fn greater_neighbours(&self, id: SiteId) -> impl Iterator<Item = SiteId> + use<'_> {
        self.entries
            .get(id)
            .into_iter()
            .flat_map(|data| data.neighbours.iter().copied())
            .filter(move |&x| x.0 > id)
            .map(|x| x.0)
    }

    pub fn distance(&self, id1: SiteId, id2: SiteId) -> f32 {
        if id1 == id2 {
            return 0.;
        }
        let a = id1.min(id2);
        let b = id1.max(id2);
        self.distances
            .get(&(a, b))
            .copied()
            .unwrap_or(f32::INFINITY)
    }

    pub fn astar(&self, start_node: SiteId, end_node: SiteId) -> Option<(Vec<SiteId>, f32)> {
        const RATE: f32 = 1000.;

        fn metric(x: f32) -> i64 {
            (x * RATE).round() as i64
        }

        fn from_metric(x: i64) -> f32 {
            x as f32 / RATE
        }

        let end_v2 = self.get(end_node).unwrap().pos;
        pathfinding::directed::astar::astar(
            &start_node,
            |&site| self.neighbours(site).iter().map(|&(s, d)| (s, metric(d))),
            |&site| {
                let site_v2 = self.get(site).unwrap().pos;
                metric(end_v2.distance(site_v2))
            },
            |&site| site == end_node,
        )
        .map(|(steps, cost)| (steps, from_metric(cost)))
    }
}
