<!-- aligned-structure:v1 -->

# Summary

Spec, rule, and ticket stores accumulate stale, conflicting, and low-value entries over time. Current checks are useful but insufficient for sustained curation because they do not combine change activity, validation outcomes, and direct user feedback into one health model.

## Behavior Story

Spec, rule, and ticket stores accumulate stale, conflicting, and low-value entries over time. Current checks are useful but insufficient for sustained curation because they do not combine change activity, validation outcomes, and direct user feedback into one health model.

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

Spec, rule, and ticket stores accumulate stale, conflicting, and low-value entries over time. Current checks are useful but insufficient for sustained curation because they do not combine change activity, validation outcomes, and direct user feedback into one health model.

## Goals

- Define a continuous health scoring model for entries across spec/rule/ticket stores.
- Surface unhealthy entries with actionable remediation hints and cleanup loops.
- Integrate user/agent feedback and validation executions as first-class health signals.

## Required behavior

### Health metric taxonomy
- Define metric families at minimum:
  - freshness and change activity
  - validation coverage and result quality
  - conflict and duplication risk
  - feedback sentiment and relevance
  - linkage completeness and traceability hygiene
- Metrics are explainable and individually inspectable.

### Scoring and thresholds
- A weighted score model classifies entries into healthy, warning, and unhealthy states.
- Scoring rules are versioned and auditable.
- The model supports per-store and cross-store rollups.

### Governance and anti-gaming
- score coefficient changes require explicit score-model version increments.
- queue stability guarantees are defined for model upgrades and tie-break semantics.
- anti-gaming heuristics identify noisy low-confidence bursts and cap disproportionate influence.

### Score contract
- Score output includes: `overall_score`, `state`, `score_version`, and per-metric contributions.
- Each contribution includes `metric_id`, `raw_value`, `normalized_value`, `weight`, and `explanation`.
- Rollups include `store_rollup` and `workspace_rollup` with cardinality and coverage metadata.

### User-facing cleanup loops
- Operators receive ranked remediation queues with reason codes.
- Cleanup loop supports triage outcomes: keep, revise, merge, deprecate, archive, or escalate.
- Feedback from triage outcomes updates future scoring and prioritization.

### Queue semantics
- queue items expose source signals, score deltas, and recommended action rationale.
- queue ordering remains deterministic within a score version.
- each automated recommendation includes reversibility metadata for safe rollback.

### Performance expectations
- define SLOs for score recomputation and queue refresh at representative workspace scales.
- define incremental recompute versus full recompute budget envelopes.
- define degraded-mode behavior when budgets are exceeded.

### Signal ingestion
- Validation execution signals are consumed from test-api evidence records.
- Feedback sentiment/relevance signals are consumed from feedback-api records.
- Missing signal sources degrade gracefully with explicit coverage warnings.

## Major risks

- score opacity reducing operator trust
- feedback bias overpowering objective validation/activity signals
- over-aggressive deprecation recommendations causing accidental knowledge loss

## Acceptance criteria

- metric taxonomy and score model are fully specified and traceable
- cleanup loop contracts and remediation states are explicit
- integration boundaries for validation and feedback signals are explicit

## Traceability

- [bd1c7cc0 [audit-api] Continuous store health scoring and cleanup loops](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/bd1c7cc0-2850-418d-b701-981b95c587ee/ticket.toml)
- [67b6117b [audit-api] Health metric taxonomy and scoring model for store entries](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/67b6117b-5978-4c89-9cd4-4c8b043f4fba/ticket.toml)
- [11fb9bcf [audit-api] Cleanup loop UX and automated remediation suggestions](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/11fb9bcf-fcd5-4eff-b380-64b80f4a5c9c/ticket.toml)
- [8f021514 [audit-api] Explainable remediation queues and reversible cleanup actions](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/8f021514-6d53-45f3-a0cf-667fb3865a4d/ticket.toml)
- [f6ee97de [audit-api] Score-model versioning and anti-gaming safeguards](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/f6ee97de-c7c9-46d3-878b-c6df5f4a4bc9/ticket.toml)
- [89f21dd2 [audit-api] Health score recompute and queue refresh SLOs](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/89f21dd2-6307-4f30-b0f8-7b36b3cfce66/ticket.toml)

## Validation

- focused unit tests for metric computations and threshold classifications
- integration tests for remediation queue ranking and triage outcome updates
- regression tests showing graceful behavior when feedback/test signals are absent
- explainability snapshot tests proving stable contribution formatting across runs
