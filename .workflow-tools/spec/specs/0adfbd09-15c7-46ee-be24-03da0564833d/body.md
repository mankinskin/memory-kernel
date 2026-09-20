<!-- aligned-structure:v1 -->

# Summary

Define the required performance properties for ticket-store scan reconciliation and move execution now that the move-domain E2E exposes major indexing hot-path costs.

## Behavior Story

Define the required performance properties for ticket-store scan reconciliation and move execution now that the move-domain E2E exposes major indexing hot-path costs.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Goal

Define the required performance properties for ticket-store scan reconciliation and move execution now that the move-domain E2E exposes major indexing hot-path costs.

## Problem

Current ticket-store performance is dominated by full-store rescan behavior and per-entity full-text indexing writes.

Observed evidence from the existing move/health perf E2E:

- `scan(false)` on 1 changed ticket costs about the same as a full scan on the perf fixture.
- `integration.search_upsert_ms` is the dominant scan sub-phase.
- move execution spends most of its time in `scan_source_ms` and `scan_target_ms`, not in entity rename or path-reference rewrite.

## Scope

- `ticket-api` scan reconciliation logic
- `memory-api` search-index bulk write behavior used by ticket scans
- ticket move-domain post-move index reconciliation
- scan timing metric correctness for rebuild-probe instrumentation
- remaining follow-up planning for workflow-facts recompute, residual SQLite/index batching, and targeted reconcile modes

## Non-goals

- public API changes for ticket CLI, HTTP, or MCP
- schema redesign for SQLite or Tantivy documents
- broad performance work outside the ticket scan/move/indexing path

## Acceptance criteria

1. A non-reindex scan avoids reprocessing unchanged ticket entries when the metadata/search index is already healthy.
2. Bulk scan integration does not pay one Tantivy commit and merge wait per ticket document.
3. Ticket move execution avoids forcing full source and target store reindex scans when only the moved ticket and directly affected facts need reconciliation.
4. The existing perf E2E remains green and continues to emit comparable phase summaries, with materially reduced scan-dominated phases on the same fixture.
5. `search_rebuild_check_ms` measures only the rebuild probe work and no longer aliases total scan duration.
6. Remaining follow-up work is tracked by concrete child tickets for workflow-facts recompute, residual SQLite/index batching, and targeted reconcile modes.

## Validation evidence

Primary evidence for this slice:

- `cargo test -p ticket-api --test e2e_perf_move_health -- --nocapture`
- targeted `ticket-api` tests for scan reconciliation and move post-scan behavior
- if needed, focused Criterion reruns in `memory-api/crates/ticket-api/benches/move_health.rs`

Implemented evidence:

- batched Tantivy search upserts during scan integration
- unchanged-entry fast path on `scan(false)` using indexed metadata equality plus manifest/description mtime checks
- ticket move-domain rescans changed from `scan(true)` to `scan(false)`
- `search_rebuild_check_ms` scoped to the rebuild probe instead of full scan duration

Representative measured deltas on the existing perf fixture:

- `move_perf execute_ms`: `17746` -> `5209`
- `move_perf scan_target_ms`: `12121` -> `2790`
- `move_seq_perf first_ms`: `36952` -> `11392`
- `move_seq_perf second_ms`: `40013` -> `10075`
- `scan_true integration.search_upsert_ms`: `12340` -> `187`
- `scan_false_1 integration.index_upsert_ms`: `2192` -> `33`
- `scan_false_1 integration.search_upsert_ms`: `14003` -> `126`
- `search_rebuild_check_ms`: now probe-only instead of matching total scan duration

## Traceability

Current implementation ticket:

- `C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/bf094901-cdb6-4b25-8ccd-3eb7716f9d20/ticket.toml`

Remaining-work tracker:

- `C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/cadf78e8-a243-4d1c-8c1b-451978bb05ea/ticket.toml`

Follow-up child tickets:

- `C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/3e4718af-3fd3-40a4-ac89-d298c99c806a/ticket.toml`
- `C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/875919d5-558c-46a8-a83f-02a6756a1e0e/ticket.toml`
- `C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/013b57bd-2e8c-4d4d-87c8-6f8687a195c8/ticket.toml`

Supporting context tickets:

- `C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/1b34dbe7-7055-45ae-8c3c-068adef1ca84/ticket.toml`
- `C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/03ed4121-ec7e-4d5f-adb4-4d3846af8031/ticket.toml`
- `C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/ff6637f5-01f6-46c3-b727-e1a19ee0f202/ticket.toml`
- `C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/0a510279-5482-4c4f-8cb5-fad3baa57427/ticket.toml`
- `C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/751f0e71-a857-484f-a45e-09717f086321/ticket.toml`

Implementation and evidence surfaces:

- `memory-api/crates/ticket-api/tests/e2e_perf_move_health.rs`
- `memory-api/crates/ticket-api/src/storage/store/scan.rs`
- `memory-api/crates/ticket-api/src/storage/move_planner.rs`
- `memory-api/crates/memory-api/src/storage/search.rs`
- `memory-api/crates/memory-api/src/storage/move_kernel.rs`
