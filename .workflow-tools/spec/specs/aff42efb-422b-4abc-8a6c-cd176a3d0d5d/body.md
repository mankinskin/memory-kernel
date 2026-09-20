# Summary

Every MCP tool call resolves its target workspace from the calling SESSION's active worktree assignment, not from the server process working directory. The anchor is enforced once, in the `mcp-toolmon` proxy, so it applies uniformly to every wrapped server rather than being re-implemented per server.

This spec is the cross-cutting protocol layer for the parent spec `context-engine/session-worktree-default-workflow`, which establishes that worktree assignment is authoritative in `session-api` but provides no enforcement point.

## Behavior Story

An agent works in a linked worktree at `.worktrees/<name>`. It calls an MCP tool to update a ticket. The record must land in that worktree's store — the store whose files the agent can see, commit, and review — and not in the main checkout the long-lived server process happens to have been started from.

## Problem

The MCP servers are long-lived processes started by the editor. Their working directory is the main checkout and never changes, regardless of where the agent is working.

- `memory-api/tools/mcp/ticket-mcp/src/server.rs#L100-L106` — an empty or `"default"` workspace selector short-circuits to the server's own index root.
- `memory-api/crates/memory-api/src/workspace.rs#L108-L114` — that index root derives from `current_dir()`/`PWD`, meaning the SERVER's cwd.
- `memory-api/tools/mcp/ticket-mcp/src/main.rs#L14-L20` — the root comes from `TICKET_INDEX_ROOT`, else this cwd-derived resolution.
- `.vscode/mcp.json#L16-L22` — the server is launched with neither `cwd` nor `TICKET_INDEX_ROOT` set, so the cwd-derived path is what is actually used.

Consequently an agent in a worktree writes through MCP into the MAIN checkout's store while its CLI and file edits write into the worktree's store. The two stores diverge silently: no error, no warning, and the divergence is only discovered later as missing or duplicated records. Because `.ticket`, `.spec`, and `.session` are all version-controlled, every worktree has its own copy of each, so this affects every store, not just tickets.

## Contract

### Session id is the anchor

- Every proxied tool call carries a REQUIRED `session_id` argument.
- The proxy resolves `session_id` to the session's active `SessionWorktreeAssignment`, and that assignment's path determines the workspace for the call.
- Process working directory is never a fallback. A call whose workspace cannot be resolved through a session anchor is REJECTED with an error naming the missing anchor and the candidate store roots it refused to choose between.
- The bare `default` selector is honoured only when it resolves through a session anchor. Unanchored, it is an error rather than a silent choice.

### Enforced in the proxy, once

`mcp-toolmon` already performs exactly this shape of work for a different required field:

- `memory-api/tools/mcp/mcp-toolmon/src/proxy.rs#L203` — reads and validates a required `caller_model` argument on every call.
- `memory-api/tools/mcp/mcp-toolmon/src/proxy.rs#L284` — strips `caller_model` before forwarding to the wrapped server, and injects the required property into `tools/list` responses so every advertised tool declares it.

`session_id` uses the same mechanism. Because the proxy fronts every server, a single implementation covers ticket, spec, session, test, feedback, audit, doc, and log surfaces. A fix applied inside one server does not satisfy this spec.

### The session already knows its worktree

- `memory-api/crates/session-api/src/model.rs#L316` — session metadata carries a `worktree` field.
- `memory-api/crates/session-api/src/model.rs#L336` — `SessionWorktreeAssignment { path, branch, allocation_mode: New | Reused | Rotated, status: Active | Superseded | Invalidated, predecessor_session_id, predecessor_path }`.
- `memory-api/crates/session-api/src/store.rs#L171-L199` — `check_in_worktree` mutates the assignment mid-session, so resolution must read the CURRENT assignment on each call rather than caching one fixed at session start.

Resolution therefore reads the CURRENT assignment on each call rather than caching one fixed at session start.

### All active stores are worktree-local

`.session`, `.ticket`, and `.spec` all live inside the session's worktree. The main checkout holds no active store — its copies are a merge target, current only once a branch merges. Session-anchored resolution consequently points every store at the same place, so there is no per-store special case and no store whose authority sits elsewhere.

