<!-- aligned-structure:v2 -->

# Worktree Gitlink Integrity

## Target Code Location

[workflow-tools/session/crates/worktree-ctl/src/gitlink.rs](../../../workflow-tools/session/crates/worktree-ctl/src/gitlink.rs) defines `verify_gitlink_containment` and `partition_statuses`; [workflow-tools/session/crates/worktree-ctl/tests/maintenance.rs](../../../workflow-tools/session/crates/worktree-ctl/tests/maintenance.rs) covers gitlink integration outcomes.

## Naming Conventions

The future persisted `component_id` is `worktree-gitlink-integrity`. Use
`verify_gitlink_containment`, `partition_statuses`, and the
`worktree-gitlink-` criterion prefix.

## Reading Order

1. [fa6e85c8 Worktree Control Component Pilot](../fa6e85c8-866c-4d53-bb50-b78bd651e8ce/body.md) - composing parent.
2. [66fbd896 Worktree Git Operations](../66fbd896-19d4-4bb7-898c-7cdc76375a5e/body.md) - submodule-path-enumeration provider.
3. [c40b790f Worktree Synchronization And Integration](../c40b790f-6704-4a5e-bc62-ae7599521a7c/body.md) - integrity consumer.
4. [585aa074 Session Worktree Lifecycle Rewrite](../585aa074-356a-4169-b08b-4e3aba659a72/body.md) - related hybrid-access evidence.

## Responsibility

If implemented, integration consumers can verify gitlink containment and classify statuses before accepting a worktree integration result.

## Interfaces And Dependencies

`verify_gitlink_containment` opens `WorktreeGit` only to enumerate
`.gitmodules` submodule paths; it opens `git2::Repository` directly for
containment/classification. `partition_statuses` classifies reported status for
caller decisions.

## Behavior

- `worktree-gitlink-containment`: detect an orphan or unresolved gitlink before integration mutation.
- `worktree-gitlink-status-partitioning`: classify gitlink statuses for actionable integration handling.
- `worktree-gitlink-integration-coverage`: maintenance tests cover accepting, rejecting, and auto-fixing relevant states.

## Boundaries And Failure Cases

This component does not select a provisioning candidate or dispatch commands. An orphan or unresolvable gitlink fails containment and must stop the consumer mutation path; a dry run must not mutate state.

## Provider/Consumer Contract

Consumes `worktree-git-submodule-path-enumeration` from [66fbd896 Worktree Git Operations](../66fbd896-19d4-4bb7-898c-7cdc76375a5e/body.md) only for submodule paths and provides `worktree-gitlink-containment` to [c40b790f Worktree Synchronization And Integration](../c40b790f-6704-4a5e-bc62-ae7599521a7c/body.md). It uses `git2` directly for containment/classification and does not consume `ProvisionPolicy` or `provision_for_session`.

## Examples

`merge_rejects_orphan_gitlink_before_mutation` demonstrates that containment rejects the merge before mutation; `merge_auto_fixes_fast_forwardable_orphan_gitlink` supplies the distinct recoverable case.

## Evidence

Position: `implemented`. Run `cargo test --manifest-path workflow-tools/session/Cargo.toml -p worktree-ctl --test maintenance`.

## Scope

Owns gitlink integrity classification and containment, not synchronization control flow or policy selection.
