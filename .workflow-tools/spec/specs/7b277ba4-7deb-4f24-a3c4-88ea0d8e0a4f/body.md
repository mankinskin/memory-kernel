<!-- aligned-structure:v2 -->

# Summary

Define track-scoped multi-session execution semantics by extending the existing session schema with three nullable fields (track_id, anchor_ticket_id, parent_session_id) to connect sessions across handoffs without introducing a new track entity or store.

# Requirements

## R1: Track storage is schema-only
A "track" is NOT a new entity and gets NO new store. It is expressed as three new NULLABLE fields on the session schema: track_id, anchor_ticket_id, parent_session_id. The existing but unused spawned_session_id field is activated.

## R2: Anchor ticket owns goal and definition-of-done
The anchor TICKET is the single source of truth for goal, scope, and definition-of-done. The SPEC owns requirements. No goal text is stored on session records.

## R3: Track completion is ticket closure
A track is complete when its anchor ticket closes. There is no independent track lifecycle or track status field.

## R4: Handoff conformance gate is blocking and mechanical
Handoff conformance gate: BLOCKING and MECHANICAL, enforced in the handoff WRITE path (session_handoff). A handoff is REJECTED at write time unless it cites the anchor ticket id and enumerates the unmet definition-of-done items. Fail closed. No model judgement is involved.

## R5: Sub-agent sessions are isolated durable sessions
Sub-agent sessions are isolated durable sessions, stamped with parent_session_id and the inherited track_id.

## R6: Lightweight sessions use the same schema
"Lightweight" sub-agent sessions use the SAME schema. Lightness comes only from lazy artifact creation: no file or directory is written until first use. There is no second session class.

## R7: Sub-agent sessions register on board with WIP exemption for read-only agents
Sub-agent sessions register on the ticket board and count against WIP, EXCEPT read-only sub-agents which are WIP-exempt. The WIP limit is raised to 20. A weighted board budget (weight derived from files touched, effort, or duration) is an explicit FUTURE non-goal.

## R8: Traceability injection via runtime instructions
Traceability: the active session id and track id are injected via session_runtime_render_instructions plus a hook reminder, so an inline answer can be traced back to its durable session by copying the text. Presence in free prose is best-effort; there is no response post-processing.

## R9: Migration is fail-open with nullable track_id
Migration is FAIL-OPEN: track_id is nullable, pre-existing sessions simply have no track, and there is NO backfill pass. Existing sessions must load without error and report track_id = null.

## R10: Track query and rollup with MCP/CLI parity
Track query and rollup must report: session count, status, aggregate tool/cost metrics, duration, token cost, and a per-agent breakdown. Exposed over both MCP and CLI with parity.

## R11: Session store on-disk layout
Session artifacts persist under `.session/sessions/<workspace_session_id>/` (containing session records, handoffs/, and finish.json). Machine-local state (active workspace session pointers) lives under `.session/local/`. Budget-offset grants remain under `.session/grants/` (unrelated). The legacy `.session/runtime/workspaces/<id>/` tree is removed and no writer or reader targets it. Back-compat reads are explicitly NOT required — removal is fail-closed by user decision; stale legacy trees on other worktrees or machines are abandoned, not migrated. Retirement of the remaining fallback shims is tracked by ticket 7fabc77a.

## R12: Git tracking policy for session artifacts
Durable session artifacts under `.session/sessions/<workspace_session_id>/` are tracked in git. Machine-local ephemeral state under `.session/local/` and lock files matching `.session/**/*.lock` are ignored. Event payload files (`events.json` and `runs/**/events.json`) are deliberately git-ignored because, without deduplication, they produce large repository bloat (existing committed files total ~99MB across ~76 files). The repository-level `.gitignore` and `memory-api/.gitignore` MUST include rules to:

- include `.session/sessions/**` (so durable artifacts like `context.json`, `handoffs/**`, `finish.json`, `session.json`, and `transcript.json` are tracked)
- ignore `.session/local/` and its contents
- ignore `.session/**/*.lock` and `session-capture-stop.log`
- ignore `.session/sessions/**/events.json` and `.session/sessions/**/runs/**/events.json`

This policy is authoritative and gates a follow-on cleanup ticket to untrack already-committed `events.json` files.

# Non-Goals

- No track entity, no new store, no track status field.
- No parallel sessions within a track (deferred to ticket 25dd26cb).
- No weighted board budget yet.
- No response post-processing to force session ids.
- No reimplementation of the existing iteration loop or handoff package schema.

# Validation

End-to-end session lifecycle test asserting:
- check-in -> handoff write -> finish -> reload cycle
- on-disk layout and track fields persistence
- MCP and CLI round-trip through the track query surface

Commands:
```bash
cargo test -p session-api
cargo test -p session-mcp
# memory-kernel board tests
```

# Related Work

Epic: 16e4063a
Implementation tickets: ab02e15a, fd7737ec, c410ca60, 648a64a6, be1552ba, 938a7ae9, 7bb007e9, 490f1cbc, 185a00a2, 9cd9440e, b43363e1
R11 (on-disk layout): 7a4f9c3d, 4817a5cc, 41ed4585, 0a45bedb, ab02e15a, fd7737ec
Deferred: 25dd26cb
Prerequisites: 3eaceaae (closed), 0a45bedb (with 7a4f9c3d, 4817a5cc, 41ed4585)
Prior art: 1fbf2d84, 0647a212, 76e831f2, 5755b694, d3af78d7
Draft docs: tmp/spec-iteration-loop.md, tmp/spec-handoff-package-schema.md
Source proposal: transcripts/27-07-2026_session-track-management/input.clean.md

Code anchors:
- memory-api/crates/session-api/src/model.rs
- memory-api/crates/session-api/src/store.rs
- memory-api/tools/mcp/session-mcp/src/server.rs
- memory-api/tools/cli/session-cli/src/lib.rs
- memory-kernel/src/storage/board.rs