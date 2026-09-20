# Summary

`ticket-api` and `session-api` resolve a store from one authoritative checkout
anchor. Resolution is containment-based: it selects the checkout that contains
the invocation working directory, rather than enumerating sibling stores and
choosing among them.

## Scope

This contract applies to ticket and session store resolution for CLI, API, and
MCP entry points that accept a workspace selector or infer one from the
invocation environment. It covers `.ticket`, `.session`, and their associated
index roots where applicable.

## Anchoring Contract

Resolution uses the following precedence order:

1. An explicit workspace selector identifies the authoritative checkout and
   its store, subject to normal path validation.
2. Without an explicit selector, a working directory contained by a linked
   worktree selects that containing worktree and its store.
3. Otherwise, a working directory contained by the repository root selects the
   repository-root store.

Sibling directories below `.worktrees/*` are never candidates when the working
directory is not contained by that sibling. A resolver MUST derive the target
from containment, not by listing worktrees or collecting their stores into a
candidate set. Ambiguous containment or an explicit selector outside the
allowed repository boundary is an error; a resolver MUST NOT silently choose a
store.

## Index and Store Coupling

An explicit `--workspace <repo-root>` is sufficient to select the
repository-root store and index: absent `--index-root`, the CLI defaults the
index to the selected workspace's `.ticket` root. An explicit `--index-root`
overrides `--workspace`; this behavior is documented at the CLI boundary. The
CLI MUST NOT permit `--workspace` to select one store while an implicit index
root selects another.

## Bootstrap Exemption

`session_check_in` is exempt from the pre-existing-worktree-assignment anchor
precheck only for the operation that creates or updates that session's
`SessionWorktreeAssignment`. The exemption permits the assignment to be
created when none exists, but still requires a valid check-in request, a
validated target path within the resolved repository worktree namespace, and
the existing ownership and active-worktree-conflict checks.

All other session operations that resolve a `default` workspace continue to
require an established assignment. The bootstrap exemption does not authorize
an arbitrary unanchored store selection, an assignment outside the repository,
or bypassing session ownership validation.

## Acceptance Criteria

1. A root-invoked ticket or session write resolves to the repository-root
   store even when at least fifteen `.worktrees/*` directories each contain a
   nested store.
2. From a repository-root invocation, no nested `.worktrees/*/.ticket` or
   `.worktrees/*/.session` path is considered a resolution candidate.
3. From a working directory contained by `.worktrees/<name>`, resolution
   selects that worktree's corresponding store when no explicit workspace
   selector overrides the target.
4. An explicit workspace selector takes precedence over inferred containment,
   and invalid or escaping selectors fail without a fallback store choice.
5. `--workspace <repo-root>` pins the ticket store and either pins the index
   by default or produces the documented pre-write error required by the Index
   and Store Coupling decision.
6. `session_check_in` creates a first assignment for an otherwise unassigned
   session without failing its own anchor precheck, while malformed,
   conflicting, or out-of-bound assignment requests remain rejected.
7. Resolver tests prove containment-based selection directly and do not rely
   on worktree enumeration order.

## Required Validation

- A `session-workspace-resolver` fixture creates a root store and at least
  fifteen nested worktree stores, then proves a root-invoked write lands only
  in the root store.
- Focused unit or integration coverage exercises explicit selector precedence,
  repository-root fallback, and containing-worktree selection for both ticket
  and session store paths.
- `session-api` coverage proves first-time `session_check_in` succeeds and
  preserves ownership, path-boundary, and active-worktree-conflict validation.
- Planned commands: `cargo test --manifest-path memory-api/Cargo.toml -p
  session-workspace-resolver`; `cargo test --manifest-path
  memory-api/Cargo.toml -p session-api`; and the focused `ticket-api` or
  `ticket-cli` regression test containing the root-write fixture.

## Non-Goals

- Changing worktree provisioning policy.
- Changing `worktree.sh`, `worktree-ctl`, or any worktree lifecycle behavior.
- Changing unrelated work in epic `db6980d1` beyond deterministic store
  resolution and the `session_check_in` bootstrap exemption.

## Traceability

- Implementation ticket: [0afe45b5 Store resolution enumerates .worktrees and mis-anchors the active store](.ticket/tickets/0afe45b5-9ec8-4f4a-af74-f46f06cc7516/ticket.toml).
- Epic: [db6980d1 Worktree provisioning and session-worktree lifecycle](.ticket/tickets/db6980d1-38bf-4819-8c07-b6db09229c1c/ticket.toml).
- Related specification: `context-engine/mcp/session-anchored-workspace-resolution` defines MCP session routing; this specification defines the local store-resolution contract that must not enumerate sibling worktrees.