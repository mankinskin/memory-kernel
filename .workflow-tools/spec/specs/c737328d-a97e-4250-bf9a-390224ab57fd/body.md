## Summary

Promote the session handoff record to be the literal provenance edge of the cross-session
work graph, and introduce a first-class `Track` entity for session-side coordination. This
closes the gap where every existing cross-session link (`parent_session_id`,
`spawned_session_id`, `predecessor_handoff`) is a singular 1:1 pointer, making N→1 merge and
1→N split unrepresentable.

## Motivation ("why")

[transcripts/29-07-2026_session-merge-pickup-workflow/input.clean.md](../../../transcripts/29-07-2026_session-merge-pickup-workflow/input.clean.md)
describes an orchestrator that starts multiple tracked subagent sessions, merges converging
work onto a shared track, and later splits the shared result back out to the originating
tracks. Today `track_id` is a free-form tag with no entity behind it, and there is no way to
express "two sessions handed work into one" or "one session handed work out to two" without
inventing ad hoc arrays. [tmp/interview-session-merge-pickup.md](../../../tmp/interview-session-merge-pickup.md)
records the settled architecture (decisions 1-15) that this spec makes normative.

## Dependent expectation

If this spec is implemented, dependents can rely on: a handoff record that is a binary
provenance edge with a target bound at pickup time; a queryable backlog of unclaimed
handoffs; full bidirectional provenance-graph reconstruction from `SessionRecord` alone with
no external index; canonical `handoff`/`fan_out`/`merge`/`pickup` verbs across CLI, MCP, and
record type names; a first-class `Track` entity under `.session/tracks/<id>/` that reads work
state through `anchor_ticket_id` rather than duplicating it; a single `depends_on` track edge
kind with derived (never stored) `overlaps`; and an advisory, non-mutating convergence
detector.

## Scope

- Handoff record becomes the provenance edge (source + target), with target bound at pickup.
- `SessionRecord` gains `emitted_handoff_ids` and `picked_up_handoff_ids`.
- Canonical vocabulary: `handoff` (1→1), `fan_out` (1→N), `merge` (N→1), `pickup` (consume
  any handoff) — CLI verb names, MCP tool names, and record type names.
- First-class `Track` entity: id, title, `anchor_ticket_id`, member session ids,
  track-to-track `depends_on` edges, rollup cache.
- Explicit `merge` operation plus advisory (non-mutating) convergence detection.
- Removal of `SessionRecord.parent_session_id`, `SessionRecord.spawned_session_id`,
  `SessionHandoffPackage.predecessor_handoff`, with best-effort backfill migration.

## Non-goals

- Changing `SessionRunLineage.predecessor_run_id` or intra-session resume behavior. That
  field is explicitly kept (decision 13); `run_lineage_init_resume_creates_distinct_linked_run`
  must keep passing unmodified.
- Making the ticket mirror of a handoff (`mirror_handoff_to_tickets`) authoritative. Recorded
  as an open question below.
- Choosing the write-side consistency mechanism for bidirectional track edges (transactional
  write vs. repair pass). The requirement is stated; the mechanism is an open question below.
- Building the long-term "root orchestrator runs all day" vision described in the transcript's
  background section — this spec covers only the merge/pickup/track primitives.
- Any exhaustive migration of historical records; backfill is best-effort only.

## Requirements

### R1 — Handoff is a binary provenance edge

A handoff record has exactly one source session and at most one target session; there are no
arrays of sources or targets inside a single handoff record. `fan_out` is one session
emitting N separate handoff records (N edges, each single-target). `merge` is one session
picking up N separate handoff records (N edges, each single-source). The provenance graph is
a plain DAG of binary edges; every traversal step is uniform regardless of fan-in or fan-out
degree. No separate `MergeRecord` entity and no separate edge store are introduced — the
handoff record remains the sole edge primitive.

### R2 — Target bound at pickup time

A handoff is created target-less: it has a source session and no target, and is a claimable,
initially open edge. The `pickup` operation writes `target_session_id` onto the handoff record
and appends the handoff id to the picking-up session's `picked_up_handoff_ids`. A handoff with
no successor is a legitimate terminal state, not an error. Unclaimed handoffs (target-less)
form a queryable backlog — an operation must exist to list open handoffs, optionally scoped by
track or by source session.

