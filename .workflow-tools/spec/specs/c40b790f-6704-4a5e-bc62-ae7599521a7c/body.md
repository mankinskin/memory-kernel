<!-- aligned-structure:v2 -->

# Worktree Synchronization And Integration

## Target Code Location

[workflow-tools/session/crates/worktree-ctl/src/sync.rs](../../../workflow-tools/session/crates/worktree-ctl/src/sync.rs) defines `handle_sync`; [workflow-tools/session/crates/worktree-ctl/tests/maintenance.rs](../../../workflow-tools/session/crates/worktree-ctl/tests/maintenance.rs) exercises integration and merge behavior.

## Naming Conventions

The future persisted `component_id` is `worktree-synchronization`. Use
`handle_sync` and `worktree-sync-` criterion ids, including
`worktree-sync-preserves-gitlinks`.

## Reading Order

1. [fa6e85c8 Worktree Control Component Pilot](../fa6e85c8-866c-4d53-bb50-b78bd651e8ce/body.md) - composing parent.
2. [66fbd896 Worktree Git Operations](../66fbd896-19d4-4bb7-898c-7cdc76375a5e/body.md) - inspection-and-metadata provider.
3. [a623ea02 Worktree Gitlink Integrity](../a623ea02-e1a9-4c8c-81ea-f1f5fb3b4a9f/body.md) - containment-and-status provider.
4. [585aa074 Session Worktree Lifecycle Rewrite](../585aa074-356a-4169-b08b-4e3aba659a72/body.md) - related hybrid-access evidence.

## Responsibility

If implemented, synchronization integrates worktree changes using Git Operations for inspection and metadata while preserving required gitlink containment and status checks.

## Interfaces And Dependencies

`handle_sync` opens and consumes `WorktreeGit` for inspection and worktree
metadata; its integration path consumes gitlink verification/status partitioning
before committing rebased gitlinks. Local helpers perform rebase, merge, and
stash writes.

## Behavior

- `worktree-sync-uses-git`: use `WorktreeGit` for inspection and worktree metadata.
- `worktree-sync-preserves-gitlinks`: invoke gitlink integrity behavior before integration completes.
- `worktree-sync-integration-coverage`: maintenance tests cover successful and rejected integration paths.
- `worktree-sync-orders-batches`: batch `sync` orders its selected worktrees by filesystem modification time, oldest first, and completes each worktree's rebase followed by merge before moving to the next.
- `worktree-sync-respects-domain-merge-policy`: sync relies on the installed `session-record-merge` driver only for `.session/sessions/**/session.json`, `transcript.json`, and `events.json`; it does not apply generic automatic resolution to ticket artifacts.

## Boundaries And Failure Cases

Synchronization does not define reclaim policy or CLI parsing, and it does not route every repository mutation through `WorktreeGit`. Git/open failures and invalid gitlink states prevent mutation; a failed integrity decision cannot be bypassed by sync.
Batch sync stops at the first failed rebase or merge, so users resolve only one
worktree conflict at a time and later worktrees remain untouched.

Ticket artifacts require a separate ticket-store-owned reconciliation capability before sync can resolve their conflicts automatically. That capability must merge a ticket bundle (`ticket.toml`, `history.ndjson`, and referenced parts) through typed parsing and validation: it may deduplicate or preserve independently appended history and distinct part identities, but it must fail closed when both sides make incompatible edits to a scalar ticket field or competing amendment lineage. A per-file text merge driver is insufficient because it cannot ensure the resulting bundle is internally consistent.

## Provider/Consumer Contract

Consumes `worktree-git-inspection-metadata` from [66fbd896 Worktree Git Operations](../66fbd896-19d4-4bb7-898c-7cdc76375a5e/body.md) and `worktree-gitlink-containment` from [a623ea02 Worktree Gitlink Integrity](../a623ea02-e1a9-4c8c-81ea-f1f5fb3b4a9f/body.md) for containment/status. It does not consume `ProvisionPolicy` or `provision_for_session`.

## Examples

`handle_sync` performs its integration through `WorktreeGit`; an unresolved gitlink state reported by the integrity component stops the integration path before a commit. Batch sync processes the oldest selected worktree first and stops before attempting the next worktree when that one fails.

The repository configures `session-record-merge` through `setup_git.sh`, and `.gitattributes` selects it for the two mirrored session JSON artifacts. A conflicting ticket manifest/history pair remains a blocking sync failure until the ticket-store reconciler described above is implemented and validates the complete merged ticket bundle.

## Evidence

Position: `implemented`. Run `cargo test --manifest-path workflow-tools/session/Cargo.toml -p worktree-ctl --test maintenance`.

## Scope

Owns synchronization/integration behavior, not lifecycle dispatch or low-level policy decisions.
