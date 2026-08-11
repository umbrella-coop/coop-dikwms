// SPEC-001/002/003: Core Graph Model — Identity, Scoped Knowledge, Moderation, Scope Hierarchy
// Property sets are keyed by (entity, scope-instance); resolution walks an instance's
// ancestor chain (nearest-wins). COMMON_INSTANCE is the global shared truth root.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Sentinel instance id for the global `common` truth layer.
pub const COMMON_INSTANCE: Uuid = Uuid::nil();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Scope {
    Common,
    Org,
    Workspace,
    Project,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    Node,
    Edge,
    Combo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Candidate,
    Current,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    property_sets: HashMap<(Uuid, Uuid), PropertySet>,
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

    pub fn set_property_set(
        &mut self,
        entity_id: Uuid,
        instance_id: Uuid,
        set: PropertySet,
    ) -> Result<(), SetError> {
        if !self.entities.contains_key(&entity_id) {
            return Err(SetError::UnknownEntity);
        }
        if let Some(existing) = self.property_sets.get(&(entity_id, instance_id))
            && set.version <= existing.version
        {
            return Err(SetError::VersionOutOfOrder {
                current: existing.version,
                attempted: set.version,
            });
        }
        self.property_sets.insert((entity_id, instance_id), set);
        Ok(())
    }

    pub fn soft_delete(&mut self, entity_id: Uuid, instance_id: Uuid) {
        if let Some(set) = self.property_sets.get_mut(&(entity_id, instance_id)) {
            set.status = Status::Retired;
        }
    }

    /// Resolve the effective property set for an entity within a scope context.
    /// `chain` is ordered nearest-first (e.g. [workspace, org, COMMON_INSTANCE]);
    /// the nearest instance holding a current set wins.
    pub fn resolve(&self, entity_id: Uuid, chain: &[Uuid]) -> Option<&PropertySet> {
        chain
            .iter()
            .filter_map(|instance| self.property_sets.get(&(entity_id, *instance)))
            .find(|set| set.status == Status::Current)
    }
}

// ---------------------------------------------------------------------------
// SPEC-002: Moderation & Promotion
// ---------------------------------------------------------------------------

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
    pub instance: Uuid,
    pub chain: Vec<Uuid>,
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
    history: HashMap<(Uuid, Uuid), Vec<PropertySet>>,
    provenance: HashMap<(Uuid, Uuid), Provenance>,
}

