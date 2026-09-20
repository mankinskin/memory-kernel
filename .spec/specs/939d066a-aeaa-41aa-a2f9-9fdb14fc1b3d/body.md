<!-- aligned-structure:v2 -->

## Goal

Persistent stores and generated output directories must never appear as a side effect of a tool merely starting up in a workspace. Every MCP server and viewer covered by this contract must leave a fresh, storeless workspace filesystem-identical before and after startup, unless the caller explicitly initializes or configures a store/log sink.

## Dependent Expectation

If this specification is implemented, callers can start an installed MCP server or viewer in a fresh storeless workspace without creating a persistent store, log directory, or other filesystem artifact; store-required operations fail without mutation until a caller explicitly initializes or configures a write.

## Authoritative Behavior (approved)

1. `.ticket` and equivalent persistent stores (`.spec`, `.session`, `.feedback`, `.test`, or any store-backed tool) require explicit initialization or configuration; mere process startup, read-only queries, or no-op tool calls against a storeless workspace must never create the store directory.
2. Generated output directories, including `test-logs/` and `target/test-logs/`, are created lazily, only on the first actual configured write. They must never be created as a side effect of process startup, tracing/logging initialization, or a read-only call.
3. Absent an explicit filesystem log destination (no configured log file/dir), filesystem logging remains disabled — no log file or directory is created as a side effect of enabling tracing/logging infrastructure.
4. Any operation that requires a store to exist, invoked against a workspace where that store is missing and not configured, must fail with a clear, non-mutating error. The tool must not silently create the missing store to satisfy the failed operation.
5. Explicit initialization (`init`) and explicit configured first writes are unaffected — they must still create their intended artifact exactly as today.

## Scope

Covers startup-time behavior of every installed MCP server and viewer binary, discovered by direct investigation rather than assumed, including at minimum:

- `memory-api/tools/mcp/ticket-mcp`
- `memory-api/tools/mcp/session-mcp` (and `session-cli`)
- `context-stack/tools/mcp/context-mcp`
- `memory-viewers/ticket-viewer`
- `memory-viewers/log-viewer`
- `memory-viewers/doc-viewer`
- `memory-viewers/spec-viewer`
- any `spec-mcp`, `test-mcp`, `audit-mcp`, `feedback-mcp`, `fs-mcp` binaries present in `memory-api/tools/mcp/` or equivalent locations

The actual installed tool matrix must be enumerated during implementation (do not hardcode a fixed list without verifying it against the repository at implementation time).

## Non-Goals

- Does not ban or restrict intentional explicit initialization (`init`) or explicit configured writes — those must keep creating their artifacts.
- Does not change the contents of a log file/store once filesystem logging or store creation is legitimately configured/enabled.
- Does not fold in or restate pre-dispatch agent-gate requirements from orchestration instructions; that is a separate concern.
- Does not resolve the session-anchored workspace-resolution/store-divergence defect tracked in ticket fa2ba34b-59ec-4321-a4ce-a3c3a9295ea3 (cwd-vs-server-cwd store resolution is a related but distinct concern from artifact-creation timing).

## Acceptance Criteria

1. A reusable empty-workspace fixture exists (e.g., a temp-dir helper) that produces a fresh, storeless workspace directory with no `.ticket`/`.spec`/`.session`/`.feedback`/`.test`/log-output artifacts, usable by both focused package tests and a repository-level fixture command.
2. A parameterized startup test matrix exists covering every relevant installed MCP server/viewer entry point identified during investigation (not just ticket-mcp and log-viewer).
3. For each tool in the matrix, starting the tool (or running its read-only/no-op startup path) against the empty-workspace fixture with no configured store/log sink produces a zero filesystem delta: before/after snapshots of the fixture directory are identical, verified by direct comparison, not by absence-of-error alone.
4. For any tool whose normal operation requires a store to exist, invoking an operation against the empty-workspace fixture (no store configured/initialized) produces a clear, non-mutating error — the tool must not silently create the missing store as a side effect of the failed operation.
5. A positive-path test exists proving that explicit initialization or an explicit configured write still creates the intended artifact (store directory, log file/dir) when the caller actually requests it — confirming the fix does not break legitimate lazy-creation behavior.
6. Regression coverage exists that pinpoints the exact code path/creator responsible for the previously observed `test-logs/`/`target/test-logs/` creation-on-startup behavior (not just a passing/failing assertion, but a test that would fail again at the specific creator if reintroduced).
7. Focused package-level tests pass for each affected crate/tool.
8. A repository-level fixture/validation command exists (e.g., a script or test target) that can run the full startup matrix and report the delta-per-tool result in one invocation.

## Validation Plan

- Design validation before implementation, per the fixture and assertion shape in Acceptance Criteria 1-2.
- Focused, affected-package test suites per tool discovered in the matrix.
- Repository-level fixture/matrix command (Acceptance Criterion 8) as the single source of truth for the full startup contract.
- Regression test targeting the exact prior `test-logs/`/`target/test-logs/` creator (Acceptance Criterion 6).

## Guards

- `vt-storeless-startup-matrix` is the required test-api `ValidationSpec` for this contract. The validation spec is not yet present in `.test/default/specs`; implementation must create it before recording startup-matrix evidence.

## Positions

- `not-implemented` — `memory-api/tools/mcp/ticket-mcp/src/main.rs` is a handoff-identified startup surface; the startup matrix must determine whether the path creates an artifact before claiming a specific creator.
- `not-implemented` — `memory-viewers/log-viewer/src/config.rs` is a handoff-identified logging startup surface; the startup matrix must determine whether the path creates an artifact before claiming a specific creator.
- `not-implemented` — `context-stack/context-trace/src/logging/tracing_utils/test_tracing.rs` and `context-stack/context-trace/src/logging/tracing_utils/config/loader.rs` are handoff-identified tracing startup surfaces; the startup matrix must determine whether either path creates an artifact before claiming a specific creator.

## Governing-rule Requirement

No PolicyRule currently introduces this specification in-session. A governing rule must be registered before the specification can be presented as implemented; until then, this contract remains a planned requirement with an absent validation guard.

## Related

- Ticket 52724aed-7215-471b-b2d8-7fb425f5ed61 — "Startup artifact pollution: MCP servers/viewers must not create stores or log dirs on mere startup in storeless workspaces" (primary tracking ticket for this spec).
- Ticket fa2ba34b-59ec-4321-a4ce-a3c3a9295ea3 — "Session-anchored MCP workspace resolution: require session_id and resolve every proxied call to the session's active worktree" (linked, not a hard dependency; addresses a related but distinct cwd-vs-server-cwd store-resolution concern).
