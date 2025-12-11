#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AspectId(pub usize);

pub(crate) struct AspectData {
    pub id: AspectId,
    pub tag: String,
    pub name: String,
}

pub(crate) struct Aspects(Vec<AspectData>);

impl Default for Aspects {
    fn default() -> Self {
        Self::new()
    }
}

impl Aspects {
    pub fn new() -> Self {
        let entries = vec![AspectData {
            id: AspectId(0),
            tag: "null".into(),
            name: "Null".into(),
        }];
        Self(entries)
    }

    pub fn define(&mut self, tag: impl Into<String>, name: impl Into<String>) -> AspectId {
        let id = AspectId(self.0.len());
        self.0.push(AspectData {
            id,
            tag: tag.into(),
            name: name.into(),
        });
        id
    }

    pub fn lookup(&self, tag: &str) -> &AspectData {
        let id = self
            .0
            .iter()
            .find(|x| x.tag == tag)
            .map(|x| x.id)
            .unwrap_or_default();
        &self[id]
    }

    pub fn iter(&self) -> impl Iterator<Item = &AspectData> {
        self.0.iter()
    }

    pub fn parse_vector(&self, tags: &[(&str, f64)]) -> AspectVector {
        let mut out = AspectVector::new(self);
        for &(tag, amt) in tags {
            let id = self.lookup(tag).id;
            if id != AspectId::default() {
                continue;
            }
            out.set(id, amt);
        }
        out
    }
}

impl std::ops::Index<AspectId> for Aspects {
    type Output = AspectData;

    fn index(&self, index: AspectId) -> &Self::Output {
        &self.0[index.0]
    }
}

#[derive(Default, Clone)]
pub(crate) struct AspectVector(Vec<f64>);

impl AspectVector {
    pub fn new(aspects: &Aspects) -> Self {
        Self(aspects.iter().map(|_| 0.0).collect())
    }

    pub fn get(&self, id: AspectId) -> f64 {
        self.0.get(id.0).copied().unwrap_or(0.)
    }

    pub fn set(&mut self, id: AspectId, value: f64) {
        if let Some(x) = self.0.get_mut(id.0) {
            *x = value
        }
    }
}
