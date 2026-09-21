This section records the round-2 settlements (Q1–Q5 + the transport-parity investigation). They supersede the prior "Open Decisions" where they overlap.

## Q1 — gate on feedback-api CORE, not the full program (resolves prior Open Decision 1)
A minimal **core curation surface** ticket `c7542933` was created: URN-keyed usage counting + entity ratings (specs/rules now, ticket extension point), query-by-frequency / low-rated, with NO dependency on the heavyweight feedback-api slices (at-scale search, SLOs, governance, retention). Edges rewired:
- `effba966` (epic) and `412964a3` (runtime) now `depends_on c7542933` (core), not `b1e9e744` (full program).
- `b1e9e744` (full program) `depends_on c7542933` (core is a child milestone of the program).
Spec `71b81a55` describes this core contract.

## Q2 — placement principle (lowest-owning store)
Entities live in the **lowest-level store containing all the code they concern**: memory-api-domain → memory-api store; viewer-api-domain → viewer-api store; only genuinely cross-workspace concerns stay in a parent (context-engine). We do NOT consolidate everything. Cleanup ticket `7599ed31` was refined with an audited move/stay list:
- MOVE to memory-api: session-bootstrap cluster, feedback-api program (incl. core), workspace-resolution (`ef0ebf38`/`07836f41`/`59d96577`), bidirectional (`185419e0`), transport parity (`39239e48`).
- STAY in context-engine (cross-workspace): `671d4e47` multi-store tracker, `82d6ada4`/`6bd67a7a` URN refs, `8a90a63c` multi-store program.

## Q3 — bidirectional transitions (resolves prior Open Decision 3)
Tracked by `185419e0` "[ticket-api] Allow bidirectional ticket state transitions by default" (in-review; code committed in memory-api `fdadb7d` + reverse-terminal tests `5a672e6`). Convergence warnings ("epic in-implementation while prerequisites are new") are to be **ignored**; once bidirectional lands, states can be walked back if needed.

## Q4 — moving active work is OK
Relocating `in-implementation` tickets via the move tooling is acceptable; the implementing agent owns the moves and journaled move + re-linking preserves in-flight state.

## Q5 — update_ticket bug fixed & closed
`7f4aaa05` fixed in memory-api commit `59f0860` (restored `transition_states` handling; state preserved on field/description patch without `to_state`). Validated 2026-06-27: `cargo test -p ticket-api bug_7f4aaa05` (3 passed) + full suite (67 passed). Ticket is **done**.

## Transport-parity investigation (new)
Reproduced: from the parent/default workspace, ticket search does NOT discover tickets in the nested `memory-api/.ticket` store (only finds parent-store copies); ticket-mcp `list_workspaces` exposes only `default`. Root cause: workspace resolution is transport-specific and not recursive. Existing planning: `ef0ebf38` (in-implementation, CLI-only shared resolver) + children `07836f41` (ticket-cli get/search/list) and `59d96577` (spec-cli + spec-mcp). Gap filled by new ticket `39239e48` "[memory-api][ticket-mcp][ticket-http] Transport-layer workspace-resolution parity + pure-transport cleanup" (`depends_on ef0ebf38`): hoist resolution into memory-api, keep transports pure, make mcp/http nested-root-aware, audit ticket-cli for misplaced logic.