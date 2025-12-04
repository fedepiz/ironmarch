use crate::geom::V2;

use std::collections::BTreeMap;

use slotmap::{Key, SlotMap};

pub struct Graph2D<K: Key> {
    nodes: SlotMap<K, Node<K>>,
    distances: BTreeMap<(K, K), f32>,
}

pub struct Neighbour<K> {
    pub id: K,
    pub distance: f32,
}

pub struct Node<K> {
    pub id: K,
    pub pos: V2,
    pub neighbours: Vec<Neighbour<K>>,
}

impl<K: Key> std::ops::Index<K> for Graph2D<K> {
    type Output = Node<K>;

    fn index(&self, index: K) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<K: Key> Default for Graph2D<K> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            distances: Default::default(),
        }
    }
}

impl<K: Key> Graph2D<K> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, pos: V2) -> K {
        let id = self.nodes.insert(Node {
            id: Default::default(),
            pos,
            neighbours: vec![],
        });
        self.nodes[id].id = id;
        id
    }

    pub fn connect(&mut self, id1: K, id2: K) {
        let distance = self.nodes[id1].pos.distance(self.nodes[id2].pos);
        Self::insert_no_repeat(&mut self.nodes[id1].neighbours, id2, distance);
        Self::insert_no_repeat(&mut self.nodes[id2].neighbours, id1, distance);

        // Record distance
        let min_id = id1.min(id2);
        let max_id = id1.max(id2);
        let p1 = self[min_id].pos;
        let p2 = self[max_id].pos;
        let distance = p1.distance(p2);
        self.distances.insert((min_id, max_id), distance);
    }

    fn insert_no_repeat(vs: &mut Vec<Neighbour<K>>, id: K, distance: f32) {
        if vs.iter().all(|x| x.id != id) {
            vs.push(Neighbour { id, distance });
        }
    }

    pub fn nodes<'a>(
        &'a self,
    ) -> impl Iterator<Item = &'a Node<K>> + ExactSizeIterator + use<'a, K> {
        self.nodes.values()
    }

    pub fn get(&self, id: K) -> Option<&Node<K>> {
        self.nodes.get(id)
    }

    pub fn neighbours(&self, id: K) -> &[Neighbour<K>] {
        &self.nodes[id].neighbours
    }

    pub fn greater_neighbours<'a>(
        &'a self,
        id: K,
    ) -> impl Iterator<Item = &'a Neighbour<K>> + use<'a, K> {
        self.neighbours(id).iter().filter(move |n| n.id > id)
    }

    pub fn distance(&self, id1: K, id2: K) -> f32 {
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

    pub fn astar(&self, start_node: K, end_node: K) -> Option<(Vec<K>, f32)> {
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
            |&site| {
                self.neighbours(site)
                    .iter()
                    .map(|n| (n.id, metric(n.distance)))
            },
            |&site| {
                let site_v2 = self.get(site).unwrap().pos;
                metric(end_v2.distance(site_v2))
            },
            |&site| site == end_node,
        )
        .map(|(steps, cost)| (steps, from_metric(cost)))
    }
}
