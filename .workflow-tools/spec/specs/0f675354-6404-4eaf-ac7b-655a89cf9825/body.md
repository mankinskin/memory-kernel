<!-- aligned-structure:v2 -->

# Lifecycle-hook log locator records

## Target Code Location

- `not-implemented` — code_ref: [log-api owner](../../../../../log/crates/log-api/).

## Naming Conventions

Use one locator-record family with source kind equivalent to `general_tool` or `vscode_copilot_lifecycle_hook`. Optional canonical joins are `canonical_event_id` and `correlation_id` (or existing canonical equivalents); absence is explicit and no identifier is synthesized.

## Requester Decision and Task

> “Additionally we have a log-api domain which should store and manage the logs of general tools and hooks.”

Decision: this slice stores and manages locator metadata only, for general-tool and configured VS Code/Copilot lifecycle-hook artifacts. It does not store raw log content and does not extend the decision to Cline or other hook systems.

## Reading Order

1. [Parent: lifecycle-hook observability](../55c7e6ab-0bad-4fa5-ab49-c320f0dbd5a7/spec.toml).
2. [Sibling: Session lifecycle-hook events](../86e20b9e-7f6d-4a51-a51c-8dd0f2fe68cc/spec.toml) for canonical join identity.
3. This child for record/fetch/list behavior.
4. [Legacy runtime diagnostics](../aa769a27-2721-4b9d-880c-5c4e2f8136a7/spec.toml) only as broad prior context.

## Responsibility

log-api records, fetches, and lists metadata-only locators for general-tool and configured VS Code/Copilot lifecycle-hook artifacts. A locator identifies an external artifact sufficiently for its owner to find it; it is not a content cache.

## Interfaces and Dependencies

Provider: [log-api](../../../../../log/crates/log-api/) accepts locator metadata from general-tool artifact producers and the VS Code/Copilot lifecycle-hook adapter.

Dependency: when available, the provider preserves canonical lifecycle event and correlation identity supplied by the Session event producer. It does not execute a locator, infer missing correlation, or parse the external artifact.

## Behavior

Each record preserves source kind and locator metadata. Fetch and list return stored metadata and linkage state. A configured-hook locator preserves canonical event/correlation links exactly when supplied. A record remains valid and queryable when either link is unavailable.

## Boundaries and Failure Cases

Absent event/correlation identity is a supported unlinked state, not an error and not grounds to invent an ID. Raw artifact content is never accepted, persisted, returned, or parsed. Cline and other hook systems, OperationJournal/event/MCP redesign, locator execution, and external retention guarantees are excluded.

## Provider/Consumer Contract

The provider returns source-discriminated locator metadata and optional linkage state only. Consumers may use `canonical_event_id`/`correlation_id` for joins when present and must handle their absence. Consumers must not expect log bytes from record, fetch, or list.

## Dependent Expectation

General-tool and configured VS Code/Copilot hook consumers can discover locator metadata without content persistence, distinguish their sources, and join to canonical session events when those identities are available.

## Acceptance Criteria

| ID | Testable requirement | Required evidence |
|---|---|---|
| L-1 | A general-tool locator round-trips through record/fetch/list with source kind and metadata. | Record/fetch/list fixture under `val-lifecycle-hook-log-locators`. |
| L-2 | A configured-hook locator preserves supplied canonical event and correlation links exactly. | Cross-component fixture under `val-lifecycle-hook-log-locators`. |
| L-3 | A configured-hook locator with one/both links absent records, fetches, and lists with absence retained. | Unlinked-locator fixture under `val-lifecycle-hook-log-locators`. |
| L-4 | Multiple source kinds remain discriminated in supported list/filter results. | Table-driven list/filter fixture under `val-lifecycle-hook-log-locators`. |
| L-5 | Raw artifact content is not accepted, persisted, or returned. | Negative input/storage/response assertions under `val-lifecycle-hook-log-locators`. |

## Examples

A lifecycle hook writes an externally managed file. log-api stores `{ source_kind: vscode_copilot_lifecycle_hook, locator: <external metadata>, canonical_event_id: <when known> }`; fetch returns metadata, not bytes. A general-tool artifact uses the same family with `general_tool`.

## Evidence

Guard: `val-lifecycle-hook-log-locators`. Owning implementation ticket: [W5 — Store general-tool and Copilot hook log locator metadata](../../../../../.workflow-tools/ticket/tickets/141d7917-41e7-4745-985f-b300ce76d95d/ticket.toml). Planned validation after implementation: focused log-api locator record/fetch/list, source discrimination, optional-link, and raw-content exclusion tests. No validation-spec or test-api record is created by this change.

## Scope

Included: metadata-only locators for general-tool and configured VS Code/Copilot lifecycle-hook artifacts. Excluded: raw log content, Cline/other hooks, unrelated MCP or envelope redesign, locator execution, and external artifact-retention guarantees.