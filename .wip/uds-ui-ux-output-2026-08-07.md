# UDS Support Map — UI/UX Quality, DDD Bounded Contexts, Component Organization

> Generated: 2026-08-07 · Source: user prompt on SPEC-017/SPEC-018 frontend organization

## How UDS supports the concerns

| Concern | UDS asset | How it helps |
|---|---|---|
| UX/UI is an interactive agreement process (PO/designer/user/engineer) | `.standards/requirement-engineering.ai.yaml` (/requirement) + `.standards/spec-driven-development.ai.yaml` (/sdd) | Spec REVIEW→APPROVE gates ARE the agreement point; INVEST stories capture "user wants" |
| Validating UX quality before code | `.standards/user-journey-testing.ai.yaml` (/journey-test) + `.standards/feature-discovery-standards.ai.yaml` | User journeys formalize the user loop |
| A11y/semantic DOM (SPEC-018) | `.standards/accessibility-standards.ai.yaml` | Bindings for semantic markup + ARIA |
| data-testid semantics/naming | `.standards/testing.ai.yaml` + `.standards/options/e2e-testing.ai.yaml` + `flow-based-testing.ai.yaml` | Selector conventions belong in a test strategy |
| Component separation (packages → own components) | `.standards/project-structure.ai.yaml` + `.standards/ai-friendly-architecture.ai.yaml` | Directory organization + AI-navigable boundaries |
| DDD bounded context + nomenclature | **MISSING in UDS** → create candidate | DDD deserves a brainstorm of generated DDD artefacts |
| `bit create react` is just a boilerplate generator | `.standards/skill-builder.ai.yaml` + UDS options mechanism | Codify generator conventions as an option/skill |
| Naming/architecture decisions | `.standards/adr-standards.ai.yaml` (/adr) | Record decisions as ADRs |

## UDS create candidates (from the prompt)

1. **DDD bounded-context organization standard** — context map, ubiquitous language, bounded-context directory/nomenclature rules → guides SPEC-017 and the Bit namespace problem
2. **Frontend component-conventions option** — SPEC-018 rules codified as `.standards/options/frontend-component-conventions.ai.yaml`
3. **Component generator standard** — what a component must include so `bit create` output conforms to the platform design

## Recommended actions (executed)

1. `/brainstorm` for DDD artefacts (context map + nomenclature + bounded-context layout) — approval gate before proceeding
2. Codify SPEC-018 + DDD outcome as UDS options under `.standards/options/`
3. ADRs for naming decisions (`graph/canvas`, `graph/select-layout`, bounded contexts)

**Gate:** no movement to another task/spec until the DDD brainstorm reaches its first approved version.
