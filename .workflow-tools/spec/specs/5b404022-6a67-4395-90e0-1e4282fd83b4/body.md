<!-- aligned-structure:v1 -->

# Summary

Current topology checks can detect orphan tickets and planned convergence risks, but they do not enforce whether dependency requirements are defined, whether required dependency evidence is passing, or whether operator workflows should pause before starting work. That leaves review and board activity vulnerable to structurally-correct but evidentially-unsound graphs.

## Behavior Story

Current topology checks can detect orphan tickets and planned convergence risks, but they do not enforce whether dependency requirements are defined, whether required dependency evidence is passing, or whether operator workflows should pause before starting work. That leaves review and board activity vulnerable to structurally-correct but evidentially-unsound graphs.

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

Current topology checks can detect orphan tickets and planned convergence risks, but they do not enforce whether dependency requirements are defined, whether required dependency evidence is passing, or whether operator workflows should pause before starting work. That leaves review and board activity vulnerable to structurally-correct but evidentially-unsound graphs.

## Goals

- Extend audit and ticket health consumers to surface dependency-validation risks for the whole workspace.
- Add board check-in warnings for tickets whose dependency requirements are missing or currently unsatisfied.
- Reuse the shared `ticket-api` dependency + validation derivation instead of re-encoding evidence logic in audit or board layers.

## Required behavior

### Workspace audit findings
- Repo audit consumes the shared `ticket-api` graph/validation model and reports findings for at least:
  - dependencies with missing validation requirements
  - dependencies whose required validation executions are failed or blocked
  - dependencies whose latest required evidence is stale or absent
  - downstream tickets depending on tickets in `in-review` where required evidence is not yet satisfied
- Findings must include the dependent ticket, the dependency ticket, the unmet requirement identifiers, and the latest relevant validation outcome when present.
- Repositories without a local test store must not hard-fail the entire audit; the affected metric should report unavailable or degraded coverage with a clear hint.

### Ticket health and board enforcement
- Ticket health surfaces consume the same shared model and emit requirement-aware findings, not just topology findings.
- Board check-in warns when the target ticket has no declared dependency requirements for dependencies that should be validated, or when any dependency currently has unmet required evidence.
- Check-in warnings should remain non-destructive initially, leaving room for a later policy gate if the repository chooses stricter enforcement.

### Severity and explainability
- Audit owns severity mapping and human remediation guidance, but the evidence facts come from `ticket-api`.
- Findings must distinguish configuration gaps (`missing-requirements`) from execution failures (`failed`) and temporary execution blockers (`blocked` / `stale`).
- Human output should suggest the next corrective step, such as defining requirements, re-running a validation spec, or waiting for an in-review dependency to converge.

### Relationship to graph rendering
- The audit contract should be compatible with graph-display tools so findings can optionally be visualized over a rendered subgraph without redefining graph semantics.

## Acceptance criteria

- Audit and ticket health contracts account for dependency evidence status, not topology alone.
- Board check-in warning behavior is specified for missing or unsatisfied dependency requirements.
- The audit surface degrades gracefully when test-store data is unavailable.
- Severity mapping and remediation guidance are tied to shared `ticket-api` evidence classifications.

## Related specs

- `audit-api/ticket-dependency-topology-validation`
- `ticket-api/validation-aware-dependency-requirements-and-health`
- `ticket-api/workflow/graph-aware-best-next`

## Traceability

- [8dbff37f [audit-api] Workspace graph health and board check-in validation enforcement](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/8dbff37f-699b-4c91-bf65-6516ea6fe609/ticket.toml)
- [43fc22b3 [ticket-graph] Tracker: validation-aware graph tooling and audit enforcement](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/43fc22b3-9b36-4a54-b520-f51000330a46/ticket.toml)
- Existing topology audit spec: `a6318461-3a06-4d6d-aabb-7e06c33f4e1b`

## Validation

- Focused audit-api tests covering missing requirements, failed dependency evidence, stale evidence, and test-store-unavailable cases.
- Focused ticket-cli or ticket-api tests covering board check-in warnings driven by the shared evidence model.
- Focused integration tests proving the same fixture graph yields aligned health and audit findings.
