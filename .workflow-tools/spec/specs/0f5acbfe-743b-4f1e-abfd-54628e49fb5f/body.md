Legacy content intentionally omitted; body written via sections below plus a concise summary.

# Summary

MCP tool calls arrive tagged with a `session_id`. Each agent session works out of its own git worktree under `.worktrees/`, but tool calls were resolving to files in the main checkout instead of the session's worktree — an agent would read and write the wrong tree. This spec covers the fix: a `SessionWorkspaceResolver` that discovers the correct worktree for a `session_id` via git-truth validation and filesystem discovery rather than trusting a possibly-stale session record, plus the supporting fixes needed to keep that record from getting poisoned again and to stop a related MCP transport (`compact-terminal-mcp`) from wedging the server the resolver's own MCP surface runs on.

## Behavior Story

An agent session identified by `session_id` calls an MCP tool (ticket, spec, terminal, etc.). The tool's workspace-resolution layer must land the operation in that session's git worktree, not in the main checkout, even when:

- the session's stored record still points at the main checkout (a record poisoned by a prior bug in the capture hook),
- the worktree was created by hand after the session started and no record yet mentions it,
- the record's `branch` field was hand-edited or is stale relative to the worktree's actual git state.

See the `resolution-chain` section for the exact algorithm as implemented in `memory-api/crates/session-workspace-resolver/src/lib.rs`.

## Provided Surface Contracts

See the `resolution-chain`, `capture-hook-fix`, `check-in-ownership-fix`, and `terminal-subprocess-isolation` sections.

## Required Validation

See the `validation-evidence` section for the full test inventory and live CLI verification.

## Related Implementation Tickets

- `a1b911ab-9394-4ba8-9134-1b2687e96ccd` — session-id worktree discovery in the resolver, plus the capture-hook fix. Delivered.
- `fd374421-f72f-4175-9daf-c47d387e7a01` — terminal subprocess stdin/timeout isolation. Delivered.
- `5e6cf4f8-120c-4674-95de-d7b79c99f5b3` — eager worktree creation and a Rust rewrite of `tools/worktree/worktree.sh`. **Not implemented** — see `out-of-scope` section.

## Background Knowledge References

- `memory-api/crates/session-workspace-resolver/src/lib.rs`
- `memory-api/crates/session-capture-hook/src/main.rs`
- `memory-api/crates/session-api/src/store/config/worktree_runtime.rs`
- `memory-api/crates/compact-terminal-api/src/execute.rs`
