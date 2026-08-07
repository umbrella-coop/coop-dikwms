# SPEC-016 Feature: Programmatic API Docs & OpenAPI Generation (mdBook)

<!-- status: Implemented -->

## Overview

Generate a valid **OpenAPI 3.1 document programmatically** from the SPEC-013 axum handlers (utoipa derives), serve it (`GET /openapi.json`) with swagger-ui mounted, and integrate it into an **mdBook documentation site** covering the platform: overview, spec index with AC traceability, generated API reference, schema-registry reference, and the live-event reference. A docgen command regenerates all artifacts from code + specs so docs never drift.

**Backlog source:** SPEC-016 (promoted per the Spec Intake Rule).

## Motivation

The platform now has a REST surface (SPEC-013) with no machine-readable contract — consumers (frontend, pipelines, agents, SPEC-015 webhooks) need an OpenAPI spec to generate clients, and humans need a living docs site. Hand-written docs drift; generation from handlers + specs does not.

## Requirements

### Requirement: OpenAPI Generation

The system SHALL emit an OpenAPI 3.1 document covering all API routes, derived programmatically from handler signatures (utoipa).

#### Scenario: OpenAPI reflects the routes
- **GIVEN** the running API
- **WHEN** `GET /openapi.json` is requested
- **THEN** a valid OpenAPI document listing all routes and their schemas is returned

### Requirement: Interactive Docs in the API

The system SHALL mount swagger-ui to browse and exercise the API.

#### Scenario: Swagger UI is served
- **GIVEN** the running API
- **WHEN** the swagger UI path is requested
- **THEN** an interactive API explorer is served against the OpenAPI document

### Requirement: mdBook Integration

The system SHALL provide an mdBook site with: platform overview, spec index (AC traceability), generated API reference (embedding the OpenAPI doc), registry namespace reference (SPEC-009), and live-event reference (SPEC-004).

#### Scenario: Book builds with generated content
- **GIVEN** the docgen command has run
- **WHEN** mdBook builds
- **THEN** the site includes the API reference, spec index, and registry namespaces

### Requirement: No-Drift Regeneration

The system SHALL regenerate docs from code + specs via a docgen command; a handler change MUST be reflected in the next regeneration (spec-vs-code convergence).

#### Scenario: New route appears in docs
- **GIVEN** a new route added to the API
- **WHEN** docgen runs and the book builds
- **THEN** the new route is present in the OpenAPI document and API reference

## Acceptance Criteria

- AC-1: Given the running API, when `GET /openapi.json` is requested, then a valid OpenAPI 3.1 document covering all routes is returned. ✅
- AC-2: Given the running API, when the swagger UI is requested, then the interactive explorer is served. ✅
- AC-3: Given docgen has run, when mdBook builds, then the site includes API reference, spec index, and registry namespaces. ✅
- AC-4: Given a handler change, when docgen runs, then the OpenAPI document reflects the change (no drift). ✅

**Implementation notes (2026-08-07):** utoipa annotations on all 11 handlers; `ApiDoc` served at `/openapi.json` + swagger-ui at `/swagger`; `cargo run -p api --example docgen` regenerates `docs/openapi/openapi.json` + book chapters (api-reference, specs index with AC counts, registry snapshot fallback). Book at `docs/book/` (mdBook).

## Technical Design

### Annotations (api crate)
- `#[derive(utoipa::OpenApi)]` on an `ApiDoc` struct listing all paths + components
- `#[utoipa::path(...)]` on each handler; `#[derive(ToSchema)]` on body/response structs
- `openapi.json` served at `GET /openapi.json` (serde serialization of the ApiDoc)
- swagger-ui via `utoipa-swagger-ui` mounted at `/swagger`

### Docgen command
- `cargo run -p api --example docgen` writes:
  - `docs/openapi/openapi.json` (API artifact)
  - `docs/book/src/api-reference.md` (embedding the spec — swagger-ui iframe or rendered endpoints table)
  - `docs/book/src/specs.md` (index generated from `docs/specs/SPEC-*.md` — titles + AC counts)
  - `docs/book/src/registry.md` (namespaces/types from `schema-registry::list_namespaces` — requires a running TerminusDB fixture; fallback to committed snapshot `docs/openapi/registry.json`)
- mdBook book at `docs/book/` (`book.toml`, `SUMMARY.md`); built with `mdbook build`

### Content plan
```text
docs/book/
  Overview            (platform philosophy, architecture)
  Specs               (generated index + AC traceability)
  API Reference       (generated — OpenAPI embed)
  Schema Registry     (generated — namespaces/types)
  Events              (SPEC-004 DomainEvents + tokens)
  Governance          (AGENTS rules, intake, delta ops)
```

## Test Plan

- [ ] Integration: `GET /openapi.json` valid + covers routes (AC-1)
- [ ] Integration: swagger UI served (AC-2)
- [ ] Unit: docgen produces book chapters from specs (AC-3)
- [ ] Integration: added route appears in regenerated OpenAPI (AC-4)

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-21 | utoipa schema derivation for custom types (PropertySet, DomainEvent) | Medium | `ToSchema` manual impls where derives fall short; openapi.json validity asserted by test |
| R-22 | mdBook external tool availability | Low | Docgen is a dev command; docs build gated in CI (SPEC-007) |
| R-23 | Registry chapter needs a live DB | Low | Committed snapshot fallback |

## Relationship to Other Specs

- SPEC-013: the API being documented; annotations added in-place
- SPEC-009: registry reference content; SPEC-004: events reference
- SPEC-007 (planned): docs build joins the gauntlet gates
- SPEC-016 is docs-only — no domain behavior changes
