use std::collections::HashMap;

use slotmap::SlotMap;

pub trait Tagged {
    fn tag(&self) -> &str;
}

pub trait TaggedCollection {
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

type Type<T> = HashMap<String, T>;

pub struct Tags<T: Copy + Ord + std::hash::Hash> {
    string_to_id: Type<T>,
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
