<!-- aligned-structure:v1 -->

# Summary

Define a workspace architecture where each store remains domain-isolated while cross-store interaction is enabled through contract interfaces and API-layer composition.

## Behavior Story

Define a workspace architecture where each store remains domain-isolated while cross-store interaction is enabled through contract interfaces and API-layer composition.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

## Goal
Define a workspace architecture where each store remains domain-isolated while cross-store interaction is enabled through contract interfaces and API-layer composition.

## Problem
Current shared storage layers expose ticket-biased naming and behavior, while multiple stores in the same workspace need uniform interaction semantics, late discovery, and robust error tracing.

## Scope
- Isolate domain persistence and workflow logic per store.
- Define a generic interfacing layer in memory-api for store discovery, indexing, references, and diagnostics.
- Define contract interfaces for cross-store interaction using inversion of control.
- Define migration phases that preserve compatibility while moving to neutral APIs.

## Locked Design Decisions
- Contract topology: hybrid contract layer (small shared core plus domain extension contract crates).
- Cross-store reference identity: URN `ce://<workspace>/<store>/<entity>`.
- Discovery mode: fully automatic recursive discovery across local and nested workspaces.
- Error envelope target: extended schema with `code`, `message`, `request_id`, `details`, `cause_chain`, `hint`, and `remediation_id`.

## Non-goals
- No immediate full rewrite of all stores in a single release.
- No runtime dependency-injection framework; composition remains in binaries via static typing.
- No removal of legacy aliases until migration gates are satisfied.

## Architecture Contract
- Each store owns domain-specific entries, persistence schema, and workflow behavior.
- Cross-store interaction occurs through store API crates and shared contract traits in leaf crates.
- Binary crates are composition roots that wire triggering-domain workflows to dependency-domain implementations.
- Nested workspaces are first-class: references may target entities in local or nested stores.

## Practical Interaction Examples
1. Ticket references spec plus tests plus feedback in same workspace:
   ticket entry links to spec id, test evidence ids, feedback ids through cross-store references resolved through API contracts.
2. Spec references rules and audit evidence:
   spec entry links to rule entries and audit artifacts across stores, preserving traceability.
3. Incremental onboarding from empty workspace:
   workspace begins with only spec store; later ticket and audit stores are added; discovery integrates newly present stores without breaking existing references.
4. Nested workspace integration:
   parent workspace references entities in nested workspace stores; discovery indexes both local and nested roots with explicit ownership metadata.

## Migration Phases
1. Phase A: introduce neutral shared vocabulary and compatibility aliases.
   - Add entity/store/workspace-neutral naming in memory-api and adapters in domain crates.
   - Gate: rule-api/spec-api/ticket-api compile against neutral names with compatibility aliases still enabled.
2. Phase B: extract and adopt contract interfaces for cross-store calls.
   - Introduce shared trait contracts and wire one end-to-end path through static composition.
   - Gate: one production path uses hybrid contract crates with no direct domain cycle.
3. Phase C: implement dynamic store discovery and cross-workspace reference resolution.
   - Recursively discover stores and unify indexing/reconciliation behavior for late-added stores.
   - Gate: absent-then-present store scenarios resolve without destructive rebuild requirements.
4. Phase D: standardize traceable error channels.
   - Structured envelope with causal chain hints and consistent machine/human outputs.
   - Gate: CLI, MCP, and HTTP surfaces emit the agreed extended envelope fields.
5. Phase E: retire legacy ticket-biased shared API names after adoption gates pass.
   - Gate: no downstream crates depend on deprecated ticket-biased names in shared memory-api layers.

## Dry-Run Findings for Extension Work
- The storage-neutralization track remains the first execution gate. Phase A vocabulary and compatibility aliases should land before any broader ticket-graph or audit consumer tries to standardize cross-store graph payloads.
- Graph rendering is a valid independent track and can proceed in parallel with phases A and B because a reusable renderer can consume existing ticket graph data without waiting for full cross-store discovery.
- Validation-aware dependency health should reuse the already-existing `test-api` store model (`ValidationSpec`, `ValidationExecution`, `ValidationLinks`) rather than inventing a separate graph-local evidence format.
- Audit and board enforcement should not implement dependency-evidence heuristics directly. They should remain downstream consumers of the shared `ticket-api` derivation to avoid duplicated classification logic across health, next, board, and audit.
- Mermaid is the best durable graph format for specs, tickets, and rule-generated docs. ASCII should stay as the terminal convenience view rather than the canonical embedded artifact.

