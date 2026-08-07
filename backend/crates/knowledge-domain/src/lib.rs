// SPEC-001: Core Graph Model — Identity & Scoped Knowledge
// Implements the resolve(scope) semantics per docs/specs/SPEC-001-core-graph-model.md

use std::collections::HashMap;

use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    Common,
    Org,
    Workspace,
    Project,
}

impl Scope {
    /// Ladder position: 0 = closest to the edge (project), 3 = root (common).
    fn ladder_index(self) -> usize {
        match self {
            Scope::Project => 0,
            Scope::Workspace => 1,
            Scope::Org => 2,
            Scope::Common => 3,
        }
    }
}

const LADDER: [Scope; 4] = [Scope::Project, Scope::Workspace, Scope::Org, Scope::Common];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Node,
    Edge,
    Combo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Candidate,
    Current,
    Retired,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertySet {
    pub scope: Scope,
    pub version: u64,
    pub status: Status,
    pub properties: HashMap<String, Value>,
}

impl PropertySet {
    pub fn current(scope: Scope, version: u64, properties: HashMap<String, Value>) -> Self {
        Self {
            scope,
            version,
            status: Status::Current,
            properties,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub id: Uuid,
    pub kind: EntityKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SetError {
    UnknownEntity,
    VersionOutOfOrder { current: u64, attempted: u64 },
}

#[derive(Debug, Default)]
pub struct Graph {
    entities: HashMap<Uuid, Entity>,
    property_sets: HashMap<(Uuid, Scope), PropertySet>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_entity(&mut self, kind: EntityKind) -> Entity {
        let entity = Entity {
            id: Uuid::now_v7(),
            kind,
        };
        self.entities.insert(entity.id, entity.clone());
        entity
    }

    pub fn get_entity(&self, id: Uuid) -> Option<&Entity> {
        self.entities.get(&id)
    }

    pub fn set_property_set(&mut self, entity_id: Uuid, set: PropertySet) -> Result<(), SetError> {
        if !self.entities.contains_key(&entity_id) {
            return Err(SetError::UnknownEntity);
        }
        if let Some(existing) = self.property_sets.get(&(entity_id, set.scope))
            && set.version <= existing.version
        {
            return Err(SetError::VersionOutOfOrder {
                current: existing.version,
                attempted: set.version,
            });
        }
        self.property_sets.insert((entity_id, set.scope), set);
        Ok(())
    }

    pub fn soft_delete(&mut self, entity_id: Uuid, scope: Scope) {
        if let Some(set) = self.property_sets.get_mut(&(entity_id, scope)) {
            set.status = Status::Retired;
        }
    }

    /// Resolve the effective property set for a scope: walk the ladder
    /// project → workspace → org → common, returning the nearest current set.
    pub fn resolve(&self, entity_id: Uuid, scope: Scope) -> Option<&PropertySet> {
        let start = scope.ladder_index();
        LADDER[start..]
            .iter()
            .filter_map(|s| self.property_sets.get(&(entity_id, *s)))
            .find(|set| set.status == Status::Current)
    }
}
