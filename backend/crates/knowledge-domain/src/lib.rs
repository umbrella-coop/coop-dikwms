// SPEC-001: Core Graph Model — Identity & Scoped Knowledge
// Implements the resolve(scope) semantics per docs/specs/SPEC-001-core-graph-model.md

use std::collections::HashMap;

use chrono::{DateTime, Utc};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeRequestStatus {
    Submitted,
    UnderReview,
    Approved,
    Rejected,
    Applied,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PromotionSource {
    pub scope: Scope,
    pub version: u64,
}

#[derive(Debug, Clone)]
pub struct ChangeRequest {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub scope: Scope,
    pub proposed: PropertySet,
    pub status: ChangeRequestStatus,
    pub created_by: String,
    pub reviewed_by: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
    pub from: Option<PromotionSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
    Applied,
    AlreadyApplied,
    NotApproved,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Provenance {
    pub request_id: Uuid,
    pub reviewed_by: String,
    pub applied_at: Option<DateTime<Utc>>,
    pub from_scope: Option<Scope>,
    pub from_version: Option<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ModerationError {
    RequestNotFound,
    WrongStatus,
    NotParentScope,
    MissingSourceSet,
}

#[derive(Debug, Default)]
pub struct ModerationLedger {
    change_requests: HashMap<Uuid, ChangeRequest>,
    history: HashMap<(Uuid, Scope), Vec<PropertySet>>,
    provenance: HashMap<(Uuid, Scope), Provenance>,
}

impl ModerationLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit_change_request(
        &mut self,
        graph: &Graph,
        entity_id: Uuid,
        scope: Scope,
        proposed: PropertySet,
        created_by: &str,
    ) -> Result<Uuid, SetError> {
        if !graph.entities.contains_key(&entity_id) {
            return Err(SetError::UnknownEntity);
        }
        if let Some(current) = graph.resolve(entity_id, scope)
            && proposed.version <= current.version
        {
            return Err(SetError::VersionOutOfOrder {
                current: current.version,
                attempted: proposed.version,
            });
        }
        let request = ChangeRequest {
            id: Uuid::now_v7(),
            entity_id,
            scope,
            proposed,
            status: ChangeRequestStatus::Submitted,
            created_by: created_by.to_string(),
            reviewed_by: None,
            decided_at: None,
            applied_at: None,
            from: None,
        };
        let id = request.id;
        self.change_requests.insert(id, request);
        Ok(id)
    }

    pub fn submit_promotion(
        &mut self,
        graph: &Graph,
        entity_id: Uuid,
        target: Scope,
        from: Scope,
        created_by: &str,
    ) -> Result<Uuid, ModerationError> {
        if from.ladder_index() >= target.ladder_index() {
            return Err(ModerationError::NotParentScope);
        }
        let source = graph
            .resolve(entity_id, from)
            .ok_or(ModerationError::MissingSourceSet)?;
        let parent_version = graph
            .resolve(entity_id, target)
            .map_or(0, |set| set.version);
        let proposed = PropertySet {
            scope: target,
            version: parent_version + 1,
            status: Status::Current,
            properties: source.properties.clone(),
        };
        let request = ChangeRequest {
            id: Uuid::now_v7(),
            entity_id,
            scope: target,
            proposed,
            status: ChangeRequestStatus::Submitted,
            created_by: created_by.to_string(),
            reviewed_by: None,
            decided_at: None,
            applied_at: None,
            from: Some(PromotionSource {
                scope: from,
                version: source.version,
            }),
        };
        let id = request.id;
        self.change_requests.insert(id, request);
        Ok(id)
    }

    pub fn get_change_request(&self, request_id: Uuid) -> Option<&ChangeRequest> {
        self.change_requests.get(&request_id)
    }

    pub fn decide_request(
        &mut self,
        request_id: Uuid,
        approve: bool,
        reviewed_by: &str,
    ) -> Result<(), ModerationError> {
        let request = self
            .change_requests
            .get_mut(&request_id)
            .ok_or(ModerationError::RequestNotFound)?;
        if !matches!(
            request.status,
            ChangeRequestStatus::Submitted | ChangeRequestStatus::UnderReview
        ) {
            return Err(ModerationError::WrongStatus);
        }
        request.status = if approve {
            ChangeRequestStatus::Approved
        } else {
            ChangeRequestStatus::Rejected
        };
        request.reviewed_by = Some(reviewed_by.to_string());
        request.decided_at = Some(Utc::now());
        Ok(())
    }

    pub fn apply_approved(&mut self, graph: &mut Graph, request_id: Uuid) -> ApplyOutcome {
        let Some(request) = self.change_requests.get_mut(&request_id) else {
            return ApplyOutcome::NotApproved;
        };
        if request.status == ChangeRequestStatus::Applied {
            return ApplyOutcome::AlreadyApplied;
        }
        if request.status != ChangeRequestStatus::Approved {
            return ApplyOutcome::NotApproved;
        }
        let key = (request.entity_id, request.scope);
        if let Some(current) = graph.resolve(request.entity_id, request.scope) {
            if current.version >= request.proposed.version {
                return ApplyOutcome::AlreadyApplied;
            }
            self.history.entry(key).or_default().push(current.clone());
        }
        let proposed = request.proposed.clone();
        let from = request.from.clone();
        graph.property_sets.insert(key, proposed);
        request.status = ChangeRequestStatus::Applied;
        request.applied_at = Some(Utc::now());
        self.provenance.insert(
            key,
            Provenance {
                request_id,
                reviewed_by: request.reviewed_by.clone().unwrap_or_default(),
                applied_at: request.applied_at,
                from_scope: from.as_ref().map(|f| f.scope),
                from_version: from.as_ref().map(|f| f.version),
            },
        );
        ApplyOutcome::Applied
    }

    pub fn history(&self, entity_id: Uuid, scope: Scope) -> &[PropertySet] {
        self.history
            .get(&(entity_id, scope))
            .map_or(&[], |sets| sets.as_slice())
    }

    pub fn provenance(&self, entity_id: Uuid, scope: Scope) -> Option<&Provenance> {
        self.provenance.get(&(entity_id, scope))
    }
}
