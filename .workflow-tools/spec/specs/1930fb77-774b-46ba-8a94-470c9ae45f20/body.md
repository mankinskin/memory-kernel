<!-- aligned-structure:v2 -->
<!-- migrated-from: context-engine/.spec/specs/585aa074-356a-4169-b08b-4e3aba659a72 (spec-api MissingSourceEntity defect prevented move primitive; recreated via create) -->

# Session Worktree Lifecycle Rewrite

## Motivation

The session component needs a Rust replacement for `tools/worktree/worktree.sh` so worktree lifecycle operations preserve repository and submodule invariants while allowing completed, disposable worktrees to be reused without losing locally built artifacts or entity-store indexes.

## Dependent expectation

If this specification is implemented, dependents can rely on session worktrees being locked during active session ownership, automatically preserved when they contain user work, and reclaimed in place only when their completed state is clean and fully reachable from local `main`.

## Scope

The Rust `worktree-ctl` binary owns local worktree lifecycle operations. The binary supports `new`, `list`, `rebase`, `merge`, `remove`, `rename`, `finish`, and `doctor`.

All operations use local `main` and recorded local git objects. No operation fetches from or otherwise depends on `origin`.

### Subcommand contract

- `new <short-id> <slug>` creates or reuses `.worktrees/<short-id>-<slug>` with branch `agent/<short-id>-<slug>` from local `main`, populates every recorded submodule offline through linked submodule worktrees, and rolls back a failed partial bootstrap. A repeated request for the same session identity reuses the existing worktree. A second worktree for an identity is rejected unless `--allow-additional` is explicit. When the main checkout has tracked changes, creation refuses and identifies the changes unless `--preserve-main-changes` explicitly stashes and later allows restoration of the changes.
- `list` reports registered worktrees and their lifecycle-relevant state without mutating Git state.
- `rebase <name>` rebases the named feature branch onto local `main`; conflicts stop the operation for human resolution and are never auto-resolved or auto-aborted.
- `merge <name>` first fast-forwards every branch-bearing nested submodule worktree into the corresponding local submodule `main`, skips detached nested worktrees, then fast-forwards the superproject local `main` from `agent/<name>`. A non-fast-forward condition fails without merging the superproject.
- `remove <name>` removes a completed worktree only when clean, unless an explicit force operation is requested. A normal removal identifies blocking dirty paths and preserves the worktree. Successful force removal uses `git worktree remove --force`, prunes registrations, and deletes only a merged branch with `git branch -d`.
- `doctor` diagnoses and repairs worktree registration and submodule initialization damage without deinitializing a submodule.

### Lifecycle state machine

- `active/locked -> complete`: session ownership ends or becomes inactive; no automatic purge, rename, or repurpose is allowed while a session remains active.
- `complete -> preserved`: preserve when any tracked or untracked worktree change exists, any nested submodule is dirty, the worktree has commits not reachable from local `main`, the worktree lacks a branch, the process current directory is inside the worktree, or the worktree has not been idle longer than `WORKTREE_IDLE_SECS` (default `86400`).
- `complete -> reclaimed`: reclaim only when there is no session-store activity, the worktree has a branch, the worktree and all nested submodules are clean, the current directory is outside the worktree, the idle threshold has elapsed, and the branch is zero commits ahead of local `main`.
- `reclaimed -> active/locked`: reuse changes the worktree topic and branch in place for the incoming session, then records the new active ownership.
- `preserved -> active/locked`: an explicit agent re-topicing decision may claim the preserved worktree; automatic repurposing is prohibited.

### Reclamation and preservation

Reclamation is a filesystem rename/move in place followed by `git worktree repair` and branch rename. Remove-and-recreate is forbidden because it discards built artifacts, `target/` contents, and rebuilt entity-store indexes. The in-place path preserves those assets and must also preserve any submodule commit object that is ahead of the recorded gitlink.

`git worktree move` is prohibited because linked worktrees in this repository contain five submodules and Git rejects such a move. The required alternative is filesystem relocation followed by `git worktree repair`, with nested `git worktree repair` only when a submodule remains unregistered after top-level repair.

`git submodule deinit` is prohibited during teardown. The command rewrites shared `.git/config` state and can silently deinitialize submodules in the main checkout. `git worktree remove --force` handles initialized submodules without deinitialization.

### Git access boundary

The binary uses libgit2 for every read: repository opening, worktree enumeration, branch existence, dirty-state checks, ahead/behind evaluation, gitlink lookup, and `.gitmodules` parsing.

The binary uses the `git` subprocess only for writes that libgit2 cannot express: `git worktree add -b`, `git worktree add --detach`, filesystem relocation followed by `git worktree repair`, `git branch -m`, `git worktree remove --force`, `git worktree prune`, `git branch -d` or `git branch -D`, and nested `git worktree repair`.

### Dry run

Every mutating lifecycle action, including `new`, `rebase`, `rename`/re-topic, `merge`, `remove`, `finish`/completion handling, and `doctor`, accepts `--dry-run`. A dry run emits its local-Git plan, makes no filesystem or Git mutation, changes neither worktree paths nor local `main`, and never references `origin`.

## Non-goals

- Rewriting or extending the retiring Bash implementation to satisfy lifecycle behavior.
- Fetching, comparing against, or otherwise requiring `origin` or `origin/main`.
- Automatic reclamation of a dirty worktree, a worktree with unreachable commits, or a worktree owned by an active session.
- Remove-and-recreate reclamation, `git worktree move`, or `git submodule deinit`.
- Automatic conflict resolution for rebase or non-fast-forward merge.
