use slotmap::new_key_type;
use spatial::graph2d::Graph2D;
use util::arena::ArenaSafe;
use util::tagged::{Tagged, TaggedCollection, Tags};

use spatial::geom::*;

new_key_type! { pub(crate) struct SiteId; }

impl ArenaSafe for SiteId {}

#[derive(Default)]
pub(crate) struct SiteData {
    pub tag: String,
}

impl Tagged for SiteData {
    fn tag(&self) -> &str {
        &self.tag
    }
}

#[derive(Default)]
pub(crate) struct Sites {
    pub graph: Graph2D<SiteId>,
    pub tags: Tags<SiteId>,
}

impl Sites {
    pub fn define(&mut self, tag: impl Into<String>, pos: V2) -> SiteId {
        let id = self.graph.insert(pos);
        self.tags.insert(tag, id);
        id
    }
}

impl TaggedCollection for Sites {
    type Output = SiteId;

    fn lookup(&self, tag: &str) -> Option<Self::Output> {
        self.tags.lookup(tag)
    }
}
