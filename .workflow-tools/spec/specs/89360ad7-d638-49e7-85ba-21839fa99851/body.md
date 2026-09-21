<!-- aligned-structure:v2 -->

# Validation Store Evidence Integration

## Target Code Location

[workflow-tools/test/crates/test-api/src/store.rs](../../../workflow-tools/test/crates/test-api/src/store.rs) owns `TestStoreConfig`; [workflow-tools/test/crates/test-api/src/lib.rs](../../../workflow-tools/test/crates/test-api/src/lib.rs) owns `ValidationSpec` and `ValidationExecution`; [.github/hooks/hooks.json](../../../.github/hooks/hooks.json) registers current hooks.

## Naming Conventions

Use `ValidationSpec`, `ValidationExecution`, and `validation-` criterion ids.
This child owns `validation-criterion-link`, `validation-observation-source`, `validation-best-effort`, and `validation-hook-enforcement`.

## Requester Input

> Hook enforcement: a git hook runs the link-parity + structure validation so markdown links stay accurate in the database.

## Reading Order

1. [b4475214 Specification Health Check](../b4475214-e14e-4926-b853-b2553444e36f/body.md) - structural validation provider.
2. [83c0b9c4 Validation Observation Contract](../83c0b9c4-1617-4751-af23-57811060f0fb/body.md) - outcome consumer.
3. [.github/hooks/hooks.json](../../../.github/hooks/hooks.json) - current registration surface.
4. [tools/agent-hooks](../../../tools/agent-hooks) - hook implementation directory.

## Responsibility

If implemented, validation outcomes remain traceable whether automated or manual,
and hooks prevent committed specification navigation from drifting from stored links.

## Interfaces And Dependencies

`ValidationSpec` identifies a target; `ValidationExecution` carries outcome,
time, and detail. Hook configuration validates changed relevant roots plus
impacted composition ancestors after relevant `.spec/` writes; explicit
`spec health --all` remains the complete CI check. The health result is
diagnostic and includes stable severity plus category/policy, including
`violation` and `migration_notice`.

## Behavior

- `validation-criterion-link`: evidence identifies applicable spec/criterion targets.
- `validation-observation-source`: outcomes expose `validation_spec_id`, `links.acceptance_criterion_ids`, status, `executed_at`, optional detail, and stable id. Ticket-gate consumers query executions by `validation_spec_id`, then filter `execution.links.acceptance_criterion_ids` for the criterion id; test-api adds no first-class criterion query or index. Newest `executed_at` wins, stable id resolves equal timestamps, absent is pending, passed satisfies, and failed or blocked revokes. Test-store identity, ticket id, and governing-spec id are not match inputs; a shared validation specification and criterion intentionally makes each matching ticket gate observe the same outcome.
- `validation-best-effort`: missing executable validation remains documented and reviewable.
- `validation-hook-enforcement`: a PostToolUse hook validates changed relevant roots and impacted composition ancestors for link parity, hierarchy, Examples, navigation, and prefix registry. `spec health --all` is the complete CI/explicit check and its JSON exits zero for findings. The hook alone applies `.spec/health-policy.toml`, maps policy error findings to a blocking response, honors only versioned allowlist identities with rationale and expiry/review metadata, and fails closed when the validator itself fails. `migration_notice` remains distinguishable from `violation`; the allowlist contains only the three unrelated `9f0b9e30` baseline findings and is never a blanket exemption.

Raw Markdown navigation is resolved relative to the owning `body.md`; hook
evidence must therefore reject root-relative internal targets that escape the
component tree's required `../../` or `../../../` base.

## Boundaries And Failure Cases

The store does not own criteria, declare fulfillment, make health globally fail
when findings exist, or decide hook blocking policy. Invalid target/status or
hook-command failure is rejected; no result remains valid when no automated
check exists. It records and exposes execution history but does not add
test-store, ticket, or governing-spec qualifiers to ticket gate matching.

## Provider/Consumer Contract

Consumes [b4475214 Specification Health Check](../b4475214-e14e-4926-b853-b2553444e36f/body.md) `health-link-parity`, `health-hierarchy-integrity`, and `health-examples-section`; provides outcomes to [83c0b9c4 Validation Observation Contract](../83c0b9c4-1617-4751-af23-57811060f0fb/body.md).

## Examples

A PostToolUse hook invokes `./target/debug/spec.exe --workspace . health --all`.
If a body link has no TOML counterpart, health returns a `violation` finding
and configured PostToolUse policy stops the write; a `migration_notice` is
reported separately and a manual validation entry can still exist where no
executable test is available.

Two tickets with gates for the same validation specification and criterion both
query the same validation-spec executions and consumer-side filter the same
`acceptance_criterion_ids` link. They observe one `passed` execution as
satisfying, then both observe a newer `failed` execution as revoking, regardless
of the execution's test-store location; this shared outcome is intentional.

## Evidence

Position: `partial`; test-api persists validation artifacts, but [.github/hooks/hooks.json](../../../.github/hooks/hooks.json) currently runs no spec health command. Planned hook integration and target/absent-automation tests.

## Scope

Owns validation evidence and enforcement wiring, not health finding semantics.