## Parallel Work Tracks
1. Track A: memory-api neutral naming map and shared storage-kernel work.
2. Track B: contract-crate extraction and binary composition wiring.
3. Track C: ticket graph rendering primitive and closure-aware CLI graph display.
4. Track D: validation-aware dependency requirements built on current test-api evidence.
5. Track E: audit and board enforcement after Track D stabilizes.

Track C is largely independent of Tracks A and B. Track D can start once it commits to the current test-api substrate and should coordinate with Track B only where shared contracts are needed. Track E depends on Track D's shared evidence model, not merely on graph rendering.

## Follow-on Tooling Extensions
- Ticket graph display should gain a reusable graph primitive, closure-aware subgraph expansion, and Mermaid export suitable for embedding in specs or tickets as generated artifacts.
- Ticket dependency edges should learn structured validation requirements so dependency satisfaction can be resolved from indexed test evidence rather than markdown-only checklists.
- Audit and board flows should surface missing dependency requirements, failed dependency evidence, blocked executions, stale evidence, and in-review dependencies whose required validation has not converged.

## Implementation Readiness Checklist
- Tracker and child tickets are linked and include migration-ready acceptance criteria.
- Validation expectations are specified per work vector (storage, contracts, discovery, diagnostics).
- Cross-store examples cover same-workspace, nested-workspace, and incremental onboarding cases.
- Rollout can proceed incrementally with compatibility aliases until phase E exit criteria are met.
- Follow-on ticket-graph and audit extensions remain specified as downstream consumers of the shared model instead of reopening architecture ownership questions.

## Acceptance Criteria
- Shared memory-api layers provide domain-neutral interfaces for index/query/scan semantics.
- Domain crates interact through contract traits for cross-store workflows with no new cyclic dependencies.
- Discovery/indexing supports local plus nested workspaces and incremental store onboarding.
- Cross-store references are resolvable, deterministic, and validated across lifecycle operations.
- CLI, MCP, and HTTP outputs include traceable and actionable error context for cross-store failures.
- Contract, discovery, and diagnostic behavior are validated with repeatable tests, not documentation-only checks.

## Related Extension Specs
- `ticket-cli/graph-rendering-and-closure-aware-dependency-display`
- `ticket-api/validation-aware-dependency-requirements-and-health`
- `audit-api/workspace-graph-health-and-board-check-in-validation`

## Traceability
- [671d4e47 [architecture][multi-store] Tracker: cross-store interaction model and migration](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/671d4e47-b53d-4a04-aa1d-30f2aa8a2bbe/ticket.toml)
- [2b1279bd [architecture][memory-api] Neutral storage kernel and API migration](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/2b1279bd-c42f-4b0e-8835-d0d645a733ab/ticket.toml)
- [0f2be510 [architecture][contracts] IoC contract crates for cross-store interactions](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/0f2be510-378a-40eb-a98c-ab516b0ec647/ticket.toml)
- [6bd67a7a [architecture][workspace] Dynamic multi-store discovery and cross-store references](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/6bd67a7a-2a76-4dd7-a897-b4d325476621/ticket.toml)
- [d03530c6 [architecture][observability] Unified traceable error channels across stores](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/d03530c6-52e4-42d3-8d57-e750ce73c8d4/ticket.toml)
- [43fc22b3 [ticket-graph] Tracker: validation-aware graph tooling and audit enforcement](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/43fc22b3-9b36-4a54-b520-f51000330a46/ticket.toml)

## Validation
- Ticket/spec health passes for the architecture tracker and child tickets.
- Focused migration tests per work vector prove compatibility during each phase.
- Follow-on graph/validation/audit extensions define their own focused checks before implementation begins.
