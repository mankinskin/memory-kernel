<!-- aligned-structure:v1 -->

# Summary

`depends_on` currently expresses structural ordering only. Validation requirements still live implicitly in ticket prose or ad hoc review habits, so the graph cannot answer whether a dependency has been proven, is failing, is blocked, or is merely in progress. That leaves `ticket next`, health, audit, and board flows unable to reason about dependency evidence with one canonical model.

## Behavior Story

`depends_on` currently expresses structural ordering only. Validation requirements still live implicitly in ticket prose or ad hoc review habits, so the graph cannot answer whether a dependency has been proven, is failing, is blocked, or is merely in progress. That leaves `ticket next`, health, audit, and board flows unable to reason about dependency evidence with one canonical model.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Problem

`depends_on` currently expresses structural ordering only. Validation requirements still live implicitly in ticket prose or ad hoc review habits, so the graph cannot answer whether a dependency has been proven, is failing, is blocked, or is merely in progress. That leaves `ticket next`, health, audit, and board flows unable to reason about dependency evidence with one canonical model.

## Goals

- Extend dependency semantics so a ticket can declare which validation items from its dependencies matter for advancement.
- Resolve dependency-validation state from indexed `test-api` evidence using existing `ValidationSpec`, `ValidationExecution`, and `ValidationLinks` concepts.
- Reuse one `ticket-api` derivation across health, next/flow planning, audit, and board enforcement.
- Migrate implicit validation steps out of markdown-only ticket text into the indexed test store.

## Required behavior

### Dependency validation requirements
- A dependency edge or dependency-owned adjunct metadata can declare required validation targets.
- Requirement targets may reference validation spec ids, acceptance-criterion ids, or other stable validation identifiers that map to `ValidationSpec.links` and `ValidationExecution.links`.
- The contract must define how a dependent ticket states: which upstream ticket it depends on, which validation requirements must be satisfied, and whether the dependency is satisfied structurally, evidentially, or both.

### Evidence resolution from test-api
- `ticket-api` must resolve dependency evidence by reading linked `ValidationSpec` and `ValidationExecution` records from the indexed test store rather than scraping ticket descriptions.
- The model must classify at least these states: `missing-requirements`, `unproven`, `passed`, `failed`, `blocked`, and `stale`.
- `stale` covers cases where the latest execution predates materially relevant dependency changes or otherwise falls outside an agreed freshness rule.
- The model must distinguish the absence of declared requirements from declared requirements that have no recorded executions.

### Shared derived graph and health model
- `ticket-api` owns the library derivation that joins dependency topology, workflow state, and dependency-evidence state.
- `ticket next`, flow-graph planning, ticket health, audit surfaces, and board check-in validation must reuse this derivation rather than inventing local heuristics.
- The model must be able to answer which dependencies are structurally done but still blocked by failed or missing required evidence.

### Relationship to review and in-review dependencies
- A dependency in `in-review` is not automatically healthy if required validation evidence is failed, blocked, stale, or missing.
- Health consumers must be able to surface cases where a downstream ticket appears unblocked topologically but should not advance because dependency evidence has not converged.

### Migration from implicit markdown validation
- Existing free-text validation sections in tickets remain readable during migration, but the canonical satisfiability signal comes from indexed `test-api` records.
- The contract should define a migration path for lifting dependency validation requirements into structured, queryable metadata.

## Acceptance criteria

- A ticket-api contract exists for declaring dependency-level validation requirements against stable validation identifiers.
- Dependency-evidence resolution is defined in terms of current `test-api` entities and link fields rather than future placeholder types.
- One shared derived model can feed next ranking, health findings, audit findings, and board warnings.
- The contract distinguishes missing requirements, missing executions, failed executions, blocked executions, stale evidence, and satisfied evidence.

## Related specs

- `ticket-api/workflow/graph-aware-best-next`
- `architecture/cross-store-workspace-interaction`

## Traceability

- [acefc2ae [ticket-api] Validation-aware dependency requirements and health model](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/acefc2ae-e257-4bc8-a4c7-0ec3137e374d/ticket.toml)
- [43fc22b3 [ticket-graph] Tracker: validation-aware graph tooling and audit enforcement](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/43fc22b3-9b36-4a54-b520-f51000330a46/ticket.toml)
- Existing validation substrate: [6f3dcdfc [test-cli] Add test-result store and `test` CLI for validation evidence](C:/Users/linus/git/graph_app/context-engine/memory-api/.workflow-tools/ticket/tickets/6f3dcdfc-bf2f-45d7-9776-0f0a360ac199/ticket.toml)

## Validation

- Focused ticket-api tests for dependency evidence classification across passed, failed, blocked, missing, and stale cases.
- Focused ticket-api tests proving dependent satisfaction changes when linked `ValidationExecution` records change outcome.
- Focused next and health regression tests proving the same derived evidence state is reused by multiple consumers.