This is bootstrappable because worktree initialization is the FIRST action of a chat session: a chat lifecycle hook creates and initializes the session's worktree before any other tool runs. There is no interval in which a session exists without a worktree, so resolution never needs a main-checkout fallback and no such fallback is specified. The ordering guarantee is a real dependency of this spec, not an incidental convenience.

The matching worktree anchoring for capture is specified in `context-engine/session-api/vscode-copilot-capture-hook-sync` and implemented by ticket `40349f3f-8d04-4bf6-9241-b79425c10a97`.

### Typed scope

The resolved workspace is a typed value, not a bare path. It distinguishes at least:

- the repository root,
- a worktree inside that repository,
- the main (non-worktree) checkout.

This is what makes a policy expressible that BLOCKS mutating work resolved to the main checkout — the failure this spec exists to prevent, stated as an enforceable rule rather than a convention.

### Nested workspaces

An OPTIONAL relative workspace path parameter addresses a workspace nested deeper inside the session's worktree. It is interpreted relative to the resolved worktree root, never relative to the server working directory, and it cannot escape that root.

### Non-Git workspaces

Memory API permits stores outside a git checkout, and that remains supported. A non-Git workspace is explicitly opt-in via an explicit selector; it is never the default and is never reached by falling through from a failed git resolution.

## Required Validation

Proxy-level tests covering:

1. A call with no resolvable `session_id` is rejected.
2. A call whose session resolves to a worktree writes to THAT worktree's store while the server process working directory is the main checkout.
3. An unanchored `default` selector is rejected rather than resolved.
4. A mutation resolved to the main checkout is blocked under the enforcing policy.
5. The required property appears in `tools/list` schemas and is stripped before forwarding, matching the existing `caller_model` handling.
6. `.session`, `.ticket`, and `.spec` all resolve to the same worktree root for one session id, with no per-store divergence.

## Non-Goals

- Creating, renaming, or removing git worktrees. That lifecycle is specified in the parent spec and implemented by ticket `ff83caf7-059b-4f2e-a0fb-eaa7757096a8`.
- Index freshness against external store writes. `ticket-mcp` never starts the reconciler (`memory-api/tools/mcp/ticket-mcp/src/server.rs#L702-L708`, `memory-api/crates/ticket-api/src/watcher/reconciler.rs#L66-L74`); that defect is tracked separately as ticket `35a60203-0a2c-4dbc-b33d-b645848871f2`.
- Deciding WHEN a session bootstraps a worktree, which is the prompt-time hook in ticket `3d535b2c-7361-4f08-bfb4-63b0b3174afc`. This spec DEPENDS on that ordering (worktree first, before any other tool) but does not specify it.

## Open Questions

- Whether the Copilot hook API exposes a `SessionStart` event is UNVERIFIED. `.github/hooks/hooks.json` currently maps only `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `Stop`, and `SessionEnd`. Implementation must not assume a `SessionStart` hook exists.
- The per-call routing mechanism remains an open design choice among argument rewriting, a per-server routing context, and a per-tool adapter. This spec fixes the CONTRACT (session-anchored, proxy-enforced, rejection over silent default) and leaves the mechanism to implementation.

## Related Implementation Tickets

- `fa2ba34b-59ec-4321-a4ce-a3c3a9295ea3` — session-anchored MCP workspace resolution (implements this spec).
- `40349f3f-8d04-4bf6-9241-b79425c10a97` — worktree-anchored capture (companion; same anchoring at the hook boundary).
- `ff83caf7-059b-4f2e-a0fb-eaa7757096a8` — managed session-worktree lifecycle.
- `c060bf94-2435-4cc5-8016-ca1d2c8264f5` — board/session binding and session activity listing.
- `35a60203-0a2c-4dbc-b33d-b645848871f2` — index staleness (adjacent, not covered here).

## Background Knowledge References

- Parent spec: `context-engine/session-worktree-default-workflow` (`2860a8db-0c4e-4e94-984a-c10a72a67ffc`).
- Listing projection: `ticket-api/board/session-worktree-binding` (`10dee1dc-ab34-4e16-810b-b0c20a7677b7`).
- Capture anchoring: `context-engine/session-api/vscode-copilot-capture-hook-sync` (`09f96d83-4795-4f19-9259-64ad0d452387`).
- Design source: `transcripts/06-08-2026_worktree-session-proxy/merged.clean.md`.
