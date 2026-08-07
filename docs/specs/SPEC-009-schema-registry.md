# SPEC-009 Feature: Versioned Schema Registry — Core (v1)

<!-- status: Draft -->

## Overview

A multi-namespace, versioned **schema registry** stored in TerminusDB: namespaces (Protobuf `package`s with immutable field tags) are registered with their compiled `FileDescriptorSet`, versioned with **tag-immutability linting**, and used to validate `additionalType` references. `knowledge-domain` ships the **core namespace** (`core.v1`) — Thing-based kinds (Node, Edge, Combo, CreativeWork, MediaObject, Action) with schema.org's minimal property set. **schema.org is the second seeded namespace** (`org.schema.v1`, curated subset).

**v1 scope (per DISCUSS):** registry storage, tag-immutability lint, `additionalType` validation, core proto definitions. **Deferred:** protoc/prost codegen chain into Rust models, per-workspace/project namespace selection, web console, multi-namespace tooling (explicit RISK per brainstorm).

## Motivation

From the brainstorm (docs/brainstorm/schema-registry-core-set.md): entities must be typed by a stable, versioned schema language; protobuf's `package` namespaces + immutable field tags give backwards/forwards wire compatibility **without a runtime central registry**. The registry makes `additionalType` a validated seam instead of an untyped string, and seeds the platform with schema.org-standard vocabulary.

## Requirements

### Requirement: Namespace Registration

The system SHALL register a namespace (package name, version, `FileDescriptorSet` bytes) and retrieve it by (package, version). The set of type names defined by the namespace SHALL be extracted at registration.

#### Scenario: Register and retrieve a namespace
- **GIVEN** a descriptor set for package `org.schema.v1` defining type `Person`
- **WHEN** it is registered
- **THEN** retrieving `(org.schema.v1)` returns the descriptor and the type list `["Person"]`

### Requirement: Tag-Immutability Linting

Registering a new version of an existing namespace SHALL fail if any field tag is **reused for a different field** or **changed** relative to the previous version; additive fields with new tags SHALL pass.

#### Scenario: Reused tag is rejected
- **GIVEN** namespace `core.v1` v1 with field `name = 1`
- **WHEN** v2 changes `name` to tag 2 and uses tag 1 for `url`
- **THEN** registration fails with a tag-violation error

#### Scenario: Additive fields pass
- **GIVEN** namespace `core.v1` v1 with fields `name = 1`
- **WHEN** v2 adds `url = 2`
- **THEN** registration succeeds (backwards compatible)

### Requirement: additionalType Validation

The system SHALL validate that every `additionalType` IRI referenced by an entity resolves to a **registered type** in the registry.

#### Scenario: Unknown type is rejected
- **GIVEN** an entity whose `additionalType` references an unregistered IRI
- **WHEN** validated
- **THEN** validation fails with a type-not-found error

#### Scenario: Registered type passes
- **GIVEN** an entity whose `additionalType` references a registered type IRI
- **WHEN** validated
- **THEN** validation passes

### Requirement: Core Namespace Definitions

The system SHALL ship `core.v1` protobuf definitions: `Thing` base (`name`=1, `identifier`=2, `url`=3, `additionalType`=4) and kinds Node, Edge (`subject`/`object`/`relationship`), Combo (`members`), CreativeWork, MediaObject, Action — all deriving from Thing. Core fields SHALL use tags 1–15.

#### Scenario: Core definitions lint clean
- **GIVEN** the core.v1 definitions
- **WHEN** linted
- **THEN** all fields use tags 1–15 with no reuse, and all kinds embed the Thing base

#### Scenario: Core namespace seeds on init
- **GIVEN** a fresh registry
- **WHEN** it is initialized
- **THEN** `core.v1` and `org.schema.v1` are registered

## Acceptance Criteria