### R3 — Bidirectional record-only graph reconstruction

`SessionRecord` gains two fields: `emitted_handoff_ids: Vec<String>` (handoffs this session
created as source) and `picked_up_handoff_ids: Vec<String>` (handoffs this session claimed as
target). The full provenance graph — forward (source → target) and reverse (target → source)
— must be reconstructible by reading persisted `SessionRecord` and handoff records alone, with
no auxiliary index, cache, or side-channel required for correctness.

### R4 — Canonical vocabulary

The following terms are canonical and must be used consistently as CLI subcommand/verb names,
MCP tool names, and Rust record/operation type names:

| Term | Cardinality | Meaning |
|---|---|---|
| `handoff` | 1 → 1 (until claimed) | Emit one target-less claimable edge. |
| `fan_out` | 1 → N | One session emits N handoff records. |
| `merge` | N → 1 | One session picks up N handoff records. |
| `pickup` | consume any handoff | Claim one open handoff, binding its target. |

No other terminology (e.g. "split", "join", "spawn") is introduced for these operations in
new surfaces; existing uses of `parent_session_id`/`spawned_session_id` naming are retired per
R7.

### R5 — First-class Track entity

A `Track` is a first-class entity persisted under `.session/tracks/<id>/`, session-side only.
Its manifest holds: `id`, `title`, `anchor_ticket_id` (the epic or tracker ticket the track is
rooted in), member session ids, track-to-track edges, and a rollup cache. All work status
(ticket state, progress) stays in ticket-api and is read live through `anchor_ticket_id` —
the Track manifest must never persist a duplicated status/state field.

### R6 — Track edges: single kind, derived overlap, bidirectional write

