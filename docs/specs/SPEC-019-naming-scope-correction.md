# SPEC-019 Feature: Naming & Bit Scope Correction — coop-codes.network-graph

<!-- status: Approved -->

## Overview

Correct the placeholder `grps.coop-graph` Bit scope to the real organization scope **`coop-codes.network-graph`** (Bit dev org `@coop-codes`), and fix the "graph-network" naming drift to `network-graph` wherever it refers to the Bit scope/namespace. The repo directory name `coop-graph-network` is unchanged (it is the repository); only Bit-scope naming is corrected.

## Motivation

The Bit workspace was scaffolded with a placeholder scope (`grps.coop-graph`, and the template's own `grps`/`n` artifacts). The platform's components must resolve under the real organization scope so `bit export` publishes to `@coop-codes`, and so component ids like `@coop-codes/network-graph.coop-graph-app` are stable before any release.

## Requirements

### Requirement: Default Scope Correction

The workspace SHALL use `coop-codes.network-graph` as its default scope; new components SHALL resolve to `@coop-codes/network-graph.*`.

#### Scenario: New component resolves to the org scope
- **GIVEN** the workspace configured with the org scope
- **WHEN** a component is created
- **THEN** its package id is `@coop-codes/network-graph.<name>`

### Requirement: Namespace Rename

Component namespaces SHALL use `network-graph` (not `coop-graph`); existing `coop-graph/` namespaces SHALL be renamed via `bit rename` semantics.

#### Scenario: Namespace matches scope name
- **GIVEN** the renamed workspace
- **WHEN** component directories are listed
- **THEN** namespaces live under `network-graph/` with component ids `@coop-codes/network-graph.*`

### Requirement: Reference Hygiene

No configuration, docs, or AGENTS references SHALL use `grps.coop-graph` for the Bit scope.

#### Scenario: No stale scope references
- **GIVEN** the corrected repository
- **WHEN** scope references are grepped
- **THEN** only `coop-codes.network-graph` (and the unchanged repo name) appear

## Acceptance Criteria

- AC-1: Given the workspace, when a component is created, then its package id is `@coop-codes/network-graph.*`.
- AC-2: Given the workspace, when `bit status` runs, then it is clean with no unresolved remote references.
- AC-3: Given the repository, when scope references are grepped, then no `grps.coop-graph` remains.

## Technical Design

- `frontend/workspace.jsonc`: `defaultScope: "coop-codes.network-graph"` (applied)
- Component dirs: `frontend/network-graph/...` (applied for new components; template leftovers under `coop-graph/` to be renamed or removed when the spike trims the demo)
- Remove stale `coop-codes.network-graph/frontend` workspace-config entry (applied — the remote scope doesn't exist yet; re-add when the scope is exported)
- Grep audit: `rg "grps.coop-graph|grps\." frontend docs AGENTS.md`

## Test Plan

- [ ] `bit status` clean (AC-2)
- [ ] `rg "grps"` zero matches (AC-3)
- [ ] New component id check (AC-1)

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-24 | `@coop-codes` org membership/scope creation not verified from this machine | Low | `bit login` + `bit scope` checks during export (not part of the spike) |
| R-25 | Template demo components under `coop-graph/` remain | Low | Removed during spike cleanup or SPEC-018 trim |

## Relationship to Other Specs

- SPEC-018: frontend platform components resolve under this scope
- Frontend spike: applies the rename now
