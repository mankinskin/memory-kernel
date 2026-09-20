<!-- aligned-structure:v2 -->

# Ticket Store Integration

## Target Code Location

[workflow-tools/spec/crates/spec-api/src/ticket_ref.rs](../../../workflow-tools/spec/crates/spec-api/src/ticket_ref.rs) defines explicit target identity; [workflow-tools/spec/src/cli/commands/validate_links.rs](../../../workflow-tools/spec/src/cli/commands/validate_links.rs) validates current bidirectional ticket/spec references.

## Naming Conventions

Use `TicketRef` for a governing component-spec target, `SpecificationGate` for the ticket-side typed record, and `ticket-` criterion ids. This child owns `ticket-governing-spec`, `ticket-explicit-spec-target`, `ticket-spec-before-plan`, `ticket-bidirectional-governance`, `ticket-gate-lifecycle`, `ticket-governance-recovery`, and `edge-spec-consumes-ticket-gate`.

## Reading Order

1. [90e4fb79 Production Workflow Cycle](../90e4fb79-2c60-42a6-ab10-91d243693150/body.md) - sequencing provider.
2. [a608f774 Specification Root Contract](../a608f774-9f50-4de1-90e2-ffeefb0198b1/body.md) - governed root consumer.
3. [89360ad7 Validation Store Evidence Integration](../89360ad7-d638-49e7-85ba-21839fa99851/body.md) - validation-execution provider.
4. [workflow-tools/spec/src/cli/commands/validate_links.rs](../../../workflow-tools/spec/src/cli/commands/validate_links.rs) - current link validation.

## Responsibility

If implemented, governed work can be planned only after it explicitly identifies
the specification that defines its goal and acceptance criteria.

## Interfaces And Dependencies

The governing component specification persists a distinct relation, for example:

```toml
[[governs_ticket]]
ticket_id = "<ticket-id>"
workspace = "<workspace>"
store_root = ".ticket"
```

The ticket store persists a distinct typed `SpecificationGate` record resolving
that specification. Neither side uses generic `related_specs` semantics.

```toml
[specification_gate]
governing_spec_target = "<spec-target>"
satisfaction_threshold_percent = 100

[[specification_gate.criteria]]
validation_spec_id = "<validation-spec-id>"
acceptance_criterion_id = "<criterion-id>"
```

`governing_spec_target` is required. `criteria` is a required, non-empty list
of distinct `{ validation_spec_id, acceptance_criterion_id }` pairs.
`satisfaction_threshold_percent` is a parameterizable `u8` in `0..=100` and
defaults to `100`. The record has no weights, selected execution ids, outcome
tables, ticket ids, governing-spec ids, or test-store identities.

This governing relation gates ticket goal and acceptance readiness only. It is
not component containment through `parent` and is not a provider/consumer edge.

## Behavior

- `ticket-governing-spec`: governed work references a governing spec as a readiness gate, not informational metadata.
- `ticket-explicit-spec-target`: identify the target without implicit resolution.
- `ticket-spec-before-plan`: create a ticket after, never instead of, its spec.
- `ticket-bidirectional-governance`: require the governing spec's `governs_ticket` relation and the ticket's typed gate record to resolve to each other.
- `ticket-gate-lifecycle`: for each configured criterion pair, query test-api executions by `validation_spec_id`, filter consumer-side to executions whose `links.acceptance_criterion_ids` contains `acceptance_criterion_id`, then select greatest `executed_at` and, for equal timestamps, lexicographically smallest execution id. Map the selected outcome to `1` only for `passed`; `failed`, `blocked`, and no matching execution map to `0`. Criteria have equal initial weight. The score is `passed / configured criteria`, and the gate is satisfied exactly when `100 * passed >= satisfaction_threshold_percent * criterion_count`. A validation specification and criterion shared across tickets intentionally makes every consuming gate observe the same newest outcome, so a later failed or blocked execution lowers that criterion for all consumers. No first-class criterion query or index, ticket id, governing-spec id, or `.test` store identity participates. Gate outcomes never transition ticket lifecycle state; the existing review-controlled workflow alone does so.
- `ticket-governance-recovery`: cross-store governance consumes the reusable shared operation journal prerequisite from [55d8f2eb Specification Store Contract](../55d8f2eb-70f1-4b90-8c8f-e50d5e311d48/body.md). It records planned and inverse writes, locks, collisions, and recovery state to expose resume or rollback; it never silently repairs ticket/spec divergence and does not require a global transaction.
- `edge-spec-consumes-ticket-gate`: Specification Root consumes all three ticket criteria.

