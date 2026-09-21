<!-- aligned-structure:v1 -->

# Summary

Interview and survey workflows are currently ad hoc and difficult to iterate collaboratively. Responses are not reliably preserved as structured records that can be revised, merged, and turned into actionable decision sheets.

## Behavior Story

Interview and survey workflows are currently ad hoc and difficult to iterate collaboratively. Responses are not reliably preserved as structured records that can be revised, merged, and turned into actionable decision sheets.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Problem

Interview and survey workflows are currently ad hoc and difficult to iterate collaboratively. Responses are not reliably preserved as structured records that can be revised, merged, and turned into actionable decision sheets.

## Goals

- Add an interview store with persistent session files and indexed metadata.
- Support single-user interviews and multi-user surveys with editable drafts and revision history.
- Produce actionable answer sheets with provenance links back to source responses.

## Required behavior

### Session and survey model
- Interview session entities persist prompt sets, participant roster, response revisions, and status metadata.
- Survey mode supports multi-user participation with per-response attribution and editable iterations.
- Session files remain human-inspectable while index/search paths remain machine-friendly.

### Explicit schema contract
- InterviewSession: `id`, `workspace_id`, `state`, `prompt_profile`, `created_by`, `opened_at`, `closed_at`, `last_activity_at`.
- InterviewResponse: `response_id`, `session_id`, `participant_ref`, `prompt_ref`, `content_ref`, `revision`, `updated_at`.
- InterviewSignal: `signal_id`, `session_id`, `response_id?`, `kind`, `severity`, `confidence`, `artifact_ref?`.
- InterviewSynthesis: `synthesis_id`, `session_id`, `summary_ref`, `open_questions_ref`, `recommended_actions_ref`, `generated_at`.

### Iterative response workflows
- Users can refine responses across rounds; each iteration retains provenance and timestamps.
- Agents can propose structured follow-up questions based on response gaps.
- Conflict-safe merges are defined for concurrent edits.

### Actionable synthesis
- The store can generate an answer-sheet artifact summarizing priorities, unresolved questions, and recommended next actions.
- Every synthesized claim links to source responses and participant context.
- Synthesis supports iterative updates as new responses arrive.

### Cross-store integration
- Synthesized output can link to tickets/specs/feedback entities through cross-store references.
- Validation and feedback records can be attached to interview sessions for quality signals.

## Operational flow

1. Initialize session with prompt profile, participant scope, and expected deliverable.
2. Capture responses with revision history and optional interviewer/agent annotations.
3. Extract structured signals and unresolved questions with confidence scores.
4. Generate answer-sheet synthesis with explicit provenance links.
5. Route actionable items into feedback/ticket flows and record disposition.

## Quality and governance gates

- Session close requires explicit unresolved-question list (can be empty, but never implicit).
- Routing requires dedupe check against already-open feedback events.
- Any unresolved artifact reference must be stored with an explicit unresolved marker and follow-up owner.
- Privileged interview mode requires actor-policy metadata and immutable audit entries.

## Major risks

- response ambiguity and over-compression in synthesis outputs
- weak provenance trails that reduce trust in generated action sheets
- large survey sets causing slow merge/synthesis cycles without indexing strategy

## Acceptance criteria

- storage contract for sessions/surveys/revisions is explicit and testable
- synthesis contract defines output schema, provenance rules, and update semantics
- integration points for ticket/spec/feedback references are defined

## Traceability

- [913fdd33 [interview-api] Interview sessions, survey orchestration, and answer synthesis](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/913fdd33-77b3-4e40-914a-db6873bf004d/ticket.toml)
- [7639449a [interview-api] Session file model and collaborative survey state](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/7639449a-22a9-4bea-9fcf-517810bc9ddf/ticket.toml)
- [0fc7b189 [interview-api] Actionable answer-sheet synthesis and iteration loop](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/0fc7b189-5c6c-4b79-a78d-5df8ad7dcf0c/ticket.toml)
- [1d6a7b5e [interview-api] Session closure governance and unresolved-question routing](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/1d6a7b5e-5f7f-4a8a-b6d8-cd3ab4c7e221/ticket.toml)

## Validation

- focused tests for session persistence, revision merge, and conflict handling
- focused tests for synthesis determinism and provenance link completeness
- integration tests for linking synthesized outputs into ticket/spec stores
- replay tests proving deterministic synthesis shape for identical response histories
