<!-- aligned-structure:v2 -->

# Motivation

Session-to-worktree assignments are currently duplicated in the main checkout's `.session/sessions/<session-uuid>/session.json`. The duplicate registry makes the main checkout dirty during otherwise isolated worktree sessions, blocks lifecycle tooling that requires a clean main checkout, and can cause live session state to be stashed accidentally. Capture already persists a session's artifacts in the assigned worktree; positional directory discovery removes the duplicate main-checkout registry.

# Dependent Expectation

If this spec is implemented, dependents can rely on session worktrees being discovered from the supported directory layouts without writing a session registry in the main checkout, and on every cross-session reader seeing active records stored in any supported worktree.

# Layout and Discovery Contract

New worktrees use exactly `.worktrees/<full-session-uuid>/<slug>` and branch `agent/<full-session-uuid>/<slug>`. The `<full-session-uuid>` segment is the complete UUID, not its eight-character prefix.

The resolver supports both layouts during transition:

1. For `<session-uuid>`, inspect `.worktrees/<session-uuid>/`. Exactly one immediate slug directory is valid and resolves to that nested worktree.
2. If no nested directory exists, inspect legacy `.worktrees/<first-8-hex-of-session-uuid>-*` directories and validate each candidate by its local `.session/sessions/<session-uuid>/session.json` record.
3. Nested layout has precedence when a valid nested worktree and a valid legacy worktree both describe the same session.
4. If more than one nested slug directory exists for one session UUID, resolution fails with a deterministic ambiguity error listing candidate paths in lexicographic path order; the resolver must not choose a path. Multiple valid legacy candidates fail identically. A malformed or nonmatching candidate does not count as valid.

The main checkout's `.session/sessions/<session-uuid>/session.json` assignment registry is removed. In particular, capture-hook anchor-registry writes and provisioning writes that create or update assignment metadata in the main checkout disappear; positional discovery replaces those writes and no replacement index is introduced. The session record written in the selected worktree remains the local validation evidence for legacy layout only.

# Worktree-Local Session State

No active-session marker exists in either worktree layout. Runtime state is persisted in the UUID-owned `.session/sessions/<session-uuid>/session.json` manifest, and agents supply the Copilot session UUID explicitly from the hook payload. The main checkout does not read or write a marker for worktree discovery or active-session state under this contract.

`.session/` remains git-tracked. Records produced in a worktree are committed on that worktree's feature branch and reach `main` only through the ordinary branch merge path.

# Federated Session Store

A federated enumerator unions records from the main-checkout session store and every discoverable worktree store at `.worktrees/*/.session/sessions/*`, including the nested session-id/slug layout and legacy flat layout. The enumerator returns records in ascending `(session_id, source-path)` order before deduplication.

For duplicate session IDs, the selected record is deterministic: a worktree-local record wins over a main-checkout record; among worktree-local records, the record from the resolver-selected worktree wins; any remaining tie selects the lexicographically smallest canonical source path. The enumerator emits at most one selected record per session ID and records duplicate-source diagnostics for inspection.

`query_sessions`, `sessions_for_ticket`, `tool_metrics`, `backfill_ticket_links`, `find_handoff`, `list_unclaimed_handoffs`, and `SessionStoreActivity` must consume the federated enumerator. `SessionStoreActivity` is safety-critical: reclaim eligibility must account for any activity in all supported worktree stores, so an incomplete scan must conservatively refuse reclaim rather than reclaim a potentially active worktree.

# Transition and Compatibility

Existing flat worktrees are not migrated and must continue to resolve, capture session records, expose active-session markers, and participate in federated reads. The currently live flat directories `.worktrees/16263c13-session`, `.worktrees/3a921774-session`, `.worktrees/23e67e65-worktree-ctl-invariant`, and `.worktrees/99e040a2-session-worktree-registry` are compatibility fixtures.

Dual-layout support may be removed only after no registered or discoverable legacy flat worktree remains and an explicit lifecycle audit records that result. Removing support because a calendar date passes is prohibited.

# Guards

No `ValidationSpec` exists yet. Before this spec may become verified, add guards that execute the acceptance checks below and record their results against ticket [99e040a2 Remove main-checkout session registry with dual-layout worktree discovery](.ticket/tickets/99e040a2-f7fb-40cf-8714-2bc487076d72/ticket.toml).

# Positions

- `partial` — `memory-api/crates/session-workspace-resolver/src/lib.rs`: resolution still reads the main-checkout assignment registry before filesystem discovery and only recognizes the legacy flat prefix.
- `partial` — `memory-api/crates/session-capture-hook/src/main.rs`: capture resolves a worktree-local store after provisioning, but anchor-registry writes still pollute the main checkout.
- `not-implemented` — `memory-api/crates/session-api/src/store/config/capture_query.rs`, `ticket_relation.rs`, `tool_metrics.rs`, `ticket_backfill.rs`, and `handoff_pickup.rs`: cross-session readers do not yet share a federated enumerator.
- `not-implemented` — `memory-api/crates/session-worktree-provision/src/policy.rs`: reclaim activity does not yet prove coverage of every supported worktree-local store.
- `partial` — `tools/worktree/worktree-ctl/src/main.rs` and the session worktree guidance: worktree paths and branch names still use the flat short-id form.

