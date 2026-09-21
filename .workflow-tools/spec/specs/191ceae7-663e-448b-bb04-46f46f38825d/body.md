<!-- aligned-structure:v2 -->

# Worktree Control CLI Lifecycle

## Target Code Location

[workflow-tools/session/crates/worktree-ctl/src/main.rs](../../../workflow-tools/session/crates/worktree-ctl/src/main.rs) defines `Cli`, `Command`, `dispatch`, and lifecycle handlers; [workflow-tools/session/crates/worktree-ctl/tests/worktree_contracts.rs](../../../workflow-tools/session/crates/worktree-ctl/tests/worktree_contracts.rs) covers lifecycle contracts.

## Naming Conventions

The future persisted `component_id` is `worktree-control-cli`. Use `Cli`,
`Command`, and `dispatch`; criterion ids use the `worktree-cli-` prefix,
including `worktree-cli-evaluates-reclaim-candidate`.

## Reading Order

1. [fa6e85c8 Worktree Control Component Pilot](../fa6e85c8-866c-4d53-bb50-b78bd651e8ce/body.md) - composing parent and parent-owned criteria.
2. [66fbd896 Worktree Git Operations](../66fbd896-19d4-4bb7-898c-7cdc76375a5e/body.md) - Git adapter operations provider.
3. [c1d13a73 Worktree Provisioning Policy](../c1d13a73-3265-42e1-8da0-5c44ef7b61ff/body.md) - reclaim-evaluation provider.
4. [585aa074 Session Worktree Lifecycle Rewrite](../585aa074-356a-4169-b08b-4e3aba659a72/body.md) - related hybrid-access evidence.
5. [workflow-tools/session/crates/worktree-ctl/src/main.rs](../../../workflow-tools/session/crates/worktree-ctl/src/main.rs) - implemented dispatch boundary.

## Responsibility

If implemented, operators can invoke supported lifecycle commands and rely on
the CLI dispatch layer to use the implemented Git and reclaim-evaluation calls
without claiming session provisioning delegation.

## Interfaces And Dependencies

`Cli` parses a `Command`; `dispatch(Command)` selects lifecycle handlers.
`main.rs` consumes Git adapter operations from `WorktreeGit` across lifecycle
handlers and consumes `ProvisionPolicy` with `evaluate_reclaim_candidate` only
for reclaim display; it does not call `provision_for_session`.

## Behavior

- `worktree-cli-dispatches-lifecycle`: each supported command reaches its intended handler.
- `worktree-cli-evaluates-reclaim-candidate`: the implemented reclaim path uses `evaluate_reclaim_candidate` with `ProvisionPolicy`; it does not establish a session-provisioning edge.
- `worktree-cli-contract-coverage`: integration tests exercise user-visible lifecycle behavior.
- `worktree-cli-selects-worktrees`: `clean`, `commit`, `rebase`, `merge`, and `sync` accept one or more positional selectors or repeatable `--worktree <selector>` options. `--all` selects every managed worktree and is mutually exclusive with explicit selectors.
- `worktree-cli-commits-selected-paths`: `commit` stages all changes by default, or stages trailing `-- <pathspec>...` arguments with `git add` semantics before committing each selected worktree.
- `worktree-cli-cleans-inactive-worktrees`: `clean` removes only selected clean worktrees that have no commits ahead of local `main`; a fully behind worktree is removable.
- `worktree-cli-checkpoints-owned-session-records`: before clean, rebase, or merge mutates a worktree, the CLI may create a fixed-message checkpoint commit only when every dirty superproject path is beneath `.session/sessions/<owner-session-id>/`, the nested worktree UUID and the registered owner agree, and that local `session.json` declares the same id. Active worktrees are preserved by clean even when checkpointable.
- `worktree-cli-checkpoints-session-mirrors`: sync checkpoints a dirty validated main-checkout mirror of the selected session before rebasing the worktree. It stages only `.session/sessions/<owner-session-id>/`, leaving unrelated main changes for the existing stash/restore guard, so the mirror cannot collide with its own later fast-forward.

## Boundaries And Failure Cases

The CLI does not own provisioning policy, does not call `provision_for_session`,
and does not invent an MCP/API parity surface. Parse failure, invalid checkout,
rejected reclaim decision, invalid selector combination, or library error is
surfaced as command failure. Batch commands stop on the first failed selected
worktree, leaving later selections untouched.
An unrelated dirty superproject path, any dirty submodule, a missing or malformed
session record, ambiguous ownership, or an owner/path mismatch prevents the
session-record checkpoint and retains the normal dirty-worktree failure.
The main mirror checkpoint uses the same owner and worktree-path validation;
it does not stage unrelated main-checkout changes.

## Provider/Consumer Contract

Consumes `worktree-git-repository-operations` from [66fbd896 Worktree Git Operations](../66fbd896-19d4-4bb7-898c-7cdc76375a5e/body.md) and `worktree-provision-reclaim-decision` from [c1d13a73 Worktree Provisioning Policy](../c1d13a73-3265-42e1-8da0-5c44ef7b61ff/body.md) only for reclaim evaluation; provides lifecycle behavior to operators. This is documented intended provider/consumer evidence, not a persisted edge until the typed model exists.

## Examples

`worktree-ctl commit <worktree> -- src/lib.rs` commits only `src/lib.rs` in the
selected worktree. `worktree-ctl clean --all` safely removes every eligible
inactive worktree. A lifecycle command reaches `dispatch`, opens `WorktreeGit`,
and the reclaim path calls `evaluate_reclaim_candidate` with
`ProvisionPolicy::default()`; it does not call `provision_for_session`.

## Evidence

Position: `partial`: the named Git and reclaim-evaluation calls are implemented,
but the persisted component/criterion/edge model is not. Run `cargo test --manifest-path workflow-tools/session/Cargo.toml -p worktree-ctl --test worktree_contracts`.

## Scope

Owns CLI dispatch, selection, cleanup, and commit behavior, not low-level
synchronization writes, gitlink checks, or policy semantics.
