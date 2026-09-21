<!-- aligned-structure:v2 -->

# Worktree Provisioning Policy

## Target Code Location

[workflow-tools/session/crates/session-worktree-provision/src/lib.rs](../../../workflow-tools/session/crates/session-worktree-provision/src/lib.rs) defines `WorktreeGit` and `WorktreeRef`; [workflow-tools/session/crates/session-worktree-provision/src/policy.rs](../../../workflow-tools/session/crates/session-worktree-provision/src/policy.rs) defines `ProvisionPolicy`, `SessionActivity`, `evaluate_reclaim_candidate`, and `provision_for_session`.

## Naming Conventions

The future persisted `component_id` is `worktree-provisioning-policy`. Use
`ProvisionPolicy` and `SessionActivity`; criterion ids use the
`worktree-provision-` prefix. `WorktreeGit` is an implemented library type, but
the typed provider component model remains specified-but-not-built.

## Reading Order

1. [fa6e85c8 Worktree Control Component Pilot](../fa6e85c8-866c-4d53-bb50-b78bd651e8ce/body.md) - composing parent.
2. [66fbd896 Worktree Git Operations](../66fbd896-19d4-4bb7-898c-7cdc76375a5e/body.md) - repository-operations provider.
3. [191ceae7 Worktree Control CLI Lifecycle](../191ceae7-663e-448b-bb04-46f46f38825d/body.md) - consuming CLI.
4. [585aa074 Session Worktree Lifecycle Rewrite](../585aa074-356a-4169-b08b-4e3aba659a72/body.md) - related hybrid-access evidence.
5. [workflow-tools/session/crates/session-worktree-provision/src/policy.rs](../../../workflow-tools/session/crates/session-worktree-provision/src/policy.rs) - policy provider.

## Responsibility

If implemented, consumers can make consistent worktree reclaim and provision decisions through one reusable library boundary.

## Interfaces And Dependencies

`WorktreeGit` supplies repository operations, `SessionActivity` supplies activity state, and `ProvisionPolicy` supplies policy input. `evaluate_reclaim_candidate` and `provision_for_session` consume `&WorktreeGit`; policy does not provide Git operations.

## Behavior

- `worktree-provision-reclaim-decision`: evaluate a candidate from repository, activity, and policy inputs.
- `worktree-provision-session-provisioning`: provision or reuse a worktree through the shared library.
- `worktree-provision-owned-session-checkpoint`: an inactive, singly owned nested worktree may checkpoint its own session artifacts before dirty-state evaluation. Reclaim accepts exactly one verified tool-created checkpoint commit; unrelated or pre-existing ahead commits remain rejected.
- `worktree-provision-session-mirror-checkpoint`: consumers may checkpoint a validated main-checkout mirror for a selected worktree session by staging only that session directory; unrelated main changes remain outside the checkpoint.

## Boundaries And Failure Cases

The library does not parse CLI arguments, dispatch commands, or integrate
gitlinks. This component must not claim that sync or gitlink consume
`ProvisionPolicy` or `provision_for_session`; the accurate Git-operation
provider component is deliberately not introduced by this pilot. Invalid
repository state or activity/policy errors return typed library failures.
Checkpoint eligibility requires the recorded owner id, nested worktree id, and
local session record id to agree, and requires every dirty superproject path to
stay under that session directory. It never checkpoints dirty submodules or
unrelated files. A checkpoint commit is accepted only when its sole parent and
all changed paths prove it is the fixed tool-created session checkpoint.
Main mirror checkpointing also requires the mirror record's session id and
recorded worktree path to match the selected worktree; it never stages other
main-checkout changes.

## Provider/Consumer Contract

Consumes `worktree-git-repository-operations` from [66fbd896 Worktree Git Operations](../66fbd896-19d4-4bb7-898c-7cdc76375a5e/body.md) and provides `worktree-provision-reclaim-decision` to [191ceae7 Worktree Control CLI Lifecycle](../191ceae7-663e-448b-bb04-46f46f38825d/body.md). `worktree-provision-session-provisioning` has no confirmed consumer in this pilot; sync and gitlink have no provisioning-policy or provisioning-function edge.

## Examples

`provision_for_session(&git, activity, &policy, session)` decides reuse/reclaim/create from the same `ProvisionPolicy` regardless of which CLI lifecycle path consumes it.
An inactive worktree containing only its own uncommitted session record is
checkpointed, then eligible for reclamation; a worktree already ahead of local
`main` for any other reason remains ineligible.

## Evidence

Position: `implemented`. Run `cargo test --manifest-path workflow-tools/session/Cargo.toml -p worktree-ctl --test worktree_contracts` and the provisioning crate's focused unit tests when present.

## Scope

Owns reusable provisioning policy/library behavior, not command presentation or integration orchestration.
