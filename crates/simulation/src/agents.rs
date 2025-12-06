use slotmap::*;
use util::arena::*;
use util::tagged::*;

use crate::entities::EntityId;

new_key_type! { pub(crate) struct AgentId; }
new_key_type! { pub(crate) struct AgentTypeId; }

impl ArenaSafe for AgentId {}

#[derive(Default)]
pub(crate) struct AgentType {
    pub id: AgentTypeId,
    pub tag: String,
    pub name: String,
}

impl Tagged for AgentType {
    fn tag(&self) -> &str {
        &self.tag
    }
}

#[derive(Default)]
pub(crate) struct AgentData {
    pub id: AgentId,
    pub type_id: AgentTypeId,
    pub entity: EntityId,
}

pub(crate) struct Agents {
    types: SlotMap<AgentTypeId, AgentType>,
    instances: SlotMap<AgentId, AgentData>,
    dummy_type: AgentTypeId,
    dummy_instance: AgentId,
}

impl Default for Agents {
    fn default() -> Self {
        Self::new()
    }
}

impl Agents {
    pub fn new() -> Self {
        let mut this = Self {
            types: Default::default(),
            instances: Default::default(),
            dummy_type: Default::default(),
            dummy_instance: Default::default(),
        };
        this.dummy_type = this.define_type("NULL").id;
        this.dummy_instance = this.spawn(this.dummy_type, EntityId::null()).id;
        this
    }

    pub fn define_type(&mut self, tag: impl Into<String>) -> &mut AgentType {
        let tag = tag.into();
        assert!(self.types.lookup(&tag).is_none());
        let id = self.types.insert(AgentType::default());
        let data = &mut self.types[id];
        data.id = id;
        data.tag = tag;
        data
    }

    pub fn spawn(&mut self, type_id: AgentTypeId, entity: EntityId) -> &mut AgentData {
        assert!(self.types.contains_key(type_id));
        let id = self.instances.insert(AgentData::default());
        let data = &mut self.instances[id];
        data.id = id;
        data.type_id = type_id;
        data.entity = entity;
        data
    }

    pub fn get_mut(&mut self, id: AgentId) -> Option<&mut AgentData> {
        self.instances.get_mut(id)
    }
}

impl std::ops::Index<AgentTypeId> for Agents {
    type Output = AgentType;

    fn index(&self, index: AgentTypeId) -> &Self::Output {
        self.types
            .get(index)
            .unwrap_or_else(|| &self.types[self.dummy_type])
    }
}

impl std::ops::Index<AgentId> for Agents {
    type Output = AgentData;

    fn index(&self, index: AgentId) -> &Self::Output {
        self.instances
            .get(index)
            .unwrap_or_else(|| &self.instances[self.dummy_instance])
    }
}
