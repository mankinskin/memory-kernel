<!-- aligned-structure:v2 -->

# VS Code Copilot lifecycle-hook invocation events

## Target Code Location

- `not-implemented` — code_ref: [Session API owner](../../../../../session/crates/session-api/).
- `not-implemented` — code_ref: [session capture-hook entrypoint](../../../../../session/src/bin/session-capture-hook/main.rs).

## Naming Conventions

Use a source/kind equivalent to `vscode_copilot_lifecycle_hook`; consistently name `invocation_id`, `lifecycle_trigger`, `configured_command_or_path`, and `outcome`. Reuse canonical Session API session/turn/tool-call correlation vocabulary rather than defining an MCP-call event shape.

## Requester Decision and Task

> “Also monitoring of hook calls in a session should be implemented as a type of tool calling.”

Decision: apply this only to configured VS Code/Copilot lifecycle hooks. Each observed invocation uses the canonical Session API event path; raw logs and unrelated hook systems are out of scope.

## Reading Order

1. [Parent: lifecycle-hook observability](../55c7e6ab-0bad-4fa5-ab49-c320f0dbd5a7/spec.toml).
2. This child for invocation-to-event semantics.
3. [Sibling: log locator records](../0f675354-6404-4eaf-ac7b-655a89cf9825/spec.toml) for optional artifact linkage.
4. [Legacy capture-hook sync](../09f96d83-4795-4f19-9259-64ad0d452387/spec.toml) only as prior capture context.
5. [Canonical MCP-router call events](../../../../../../.workflow-tools/spec/specs/0c129254-0e31-4088-a61c-11f52d82a0e7/spec.toml) only to distinguish that separate call source.

## Responsibility

For each configured VS Code/Copilot lifecycle-hook invocation, including unsuccessful completion, persist exactly one canonical, tool-like Session API event through the normal event path. A retry is a new invocation and receives a new identity.

## Interfaces and Dependencies

Provider: the configured-hook adapter at [session capture-hook entrypoint](../../../../../session/src/bin/session-capture-hook/main.rs) maps an observed execution into [Session API](../../../../../session/crates/session-api/).

Consumer: log-api may use the emitted canonical event identity and correlation identity when available. This child neither creates a parallel event stream nor defines MCP-router call semantics.

## Behavior

The event carries a stable per-execution invocation identity; lifecycle trigger; configured command and/or path; observed completion result including exit outcome when available; and every available session, turn, and tool-call correlation. Missing correlation remains absent. Failure or nonzero exit changes outcome, not event existence.

## Boundaries and Failure Cases

A failed/nonzero execution still emits once. Retry means a new identity, not replay. The adapter must not fabricate missing session/turn/tool-call correlation. It must not persist emitted log text. Cline, other hook systems, hook installation/relocation, scheduling, command execution semantics, unrelated MCP calls, and broad event-schema redesign are excluded.

## Provider/Consumer Contract

The provider offers exactly-once canonical invocation representation by `invocation_id`. Consumers may treat the event as the canonical join target for a lifecycle-hook locator only when event/correlation identity is present; consumers must tolerate its absence and must not expect raw log content.

## Dependent Expectation

Session consumers can distinguish a configured lifecycle-hook execution from transcript capture and MCP-router calls, correlate it to a session/turn/tool-call when supplied, and represent failure without losing the invocation.

## Acceptance Criteria

| ID | Testable requirement | Required evidence |
|---|---|---|
| S-1 | One configured lifecycle-hook invocation persists exactly one canonical tool-like event for its `invocation_id`. | Count-by-invocation fixture under `val-session-lifecycle-hook-events`. |
| S-2 | Distinct triggers and command/path configurations round-trip independently. | Table-driven adapter-to-Session-API fixture under `val-session-lifecycle-hook-events`. |
| S-3 | Success, nonzero, and otherwise available completion results round-trip. | Outcome matrix under `val-session-lifecycle-hook-events`. |
| S-4 | Available session/turn/tool-call correlation round-trips and absent correlation remains absent. | Serialization/read-query fixture under `val-session-lifecycle-hook-events`. |
| S-5 | Retry after failure creates a distinct identity and one event per execution. | Retry fixture under `val-session-lifecycle-hook-events`. |
| S-6 | Event payload/storage omits raw hook-log content. | Negative serialization/persistence assertion under `val-session-lifecycle-hook-events`. |

## Examples

A configured `PreToolUse` command at `/repo/.github/hooks/audit` exits `1`. One event records a new invocation identity, trigger, configured path, nonzero outcome, and available correlation; it contains no emitted log text.

## Evidence

Guard: `val-session-lifecycle-hook-events`. Owning implementation ticket: [W4 — Record configured Copilot lifecycle-hook invocations as canonical session events](../../../../../.workflow-tools/ticket/tickets/16ea5791-a3f1-4c3e-aa1f-eec6e3fd5c84/ticket.toml). Planned validation after implementation: focused Session API lifecycle-hook ingestion/persistence, outcome, correlation, retry, and negative-content tests. No validation-spec or test-api record is created by this change.

## Scope

Included: configured VS Code/Copilot lifecycle-hook invocation events only. Excluded: Cline and other hook systems, raw-log persistence, hook installation/relocation, unrelated MCP semantics, and broad event-schema redesign.