# ADR-001: Bounded-Context Map for Codebase Organization

- **Status:** Accepted (2026-08-07)
- **Source:** DDD brainstorm (docs/brainstorm/ddd-bounded-contexts.md, approved)
- **Standard:** `.standards/options/ddd-bounded-context-org.ai.yaml`

## Context
The codebase organized by component type (ui/, hooks/, apps/); humans and AI
agents navigate by *what they are doing*. Backend crates already approximate
bounded contexts.

## Decision
Adopt the six-context map (Graph Knowledge, Governance, Access, Registry,
Stream, Workspace Shell) as the organization spine. Frontend namespaces are
context-first (`<context>/<component>`); backend crates remain the context
containers in v1.

## Consequences
- New components land under their context namespace.
- The API layer is a translation layer, not a context.
- CI import-boundary enforcement deferred (documented rule only).

## Alternatives Considered
- Component-type organization (status quo) — rejected: poor findability.
- Per-crate context re-split — rejected: ceremony without volume.
