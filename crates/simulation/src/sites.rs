use slotmap::{SecondaryMap, new_key_type};
use spatial::graph2d::Graph2D;
use util::arena::ArenaSafe;
use util::tagged::{TaggedCollection, Tags};

use spatial::geom::*;

use crate::entity::EntityId;

new_key_type! { pub(crate) struct SiteId; }

impl ArenaSafe for SiteId {}

#[derive(Default)]
pub(crate) struct Sites {
    pub graph: Graph2D<SiteId>,
    pub tags: Tags<SiteId>,
    pub data: SecondaryMap<SiteId, SiteData>,
}

impl Sites {
    pub fn define(&mut self, tag: impl Into<String>, pos: V2) -> SiteId {
        let id = self.graph.insert(pos);
        self.tags.insert(tag, id);
        self.data.insert(
            id,
            SiteData {
                id,
                ..Default::default()
            },
        );
        id
    }

    pub(crate) fn lookup_data_mut(&mut self, tag: &str) -> Option<&mut SiteData> {
        match self.lookup(tag) {
            Some(id) => Some(&mut self.data[id]),
            None => {
                println!("Undefined site '{}'", tag);
                None
            }
        }
    }

    pub(crate) fn pos_of(&self, site: SiteId) -> V2 {
        self.graph
            .get(site)
            .map(|site| site.pos)
            .unwrap_or(V2::INFINITY)
    }
}

impl TaggedCollection for Sites {
    type Output = SiteId;

    fn lookup(&self, tag: &str) -> Option<Self::Output> {
        self.tags.lookup(tag)
    }
}

#[derive(Default)]
pub(crate) struct SiteData {
    pub id: SiteId,
    pub bound_entity: EntityId,
}
