## Problem

MCP tool calls carry a `session_id`. Each agent session works in its own git worktree. Tool calls were landing in the main checkout instead of the session's worktree, so an agent read and wrote the wrong files.

## Resolution chain (`SessionWorkspaceResolver::resolve(session_id)`)

Source: `memory-api/crates/session-workspace-resolver/src/lib.rs`.

1. `resolve(session_id)` reads the session record from the **main checkout's** `.session` store only (`session_store()` = `main_checkout/.session`). The worktree's own store is never consulted during resolution.
2. If a receipt exists but its `worktree_path` refers to the same directory as the main checkout, the receipt is **distrusted** — main is never a legitimate answer — and filesystem discovery runs.
3. If the receipt is missing (`MissingWorktreeAssignment` or `NotFound`), discovery runs.
4. `discover_worktree()` scans `<main_checkout>/.worktrees` for a directory whose name starts with the first 8 characters of the session id followed by `-`. Exactly one match wins; zero matches falls back to scanning session records; two or more returns `AmbiguousSessionWorktree`.
5. A discovered candidate is validated as a git checkout and its branch is read live from git HEAD, so a hand-edited `branch` field in a session record is never trusted.
6. Discovery successes are cached; failures are not.
7. If a receipt existed and nothing was discovered, the receipt is honored (it may legitimately point at a worktree that is not under `.worktrees`).
8. If nothing resolves at all, the result is `MissingSessionWorktree`.

Key invariant: a session record whose `worktree_path` equals the main checkout is never trusted as a resolution result — it is always superseded by discovery when discovery succeeds, and used only as a last resort when discovery finds nothing (case 7 only applies to non-main receipts).