impl ModerationLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit_change_request(
        &mut self,
        graph: &Graph,
        entity_id: Uuid,
        instance: Uuid,
        proposed: PropertySet,
        created_by: &str,
        chain: &[Uuid],
    ) -> Result<Uuid, SetError> {
        if !graph.entities.contains_key(&entity_id) {
            return Err(SetError::UnknownEntity);
        }
        if let Some(current) = graph.resolve(entity_id, chain)
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
            instance,
            chain: chain.to_vec(),
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
        target: Uuid,
        created_by: &str,
        target_chain: &[Uuid],
        from_chain: &[Uuid],
    ) -> Result<Uuid, ModerationError> {
        let source = graph
            .resolve(entity_id, from_chain)
            .ok_or(ModerationError::MissingSourceSet)?;
        let parent_version = graph
            .resolve(entity_id, target_chain)
            .map_or(0, |set| set.version);
        let proposed = PropertySet {
            scope: source.scope,
            version: parent_version + 1,
            status: Status::Current,
            properties: source.properties.clone(),
        };
        let request = ChangeRequest {
            id: Uuid::now_v7(),
            entity_id,
            instance: target,
            chain: target_chain.to_vec(),
            proposed,
            status: ChangeRequestStatus::Submitted,
            created_by: created_by.to_string(),
            reviewed_by: None,
            decided_at: None,
            applied_at: None,
            from: Some(PromotionSource {
                scope: source.scope,
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
        let key = (request.entity_id, request.instance);
        if let Some(current) = graph.resolve(request.entity_id, &request.chain) {
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

    pub fn history(&self, entity_id: Uuid, instance: Uuid) -> &[PropertySet] {
        self.history
            .get(&(entity_id, instance))
            .map_or(&[], |sets| sets.as_slice())
    }

    pub fn provenance(&self, entity_id: Uuid, instance: Uuid) -> Option<&Provenance> {
        self.provenance.get(&(entity_id, instance))
    }
}

// ---------------------------------------------------------------------------
// SPEC-003: Scope Hierarchy & Access Control
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Org,
    Workspace,
    Project,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeNode {
    pub id: Uuid,
    pub level: Level,
    pub name: String,
    pub parent: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    SubmitChangeRequest,
    DecideChangeRequest,
    Read,
    Write,
    ManageScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Authorization {
    Allow,
    Deny,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PolicyError {
    ScopeNotFound,
    InvalidParentLevel,
    CycleDetected,
}

#[derive(Debug, Default)]
pub struct Policy {
    scopes: HashMap<Uuid, ScopeNode>,
    memberships: HashMap<Uuid, Vec<Uuid>>,
    grants: HashMap<(Uuid, Uuid, Permission), bool>,
    attachments: HashMap<Uuid, Vec<Uuid>>,
}

impl Policy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_scope(
        &mut self,
        level: Level,
        name: &str,
        parent: Option<Uuid>,
    ) -> Result<Uuid, PolicyError> {
        let parent_level = match parent {
            Some(parent_id) => Some(
                self.scopes
                    .get(&parent_id)
                    .ok_or(PolicyError::ScopeNotFound)?
                    .level,
            ),
            None => None,
        };
        let valid = match (level, parent_level) {
            (Level::Org, None) => true,
            (Level::Org, Some(_)) => false,
            (Level::Workspace, Some(Level::Org)) => true,
            (Level::Workspace, _) => false,
            (Level::Project, Some(Level::Workspace)) => true,
            (Level::Project, _) => false,
        };
        if !valid {
            return Err(PolicyError::InvalidParentLevel);
        }
        let scope = ScopeNode {
            id: Uuid::now_v7(),
            level,
            name: name.to_string(),
            parent,
        };
        let id = scope.id;
        self.scopes.insert(id, scope);
        Ok(id)
    }

    pub fn scope(&self, id: Uuid) -> Option<&ScopeNode> {
        self.scopes.get(&id)
    }

    /// Ancestor chain, nearest-first: [self, parent, ..., root].
    pub fn chain(&self, instance_id: Uuid) -> Vec<Uuid> {
        let mut chain = Vec::new();
        let mut current = Some(instance_id);
        while let Some(id) = current {
            let Some(node) = self.scopes.get(&id) else {
                break;
            };
            chain.push(id);
            current = node.parent;
        }
        chain
    }

    pub fn add_membership(&mut self, principal: Uuid, scope_id: Uuid) {
        self.memberships
            .entry(principal)
            .or_default()
            .push(scope_id);
    }

    /// Own membership or membership of any ancestor scope.
    pub fn is_member(&self, principal: Uuid, scope_id: Uuid) -> bool {
        let chain = self.chain(scope_id);
        self.memberships
            .get(&principal)
            .is_some_and(|scopes| scopes.iter().any(|s| chain.contains(s)))
    }

    pub fn set_grant(
        &mut self,
        scope_id: Uuid,
        principal: Uuid,
        permission: Permission,
        allow: bool,
    ) -> Result<(), PolicyError> {
        if !self.scopes.contains_key(&scope_id) {
            return Err(PolicyError::ScopeNotFound);
        }
        self.grants.insert((scope_id, principal, permission), allow);
        Ok(())
    }

    /// Nearest-wins authorization: walk scope + ancestors; first matching grant
    /// for (principal, permission) decides; no match -> Deny.
    pub fn authorize(
        &self,
        principal: Uuid,
        scope_id: Uuid,
        permission: Permission,
    ) -> Authorization {
        for id in self.chain(scope_id) {
            if let Some(allow) = self.grants.get(&(id, principal, permission)) {
                return if *allow {
                    Authorization::Allow
                } else {
                    Authorization::Deny
                };
            }
        }
        Authorization::Deny
    }

    pub fn attach_entity(&mut self, entity_id: Uuid, scope_id: Uuid) {
        let attached = self.attachments.entry(scope_id).or_default();
        if !attached.contains(&entity_id) {
            attached.push(entity_id);
        }
    }

    pub fn entities_in_scope(&self, scope_id: Uuid) -> Vec<Uuid> {
        self.attachments.get(&scope_id).cloned().unwrap_or_default()
    }
}