# Governing-Rule Requirement

The session bootstrap and worktree lifecycle guidance must introduce this draft as a coming-soon contract whenever an agent is assigned or resolves a worktree. The updated `.agents/instructions/session/worktree-provisioning.instructions.md`, `.agents/instructions/session/session-identity-and-handoff.instructions.md`, and `.agents/instructions/commit/branch-worktree.instructions.md` must state the nested layout, legacy compatibility, and main-checkout mutation prohibition. No PolicyRule is currently linked; adding the governing rule is required before implementation review.

# Acceptance Criteria

1. A focused resolver test creates `.worktrees/<full-session-uuid>/<slug>` and verifies resolution returns that path and `agent/<full-session-uuid>/<slug>` without reading or writing a main-checkout session assignment record.
2. A focused resolver test covers a valid legacy `.worktrees/<short-id>-<slug>` worktree and verifies legacy resolution succeeds when no nested worktree exists.
3. A focused resolver test provides valid nested and legacy worktrees for one session and verifies nested layout wins; a second test provides two valid nested slug directories and asserts the deterministic ambiguity error lists lexicographically ordered candidate paths.
4. A capture-hook integration test runs with the main checkout as the process anchor, confirms capture persists only in the assigned worktree store, and asserts no main-checkout `.session/sessions/<session-uuid>/session.json` assignment record is created or updated.
5. A federated-enumerator test places distinct sessions in the main store, nested worktree store, and legacy worktree store, then verifies all six named reader surfaces return the union; a duplicate session record verifies the documented winner and one result per session ID.
6. A `SessionStoreActivity` reclaim-policy test records recent activity only in a nested or legacy worktree store and verifies reclaim is refused; an unreadable candidate store also verifies conservative refusal.
7. A worktree-tooling test verifies new creation and rename paths use the full-UUID nested directory and mirrored branch name, while the four listed legacy flat fixture paths remain discoverable without migration.
8. A guidance validation reads the three named instruction files and verifies each documents the nested layout, legacy transition rule, and worktree-local active-session marker; a lifecycle audit test proves dual-layout removal is rejected while any legacy flat worktree remains.
9. Targeted implementation validation passes with `cargo test -p session-workspace-resolver -p session-capture-hook -p session-api -p session-worktree-provision` and `cargo test -p worktree-ctl`; the pre-existing unrelated failures in `context-read` (7) and `memory-matrix`/`memory-kernel` (2) are recorded but do not gate this slice.

# Non-goals

- Migrating existing flat worktrees.
- Removing `.session/` from git tracking.
- Pushing any branch or artifact to a remote.
- Repairing the pre-existing unowned `context-read` (7) or `memory-matrix`/`memory-kernel` (2) test failures.
- Changing ticket-board ownership semantics beyond consuming federated session data where the named readers already require it.

# Traceability

- Parent spec: [2860a8db default worktree-backed session workflow](.spec/specs/2860a8db-0c4e-4e94-984a-c10a72a67ffc/spec.toml).
- Related spec: [10dee1dc board session-worktree binding](.spec/specs/10dee1dc-ab34-4e16-810b-b0c20a7677b7/spec.toml) (projection boundary; this spec replaces the assignment-discovery dependency).
- Related spec: [09f96d83 VS Code Copilot capture-hook session sync](.spec/specs/09f96d83-4795-4f19-9259-64ad0d452387/spec.toml) (worktree-local capture prerequisite).
- Related spec: [34d5f1cd session audit unified interface](.spec/specs/34d5f1cd-e6ad-41db-955c-672b22fc9bb5/spec.toml) (federated session-read neighbor).
- Ticket: [99e040a2 Remove main-checkout session registry with dual-layout worktree discovery](.ticket/tickets/99e040a2-f7fb-40cf-8714-2bc487076d72/ticket.toml).
- Epic: [db6980d1 Worktree provisioning and session-worktree lifecycle](.ticket/tickets/db6980d1-38bf-4819-8c07-b6db09229c1c/ticket.toml).
- Related tickets: [674e8e44 Session-to-worktree assignment is never persisted, and provisioning outcomes are unobservable per hook event](.worktrees/b66ea3a4-session/.ticket/tickets/674e8e44-55ee-472f-8044-5d6e473438cf/ticket.toml), [3d535b2c Add prompt-time worktree bootstrap hook](.ticket/tickets/3d535b2c-7361-4f08-bfb4-63b0b3174afc/ticket.toml), [ff83caf7 Managed session-worktree lifecycle: preserve, reuse, rename, and finish](.ticket/tickets/ff83caf7-059b-4f2e-a0fb-eaa7757096a8/ticket.toml), and [5e6cf4f8 Rewrite worktree.sh as a Rust binary and add worktree lifecycle recycling](.ticket/tickets/5e6cf4f8-120c-4674-95de-d7b79c99f5b3/ticket.toml).