# Goal

Bridge the existing domain-neutral move kernel journal to the generalized operation-journal model without breaking current move APIs.

## Scope

- Map `MovePlan`, `MoveJournal`, `MoveExecutionPhase`, `MoveOutcome`, path rewrites, manual follow-ups, locks, and board migration data to the generic journal envelope.
- Preserve CLI/MCP/HTTP response compatibility for ticket/spec move surfaces.
- Add conversion or metadata registration into `log-api`/journal index.
- Keep resume and rollback tests passing.

## Acceptance criteria

- Existing move journals remain readable.
- New journal metadata can reference move journals by id and operation kind.
- Resume/rollback validation still passes for ticket moves.