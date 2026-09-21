<!-- aligned-structure:v2 -->

# Worktree Git Operations

## Target Code Location

[workflow-tools/session/crates/session-worktree-provision/src/lib.rs](../../../workflow-tools/session/crates/session-worktree-provision/src/lib.rs) defines the concrete public `WorktreeGit` adapter and `WorktreeGitError`; [workflow-tools/session/crates/session-worktree-provision/src/policy.rs](../../../workflow-tools/session/crates/session-worktree-provision/src/policy.rs) consumes `&WorktreeGit`; [workflow-tools/session/crates/worktree-ctl/src/main.rs](../../../workflow-tools/session/crates/worktree-ctl/src/main.rs), [sync.rs](../../../workflow-tools/session/crates/worktree-ctl/src/sync.rs), and [gitlink.rs](../../../workflow-tools/session/crates/worktree-ctl/src/gitlink.rs) are its direct consumers.

## Naming Conventions

The immutable future persisted `component_id` is `worktree-git-operations`.
Use `WorktreeGit`, `WorktreeGitError`, and `WorktreeRef`; provider criterion
ids use the `worktree-git-` prefix, including
`worktree-git-repository-operations`, `worktree-git-inspection-metadata`, and
`worktree-git-submodule-path-enumeration`.

## Reading Order

1. [fa6e85c8 Worktree Control Component Pilot](../fa6e85c8-866c-4d53-bb50-b78bd651e8ce/body.md) - composing parent and scoped provider graph.
2. [585aa074 Session Worktree Lifecycle Rewrite](../585aa074-356a-4169-b08b-4e3aba659a72/body.md) - related hybrid `git2`-read and subprocess-write access contract.
3. [191ceae7 Worktree Control CLI Lifecycle](../191ceae7-663e-448b-bb04-46f46f38825d/body.md) - lifecycle consumer.
4. [c1d13a73 Worktree Provisioning Policy](../c1d13a73-3265-42e1-8da0-5c44ef7b61ff/body.md) - repository-operations consumer.
5. [c40b790f Worktree Synchronization And Integration](../c40b790f-6704-4a5e-bc62-ae7599521a7c/body.md) - inspection-and-metadata consumer.
6. [a623ea02 Worktree Gitlink Integrity](../a623ea02-e1a9-4c8c-81ea-f1f5fb3b4a9f/body.md) - submodule-path-enumeration consumer.

## Responsibility

If implemented, consumers can rely on `WorktreeGit` as the concrete public
adapter that opens local repositories, reads worktree and branch state, and
performs the adapter-owned local Git writes needed for worktree lifecycle
operations. This component specifies implemented code behavior; typed
component/criterion/edge persistence is specified-but-not-built.

## Interfaces And Dependencies

`WorktreeGit::open` owns repository opening. Its `git2` reads enumerate
worktrees, check branches, dirty paths, ahead/behind state, gitlinks, and
`.gitmodules` submodule paths. Its write methods invoke `git` subprocesses for
worktree creation/removal/pruning, branch rename/delete, stashing, offline
submodule bootstrap, filesystem relocation plus repair, and rollback. It is a
struct, not a trait and not `ProvisionPolicy`. `ProvisionPolicy`,
`evaluate_reclaim_candidate`, and `provision_for_session` consume
`&WorktreeGit`.

## Behavior

- `worktree-git-repository-operations`: open the main checkout and provide
	worktree/branch/status/gitlink inspection plus adapter-owned lifecycle writes.
- `worktree-git-inspection-metadata`: expose worktree metadata, dirty state,
	ahead/behind state, and submodule paths to consumers without asserting that
	every repository mutation uses this adapter.
- `worktree-git-submodule-path-enumeration`: enumerate `.gitmodules` paths for
	gitlink behavior; it does not own containment classification.
- `worktree-git-relocation-recovery`: relocate a worktree in place, repair
	registrations, and roll back a failed relocation or bootstrap.

## Boundaries And Failure Cases

`sync.rs` uses local helpers for rebase, merge, and stash writes, so this
component must not claim all repository mutation routes through `WorktreeGit`.
`gitlink.rs` uses `WorktreeGit` only to enumerate submodule paths and opens
`git2::Repository` directly for containment/classification. Open, repository,
subprocess, offline-bootstrap, repair, and rollback failures surface as
`WorktreeGitError`; a failed partial create or relocation attempts rollback.

## Provider/Consumer Contract

Provides `worktree-git-repository-operations` to [191ceae7 Worktree Control CLI Lifecycle](../191ceae7-663e-448b-bb04-46f46f38825d/body.md) and [c1d13a73 Worktree Provisioning Policy](../c1d13a73-3265-42e1-8da0-5c44ef7b61ff/body.md); provides `worktree-git-inspection-metadata` to [c40b790f Worktree Synchronization And Integration](../c40b790f-6704-4a5e-bc62-ae7599521a7c/body.md); and provides `worktree-git-submodule-path-enumeration` to [a623ea02 Worktree Gitlink Integrity](../a623ea02-e1a9-4c8c-81ea-f1f5fb3b4a9f/body.md). These are documented intended edges, not persisted typed edges until the model exists.

## Examples

`list_worktrees_and_branch_existence_use_git_metadata` proves enumeration and
branch checks; `dirty_and_ahead_behind_report_worktree_state` and
`stash_push_creates_an_entry_with_the_requested_message` prove state and stash
behavior. `create_worktree_populates_submodules_and_reports_gitlink` and
`create_worktree_rolls_back_when_submodule_commit_is_unavailable` cover offline
bootstrap and rollback. `worktree_move_repairs_a_worktree_containing_a_submodule`,
`worktree_move_rolls_back_when_nested_repair_fails`,
`rename_worktree_moves_in_place_and_preserves_marker`, and
`remove_and_prune_clear_worktree_registration` cover relocation/repair and
rename/remove/prune.

## Evidence

Position: `partial`: `WorktreeGit` and its source/test behavior are
implemented; the immutable typed component identity, provider criteria, and
edges are specified-but-not-built. Run `cargo test --manifest-path
workflow-tools/session/Cargo.toml -p session-worktree-provision` and `cargo
test --manifest-path workflow-tools/session/Cargo.toml -p worktree-ctl --test
worktree_contracts --test maintenance`. Evidence anchors are [lib.rs](../../../workflow-tools/session/crates/session-worktree-provision/src/lib.rs), [worktree_contracts.rs](../../../workflow-tools/session/crates/worktree-ctl/tests/worktree_contracts.rs), and [maintenance.rs](../../../workflow-tools/session/crates/worktree-ctl/tests/maintenance.rs). Related lifecycle ticket: [5e6cf4f8 Rewrite worktree.sh as a Rust binary and add worktree lifecycle recycling](../../../.ticket/tickets/5e6cf4f8-120c-4674-95de-d7b79c99f5b3/ticket.toml).

## Scope

Owns the provider contract for the existing `WorktreeGit` adapter. It does not
own provisioning decisions, CLI dispatch, synchronization write helpers,
gitlink containment/classification, or typed persistence implementation.
