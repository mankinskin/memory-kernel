# Goal
Define a stable contract for session-level auditing from persisted session artifacts and expose it through the unified audit CLI surface.

## Problem
Session debugging currently requires ad-hoc log/file analysis and lacks a first-class audit report for a single session context. Persisted session artifacts also need an explicit schema version marker so format evolution remains auditable.

## Scope
- Add a `session_audit` capability in session-api that audits one persisted session.
- Support deterministic session selection by explicit session id and by latest session resolution.
- Add session schema version tagging in persisted artifacts and include it in audit output.
- Expose this through audit-cli's unified audit interface.

## Non-goals
- Full migration framework for historical sessions across arbitrary schema versions.
- Replacing repository-wide audit reporting; this extends it with session-specific review.

## Acceptance criteria
1. Session-api exposes a `session_audit` operation that accepts either explicit session id or latest-session selector and returns a structured report.
2. Persisted session records include a schema version field managed by session-api.
3. Session load/audit behavior handles schema-version mismatch explicitly (compatible read path or clear diagnostic outcome).
4. Audit-cli unified interface can run `session_audit` for latest session and specific session id.
5. Text and JSON output include audited session id, schema version, and core findings summary.
6. Focused tests cover selector resolution, schema version behavior, and audit output shape.

## Traceability
- Tracker: C:/Users/linus/git/graph_app/workflow-tools/.workflow-tools/ticket/tickets/2d8d0487-1680-424c-816d-01925e187e62/ticket.toml
- Spec contract ticket: C:/Users/linus/git/graph_app/workflow-tools/.workflow-tools/ticket/tickets/5e99cc3e-5b9e-4ca1-a54c-cbdf82444b50/ticket.toml
- Session-api implementation: C:/Users/linus/git/graph_app/workflow-tools/.workflow-tools/ticket/tickets/627d4152-36a7-4b24-9a9c-5f047abcac60/ticket.toml
- audit-cli integration: C:/Users/linus/git/graph_app/workflow-tools/.workflow-tools/ticket/tickets/2663a981-d279-45dc-abc0-42270491dca6/ticket.toml
- Validation/evidence: C:/Users/linus/git/graph_app/workflow-tools/.workflow-tools/ticket/tickets/f1161fae-b1fc-4cc1-baae-18c0eb7e7868/ticket.toml