Tracks have exactly one persisted edge kind: `depends_on`. `contains` is expressed as
`depends_on` — a parent track depends on its child tracks; no separate `contains` kind
exists. `overlaps` is derived, never stored: two tracks overlap when they both depend on the
same child track, computed at query time. `depends_on` edges are written on both the
depending track and the depended-on track (so "which larger tracks depend on me" is an O(1)
lookup on the child's own manifest, never a full scan). The write path must guarantee both
sides are updated together; the exact mechanism (transactional write vs. repair/reconciliation
pass) is an open question left to implementation, not decided by this spec.

### R7 — Legacy singular cross-session fields removed

`SessionRecord.parent_session_id`, `SessionRecord.spawned_session_id`, and
`SessionHandoffPackage.predecessor_handoff` are removed. The handoff edge graph (R1-R3)
becomes the only cross-session lineage mechanism. `SessionRunLineage.predecessor_run_id` is
explicitly out of scope for removal (see Non-goals) — it is intra-session resume lineage, not
cross-session provenance, and continues to back `session_runtime_resume`.

### R8 — Explicit merge with advisory, non-mutating convergence detection

`merge` is always an explicit, user- or agent-invoked operation; the system never merges
sessions automatically. A separate advisory convergence detector flags merge *candidates*
using all three signals together: (a) shared ticket or spec ids in `SessionLinks`, (b)
overlapping owned files on the board, and (c) same `track_id`. The detector is read-only: it
must never mutate session, handoff, or track state, and its output is a suggestion surfaced
to the caller, not an automatic action.

### R9 — Backfill migration, best-effort only

Migration backfills `emitted_handoff_ids`/`picked_up_handoff_ids` (or equivalent) only where
the removed singular fields (`parent_session_id`, `spawned_session_id`, `predecessor_handoff`)
are actually populated in existing records. Records where these fields are already empty are
skipped silently — no exhaustive rewrite of all historical session or handoff records is
required or expected.

## Acceptance criteria

- AC1 (unit) — Given a set of persisted `SessionRecord` and handoff records covering a
  fan_out (1 source → N target-less handoffs, some claimed, some still open) and a merge
  (1 session with N entries in `picked_up_handoff_ids`), a graph-reconstruction routine built
  only from those persisted records recovers the full forward and reverse provenance graph,
  correctly distinguishing claimed vs. open (target-less) edges. No auxiliary store is
  consulted.
- AC2 (unit) — Listing unclaimed handoffs returns exactly the target-less handoff records,
  scoped correctly by track and by source session.
- AC3 (unit) — `run_lineage_init_resume_creates_distinct_linked_run` continues to pass
  unmodified after `SessionRunLineage.predecessor_run_id` is retained and the three legacy
  cross-session fields are removed.
- AC4 (unit) — `Track` `depends_on` edges written from a parent to a child are readable in
  reverse from the child's own manifest without a scan; a derived `overlaps` query over two
  tracks depending on the same child returns that overlap without any stored `overlaps` edge.
- AC5 (unit) — The advisory convergence detector, run against fixtures with each of the three
  signals present individually and in combination, returns candidate flags only and performs
  no writes to any session, handoff, or track record.
- AC6 (integration, scripted orchestrator round-trip fixture) — A single scripted fixture
  drives the full worked example from
  [transcripts/29-07-2026_session-merge-pickup-workflow/input.clean.md](../../../transcripts/29-07-2026_session-merge-pickup-workflow/input.clean.md):
  1. An orchestrator session `fan_out`s to start 2 track sessions.
  2. Each of the 2 track sessions independently emits one `handoff`.
  3. A shared-track session `merge`s (picks up) both handoffs into itself.
  4. The shared-track session finishes and `fan_out`s back into 2 new handoffs (one per
     originating track).
  5. Each originating track's session `pickup`s its respective handoff and finishes.
  6. The orchestrator `merge`s (picks up) both finishing handoffs back into itself.

  The fixture asserts: every provenance edge in the resulting graph reconstructs correctly
  from persisted records alone (per AC1); the orchestrator, the 2 original track sessions,
  and the shared-track session all appear exactly once each in the reconstructed graph; no
  handoff is claimed twice; and no unit of work (ticket/spec id referenced via `SessionLinks`)
  is lost or duplicated across the merge/fan_out/merge cycle.

## Migration

Backfill runs once against existing session and handoff stores. Per R9, a record is migrated
only if it has a populated `parent_session_id`, `spawned_session_id`, or `predecessor_handoff`
value; the migration derives the corresponding `emitted_handoff_ids`/`picked_up_handoff_ids`
entries (creating a synthetic bound-target handoff edge where no handoff record already
covers the link) and then the legacy field is dropped. Records with none of these fields
populated are left untouched and are not visited beyond a version bump.

## Related

- [c677182e Durable session workflow graph and handoff continuity](../c677182e-90da-4ac3-8b94-9e2e97c825cf/spec.toml) — parent spec; this spec extends its handoff/session model with the binary provenance edge and Track entity.
- [5e52039d Handoff Package Schema](../5e52039d-aabc-434d-bdf3-eca63e312476/spec.toml) — updated in place (see below) because `predecessor_handoff` is superseded by the binary edge model in R1/R2/R7.
- [b71658f1 Iteration Loop Workflow](../b71658f1-8de2-444a-9be1-64b1d8ecce70/spec.toml) — the merge/pickup vocabulary in R4 is consumed by the Iteration Agent's re-packaging and next-handoff authoring steps; no change to that spec's phase model is required.

## Related Tickets

- [d28afbc0 [session-api] Session merge and pickup: handoff-edge provenance graph and first-class tracks](.ticket/tickets/d28afbc0-9d16-4494-8ca5-4154f3ace9be/ticket.toml) — owning epic.
- [938a7ae9 Track query and rollup surface](.ticket/tickets/938a7ae9-570e-40c4-91f5-d32d2fae0b4f/ticket.toml) — to be reparented under the epic; delivers the R5 query/rollup surface.
- [185a00a2 MCP tool surface for track query and rollup](.ticket/tickets/185a00a2-a849-48b1-b4ce-08cc8fd3552d/ticket.toml) — to be reparented under the epic; delivers MCP access to R5.
- [f35f4dd9 Embed a persistent step graph in the handoff package](.ticket/tickets/f35f4dd9-1a05-47ee-b334-809bb34e63a7/ticket.toml) — to be reparented under the epic; its step-graph snapshot work composes with the R1/R2 binary edge model.

## Open questions

- Whether the best-effort ticket mirror of a handoff (`mirror_handoff_to_tickets`) becomes
  authoritative now that handoffs are the provenance edges. Out of scope for this spec.
  Owner: user.
- The exact write-side consistency mechanism for bidirectional `depends_on` track edges
  (transactional write vs. repair/reconciliation pass) — R6 states the requirement only; the
  mechanism is left to the implementing ticket(s).
