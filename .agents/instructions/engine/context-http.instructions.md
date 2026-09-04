---
description: "Use when editing context-http, the HTTP and optional GraphQL adapter for context-api. Covers RPC dispatch, trace capture, state access, and HTTP error mapping."
---


## Architecture

- `context-http` is a thin transport adapter over `context-api`.
- Keep business/domain behavior in `context-api`; keep adapter behavior in `context-http`.

## Primary Interface

- `POST /api/execute` is the universal RPC endpoint and should remain the primary API surface.
- Command payloads must preserve `Command` JSON shape with `"command"` discriminant.
- Optional `"trace": true` behavior should keep parity with current traced execution semantics.
- REST routes under `/api/workspaces/...` are convenience endpoints; maintain consistency with command semantics.

## State and Concurrency

- Use `AppState` with `Arc<Mutex<WorkspaceManager>>` patterns for shared mutable access.
- Do not bypass manager locking patterns for command execution paths.
- Keep capture-config resolution (`capture_config_for`) aligned with workspace log-directory behavior.

## GraphQL

- GraphQL is feature-gated (`graphql` feature).
- Keep GraphQL endpoint behavior isolated from core RPC logic.
- Avoid introducing multiple GraphQL libraries or duplicate schema paths.

## Error Mapping

- Preserve explicit mapping from `ApiError` categories to HTTP status codes.
- Keep response body shape stable for handler errors.
- Add new error kinds only with explicit HTTP status mapping.

## Runtime and Configuration

- Respect env-driven configuration (`PORT`, `HOST`, `LOG_LEVEL`, `LOG_FILE`, `CONTEXT_ENGINE_DIR`, optional `STATIC_DIR`).
- Prefer viewer-api tracing and router utilities over ad hoc setup.

## Validation

- Run focused adapter tests after route or error-mapping changes.
- Verify both `/api/execute` and relevant REST endpoints for behavior parity.
- If GraphQL code changes, validate feature-gated build/test behavior.