## Boundaries And Failure Cases

This child neither authors requirements nor permits an ungoverned ticket to claim
governed work. The pair is not a `ComponentContractEdge`; legacy generic forms
are detect-and-report only and must never silently infer the typed relation.
Missing target identity, wrong store root, missing reverse typed gate,
pre-spec planning, or a cross-store interruption silently repaired without an
explicit resume/rollback choice is invalid. A recovered operation reports its
status and preserves the original divergent state until the selected recovery
action runs. The validation match predicate does not gain a first-class
criterion query/index, ticket, governing-spec, or test-store qualifier. An
empty or duplicate criterion-pair list, a threshold outside `0..=100`, or any
persisted score, per-criterion outcome, weight, or selected execution is
invalid.

## Provider/Consumer Contract

Provides `ticket-governing-spec`, `ticket-explicit-spec-target`, and `ticket-spec-before-plan` to [a608f774 Specification Root Contract](../a608f774-9f50-4de1-90e2-ffeefb0198b1/body.md) through `edge-spec-consumes-ticket-gate`; consumes [90e4fb79 Production Workflow Cycle](../90e4fb79-2c60-42a6-ab10-91d243693150/body.md) sequencing.

## Examples

An implementation ticket records a `TicketRef` for its governing spec before
planning begins; `validate-links` rejects it if the referenced ticket does not
link back to that spec or its declared `.ticket` store is wrong.

A worktree ticket configures three distinct criterion pairs. The newest matching
executions for its pairs are `passed`, `passed`, and `failed`, so it scores
`2 / 3`. With the default `100` threshold, `100 * 2 >= 100 * 3` is false and
the gate is unsatisfied. With its configured threshold set to `60`,
`100 * 2 >= 60 * 3` is true and the same gate is satisfied. A second worktree
ticket consuming the first pair observes the same newest execution; a later
failed or blocked execution for that pair changes both consumers' recomputed
scores, without changing either ticket's lifecycle state.

If writing the governing spec relation succeeds but the ticket-side gate write
is interrupted, the operation reports recoverable drift. Resume completes the
recorded counterpart write; rollback removes the recorded first write. Neither
store is silently changed merely because the mismatch is observed.

## Acceptance Criteria

| Criterion | Expected evidence |
| --- | --- |
| Gate shape | Reject missing governing target, empty or duplicate criterion pairs, and thresholds outside `0..=100`; default an omitted threshold to `100`. |
| Criterion selection | For each pair, query by validation spec, filter its criterion links, select greatest execution time and lexicographically smallest id on a tie, then map only `passed` to `1`. |
| Proportional satisfaction | Verify a three-criterion `2 / 3` gate is unsatisfied at `100` and satisfied at `60`, with equal criterion weight. |
| Shared revocation | Verify a newer failed or blocked matching execution lowers the recomputed criterion result for every consuming ticket and does not transition lifecycle state. |

## Evidence

Position: `partial`; explicit `TicketRef` and bidirectional validation exist, while the proportional workflow gate and journaled cross-store recovery are specified-but-not-built. Planned focused tests cover gate shape, latest-execution tie-breaking, shared revocation, the three-criterion `100` and `60` thresholds, ticket/spec relationship, interruption, recoverable drift, resume, and rollback. `./target/debug/spec.exe --workspace . health --all` validates this specification's structural health; no implementation ticket is linked now.

## Scope

Owns governing ticket linkage only; it does not create tickets or modify this draft's state.
