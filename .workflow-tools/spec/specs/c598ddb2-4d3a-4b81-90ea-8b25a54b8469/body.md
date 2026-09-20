<!-- aligned-structure:v1 -->

# Summary

Define the benchmarking and profiling plan for store-index generation so hook automation and generator implementations share explicit latency budgets, a repeatable measurement method, and evidence requirements before commit-time automation is enabled.

## Behavior Story

Define the benchmarking and profiling plan for store-index generation so hook automation and generator implementations share explicit latency budgets, a repeatable measurement method, and evidence requirements before commit-time automation is enabled.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Goal

Define the benchmarking and profiling plan for store-index generation so hook automation and generator implementations share explicit latency budgets, a repeatable measurement method, and evidence requirements before commit-time automation is enabled.

# Problem

Profiling is mentioned only in passing. Without a concrete performance plan, pre-commit integration risks stalling commits, and generator tickets have no shared evidence standard for runtime cost.

# Contract

## P1: Latency budgets

Anchored to parent-spec D2 (< 100ms for incremental runs):

- **Incremental pre-commit run** (one domain, only changed entities): target < 100ms wall-clock on a warm checkout; hard ceiling 250ms before the fallback path (git-hook-automation spec) is required.
- **Full regeneration** (all entities in a domain, e.g. `--force`): target < 2s per domain; not run inside pre-commit.
- Budgets are per-domain; a domain that cannot meet the incremental ceiling MUST use the post-commit fallback or be explicitly documented as hook-exempt.

## P2: Measurement method

- Use Criterion benches under each generator's owning crate `benches/` for micro-level generator timings (entry construction + seal + sidecar assembly + render).
- Use `tracing` spans around the generator entrypoint for end-to-end commit-path timing, read from `target/test-logs/` per the repo tracing convention.
- Report wall-clock for the incremental and full scenarios; record p50 and p95 over >= 20 runs.
- Measurements are taken on a representative fixture set (P3), not synthetic single-entry inputs.

## P3: Representative workloads

Fixtures per domain sized to the current store, with a defined growth multiple:

- ticket: current `.ticket` count and a 5x synthetic set.
- spec: current `.spec` tree depth/count and a 5x set.
- rule: current `.rule` catalog and a 5x set.
- audit: a representative `AuditReport` with mixed-severity findings.
- workspace: the current workspace DAG plus a multi-parent/multi-child synthetic DAG.

## P4: Recording and regression evaluation

- Each generator ticket records its incremental p50/p95 and full-run timing as evidence linked from the ticket and this spec.
- A regression is any incremental p95 exceeding the P1 ceiling, or a > 25% p50 regression versus the recorded baseline.
- Hook automation (`52dfd793`) MUST NOT enable a domain by default until that domain's recorded incremental p95 is within budget; otherwise the fallback path is used.

## P5: Feed into hook automation

This plan is the performance contract `52dfd793` cites. The hook decision (pre-commit vs post-commit vs exempt) per domain is driven by the recorded P1 result, not by ad hoc timing checks inside the hook.

# Scope

- Latency budgets for incremental and full runs.
- Criterion + tracing measurement methodology.
- Representative fixtures and the regression rule.
- The hand-off of evidence to git-hook automation.

# Non-goals

- Does not implement generators (generator tickets).
- Does not wire git hooks (git-hook-automation spec).
- Does not redefine digest inputs or rendering behavior.

# Acceptance Criteria

- Explicit latency targets exist for generator execution in commit-time workflows (P1).
- A repeatable benchmarking/profiling method is defined for the relevant domains (P2, P3).
- Hook automation can cite this plan instead of inventing one-off timing checks (P5).
- Generator tickets know what evidence is required before review (P4).

# Traceability

- Design ticket: [98bc6b1c](.ticket/tickets/98bc6b1c-fe7e-4c5f-b0a3-b05586f442e0/ticket.toml)
- Parent spec: generated-context/index-hierarchy-semantic-refs (`18b6a9c5`)
- Depends-on spec: generated-context/peek-lod-validation (`c4f7b0ae`)
- Feeds: generated-context/git-hook-automation (sibling)
- Generator tickets: `c5e9bb39`, `b9757ba7`, `9336a096`, `855a1e5d`, `c2409055`, `a72e3aca`
