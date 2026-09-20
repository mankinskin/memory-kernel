## Overview

The board-and-worktree isolation protocol (spec `2a710b29-13e9-40b8-8a53-c0ea366bd0bf`, ticket `e38c258e-4502-4a92-95c7-1dac38fd24b7`) requires each implementation session to run in its own git worktree on its own branch cut from `main`. That binding currently exists only as an unvalidated text convention inside the board `intent` field. This spec defines the binding as a first-class, queryable property of a board entry, and defines a read surface that lists active worktrees with their owning sessions.

## Binding Invariant

An active board entry MAY carry a `session_id`, a `worktree_path`, and a `branch`. These three fields are independently optional, but they are constrained together: when an entry carries a `worktree_path`, it MUST also carry the `session_id` that owns that worktree. A `worktree_path` with no owning `session_id` is not a valid binding.

The session store (`SessionWorktreeAssignment` in `memory-api/crates/session-api/src/model.rs`, written by `check_in_worktree`) remains the authoritative source of truth for the session-to-worktree assignment. The board's `session_id`/`worktree_path`/`branch` fields are a queryable projection of that assignment onto the board entry, not a competing or independently-authoritative record. Nothing in this spec requires the board to re-derive or validate the assignment against the session store at write time; it only requires the board to carry and expose the projected values.

## Uniqueness

At most one active board entry may exist per `(worktree_path, session_id)` pair. A second active entry that names the same `worktree_path` under a different `session_id` is a conflict: the checking-in operation MUST report the conflict rather than silently accepting the checked-in entry. This mirrors the existing file-conflict detection behavior of `board_check_in` for `owned_files`, applied to worktree ownership instead of file ownership.

An entry checked in with the same `worktree_path` and the same `session_id` as an existing active entry (e.g. a heartbeat-driven re-check-in or refresh) is not a conflict.

## Discovery

A new read surface enumerates active worktrees, derived entirely from the current active board entries (no independent session-store query is required by this spec, though one MAY be added later as a separate reconciliation feature — see Non-goals). For each distinct `worktree_path` among active entries, the surface reports:
- the worktree path
- the branch recorded against that worktree path
- the owning session id
- the owning agent id
- the set of ticket ids currently being worked in that worktree (an active entry's `worktree_path` may be shared across multiple tickets checked in in the same worktree)

This surface is exposed on both the `ticket` CLI and the ticket-mcp MCP surface, alongside the existing `board_show` command/tool.

## Backward Compatibility

Board entries persisted before these fields existed do not have `session_id`, `worktree_path`, or `branch` recorded. Loading such an entry MUST succeed (no deserialization failure) and MUST report all three fields as absent (`None` / `null`), never as an empty string or other placeholder value that could be mistaken for a real (but empty) value. Absence must be distinguishable from an empty string in both the internal representation and any serialized (JSON/TOML) output.

## Surface Changes

- `BoardEntry` (`memory-api/crates/memory-api/src/storage/board.rs`) gains three optional fields: `session_id: Option<String>`, `worktree_path: Option<String>`, `branch: Option<String>`.
- `board_check_in` (MCP tool and `ticket board check-in` CLI subcommand) accepts the three values as optional arguments and persists them on the created/updated entry.
- `board_show` (MCP tool and CLI) returns the three fields on every reported entry, in both human-readable and machine-readable (JSON/TOON) output.
- A new active-worktree listing command/tool (name to be finalized during implementation) performs the grouping described under Discovery.

## Non-Goals

- Automatically creating or removing git worktrees on disk.
- Verifying that a recorded `worktree_path` exists on the filesystem.
- Reconciling the board against the session store as a background job.
- CI enforcement of the branch-and-worktree protocol.

## Related

- Ticket `c060bf94-2435-4cc5-8016-ca1d2c8264f5` — implementation ticket for this spec.
- Ticket `e38c258e-4502-4a92-95c7-1dac38fd24b7` — landed the branch-and-worktree isolation protocol this spec makes enforceable/observable.
- Spec `2a710b29-13e9-40b8-8a53-c0ea366bd0bf` — the branch-and-worktree isolation protocol spec this spec depends on and complements.
- Instruction file `.agents/instructions/commit/branch-worktree.instructions.md` — the guidance this spec's tooling makes queryable.

## Session Activity Listing Projection (2026-08-06 refinement)

The Discovery section above defines a listing keyed by `worktree_path`. Session-anchored MCP workspace resolution (spec `context-engine/mcp/session-anchored-workspace-resolution`, ticket `fa2ba34b-59ec-4321-a4ce-a3c3a9295ea3`) requires the complementary projection keyed by SESSION, so an agent or human can answer "which sessions exist, what is each working on, and which are still live" without reading the raw store.

### Projection fields

Per session, the listing reports: session id, assigned worktree path, branch, the ticket track (ticket ids associated with the session), an active/inactive flag, and a last-activity timestamp.

### Derived active flag

The active flag MUST be derived, never stored as a second source of truth. It is computed from the worktree assignment `status` (`Active` versus `Superseded`/`Invalidated`) combined with activity recency. Storing it independently would let it drift from the assignment record, which is the authority the resolution protocol resolves against.

### Parameters

- **Filter**: active-only (default) and include-completed. A completed, superseded, or invalidated session is excluded by default and appears only on explicit request.
- **Detail**: a compact form (id, worktree, active, last activity) is the default so routine listing stays token-cheap; a full form adds the ticket track and branch.

### Absent assignments

Consistent with the Backward Compatibility section, a session with no worktree assignment appears in the listing with those fields explicitly absent — never omitted from the listing, and never defaulted to a misleading value.

### Surface

Exposed on both the CLI and the MCP surface, matching the existing active-worktree discovery surface.

### Validation

Tests cover the derived active flag for each assignment status, default exclusion of completed sessions, both detail levels, and a session lacking a worktree assignment.