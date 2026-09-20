## Goal

Agents can author dense, dynamic session workflow graphs with minimal round trips. Rejections name a copy-ready legal alternative, custom labels use task nodes plus category, and contextual entity references use a non-gating anchor_urn.

## Requirements

- Keep behavioral node kinds closed to ticket, validation, spec, and task; expose category as the free-text label escape hatch.
- Keep ticket_urn and spec_urn as finish-gating fields for their matching node kinds. Add anchor_urn for a ticket or spec reference on any node kind; it never affects finish or handoff gating.
- Add atomic batch node and edge creation in session-api, session-mcp, and session-cli. Validation errors identify the offending array index; duplicate node IDs retain existing no-op behavior.
- All workflow enum and URN-gating errors include a compact legal example or alternative.
- Document final authoring patterns in path-scoped agent guidance.

## Validation

Validation `val-session-workflow-flexibility` passed in execution
`exec-session-workflow-flexibility-20260725`: `cargo test -p session-api -p
session-mcp -p session-cli` passed 146 tests across 12 suites. Existing
ticket, spec, and validation finish-gating tests remained green.

The hand-owned path-scoped authoring guidance passes frontmatter diagnostics
and is linked from session bootstrap for discovery. It documents the final
field names, rejection fixes, and atomic batch patterns.

## Implementation

- `session-api` persists non-gating `anchor_urn` values and atomically creates
	node and edge batches with indexed validation errors.
- `session-mcp` exposes `session_workflow_add_nodes` and
	`session_workflow_add_edges`, documents `category`, and returns copy-ready
	alternatives for rejected enum and URN values.
- `session-cli` provides nested and compatibility-flat batch commands using
	structured JSON arrays.
- `.agents/instructions/session/session-workflow.instructions.md` documents the final
	node model, entity references, batch behavior, rejection fixes, and an
	anchored review-criteria example.