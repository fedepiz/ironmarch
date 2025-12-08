use slotmap::*;
use util::tagged::{TaggedCollection, Tags};

#[derive(Default)]
pub(crate) struct Aspects {
    entries: SlotMap<AspectId, AspectData>,
    tags: Tags<AspectId>,
}

impl Aspects {
    pub fn define(&mut self, tag: &'static str, name: &'static str) -> AspectId {
        let id = self.entries.insert(AspectData { tag, name });
        self.tags.insert(tag, id);
        id
    }
}

impl TaggedCollection for Aspects {
    type Output = AspectId;

    fn lookup(&self, tag: &str) -> Option<Self::Output> {
        self.tags.lookup(tag)
    }
}

impl std::ops::Index<AspectId> for Aspects {
    type Output = AspectData;

    fn index(&self, index: AspectId) -> &Self::Output {
        &self.entries[index]
    }
}

new_key_type! { pub struct AspectId; }

#[derive(Clone, Copy)]
pub(crate) struct AspectData {
    pub tag: &'static str,
    pub name: &'static str,
}