- AC-1: Given a descriptor set for a package, when registered, then it is retrievable with its extracted type list.
- AC-2: Given a namespace with a changed/reused tag, when a new version is registered, then registration fails with a tag violation.
- AC-3: Given a namespace with only additive fields, when a new version is registered, then registration succeeds.
- AC-4: Given an entity with an unregistered `additionalType` IRI, when validated, then validation fails.
- AC-5: Given an entity with a registered `additionalType` IRI, when validated, then validation passes.
- AC-6: Given the core.v1 definitions, when linted, then tags are 1–15, no reuse, kinds embed Thing.
- AC-7: Given a fresh registry, when initialized, then `core.v1` and `org.schema.v1` are registered.

## Technical Design

### Crate: `backend/crates/schema-registry`

```text
schemas/proto/core/v1/core.proto   # canonical core definitions (artifact)
crates/schema-registry/
  descriptor.rs   # FileDescriptorSet parse via prost-types (package/type/field/tag extraction)
  registry.rs     # TerminusDB-backed NamespaceDoc store (register/get/types)
  lint.rs         # pure tag-immutability check between versions
  validate.rs     # additionalType -> registered type resolution
  core.rs         # core.v1 descriptor constant (constructed via prost-types, mirrors core.proto)
```

- Deps: `prost-types` (+ `prost` for descriptor encoding in tests), `terminusdb-client` (fork), `uuid`, `serde_json`, `tokio`
- `NamespaceDoc` (`#[derive(TerminusDBModel)]`): package, version, descriptor (base64), types `Vec<String>` — one immutable doc per (package, version)
- Validation on registration: tag lint against the stored previous version; duplicate (package, version) rejected
- `additionalType` IRI format: `terminusdb://schema/{package}/{Type}` (resolvable, per brainstorm C3)
- **RISK (explicit):** multi-schema-namespace complexity — v1 ships exactly `core.v1` + `org.schema.v1`; schema.org curated subset is a minimal hand-built descriptor (no upstream import in v1)

### Seed descriptors
- `core.v1`: constructed in Rust via `prost_types` (mirrors `core.proto`); protoc-compiled artifacts deferred to the codegen-chain spec
- `org.schema.v1`: minimal curated subset (Thing, Person, Organization, CreativeWork, Report, MediaObject, Action/AssessAction tree subset)

## Test Plan

- [ ] Unit: descriptor parsing extracts package/type/field/tag (pure)
- [ ] Unit: tag lint — reuse rejected, additive passes (AC-2/AC-3, pure)
- [ ] Unit: core definitions lint 1–15 + Thing embedding (AC-6, pure)
- [ ] Integration: register + retrieve + type list (AC-1)
- [ ] Integration: seeded namespaces on init (AC-7)
- [ ] Integration: additionalType validation pass/fail (AC-4/AC-5)
- [ ] CI: real-server fixture (TerminusDBServer pattern)

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-11 | Multi-namespace complexity (explicit RISK per brainstorm) | Medium | v1 = 2 namespaces; schema.org minimal subset; extend later |
| R-12 | Hand-built descriptors drift from `core.proto` | Low | Single construction site + lint test; protoc integration deferred |
| R-13 | prost-types dependency footprint | Low | Only descriptor encoding/decoding used |
| R-14 | ~~additionalType validation as hard failure~~ **AMENDED (2026-08-07):** per the **Schemaless-by-default** project philosophy, data mismatches with registered schemas produce **warnings, not failures** — `validate_additional_type` SHALL return a warning outcome; the write proceeds. Tag-immutability lint remains hard (wire-compatibility guarantee, not data conformance) | Low | Change `validate_additional_type` to warn-and-continue; enforcement opt-in per scope later |

## Relationship to Other Specs

- SPEC-001: entity `kind`/`additionalType` model; codegen chain deferred
- SPEC-006: TerminusDB storage pattern reused (immutable versioned docs)
- SPEC-003: per-scope namespace selection deferred (v2)
- SPEC-002: AssessAction types seeded via `org.schema.v1` (action records in PostgreSQL per brainstorm — future spec)
