<!-- aligned-structure:v1 -->

# Summary

Define a shared observability contract for `memory-api`, its domain crates, context-stack graph operations, and every CLI, MCP, HTTP, benchmark, test, and long-running server transport so internal operations can be traced consistently with the Rust `tracing` ecosystem.

## Behavior Story

Define a shared observability contract for `memory-api`, its domain crates, context-stack graph operations, and every CLI, MCP, HTTP, benchmark, test, and long-running server transport so internal operations can be traced consistently with the Rust `tracing` ecosystem.

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

Define a shared observability contract for `memory-api`, its domain crates, context-stack graph operations, and every CLI, MCP, HTTP, benchmark, test, and long-running server transport so internal operations can be traced consistently with the Rust `tracing` ecosystem.

The architecture must make high-signal diagnostics available in files and, as `log-api` matures, in the log store with searchable metadata and cross-store references to tickets, specs, tests, benchmarks, journals, graph visualizations, and agent sessions.

# Problem

The repository already has useful tracing and journaling pieces, but they are fragmented:

- `context-trace` provides graph visualization events and test-log capture for context-domain operations.
- `context-api` already has per-command trace capture and log parsing/query commands.
- `viewer-api` provides reusable server tracing/file logging for viewer tools.
- `memory-api` has a domain-neutral cross-store move kernel with `MovePlan`, `MoveJournal`, `resume_move`, and `rollback_move`, but this journal model is scoped to move operations.
- Several `memory-api` HTTP and MCP tools initialize `tracing-subscriber` independently.
- `log-api` currently models validation-log captures, not general runtime log sessions, operation journals, or live-indexed diagnostic streams.
- Domain crates emit sparse or inconsistent spans, so complex store, search, move, board, graph, and transport behavior can be hard to reconstruct after a failure.

This makes observability optional and uneven. Long-running HTTP transports are especially difficult to debug because server behavior, domain store decisions, journaled operation state, graph replay state, and test/agent context are not consistently correlated.

# Scope

This spec covers the architecture and rollout requirements for:

- `memory-api/crates/{memory-api,ticket-api,spec-api,doc-api,rule-api,audit-api,session-api,test-api,log-api}` and future memory-domain crates.
- `context-stack` graph operation logs and replayable graph-operation journals where they feed log-viewer visualizations.
- `memory-api/tools/cli/*`, `memory-api/tools/mcp/*`, and `memory-api/tools/http/*` transport layers.
- Shared tracing setup, log file creation, log metadata, operation journal metadata, and log-api indexing/search contracts.
- Correlation metadata for test runs, benchmarks, HTTP request ids, MCP tool calls, CLI invocations, ticket/spec ids, validation execution ids, graph operation ids, journal ids, and agent/session ids.

# Non-goals

- Replacing `tracing` with a custom logging facade.
- Duplicating separate tracing initialization code in every binary.
- Requiring the unfinished `log-api` store before file logging provides value.
- Treating trace logs as the durable source of truth for reversible state changes.
- Capturing secrets or full request bodies by default.
- Completing every instrumentation point in one implementation ticket.

# Architecture Direction

## Shared logging facility

Create one shared Rust logging/observability facility for memory-system tools. It may live in `log-api` if that crate owns runtime log metadata, or in a small shared crate if dependency direction requires keeping domain models independent.

The shared facility should provide:

- one initialization API for CLI, MCP, HTTP, tests, benchmarks, and long-running servers
- configuration from environment variables plus optional TOML config
- stdout/stderr and rolling JSONL file sinks
- `EnvFilter` support with module/target-specific granularity
- a non-blocking writer and guard ownership pattern suitable for server lifetimes
- common span fields and event naming conventions
- helpers/macros for domain-operation spans where they reduce repetition
- optional log-api metadata registration when a log store is available

Existing ad hoc subscribers in memory-api tools should be replaced by this shared facility. Existing `viewer-api::init_tracing_full`, `context-api::tracing_capture`, and `context-trace` test tracing should be treated as design inputs; avoid copy-pasting their internals into every domain transport.

## Logs, journals, and visualization events

The design separates three related artifacts:

- **Trace logs** observe runtime behavior. They are append-only diagnostic records with timing, span hierarchy, levels, targets, errors, and structured fields. They are ideal for search, profiling, and debugging, but not sufficient as the durable source of truth for reversible operations.
- **Operation journals** describe planned and applied state transitions. They are deterministic ledgers with preflight inputs, validation blockers, ordered steps, phase transitions, rollback/resume metadata, touched files/entities, and recovery instructions. They are the source of truth for replayable or revertable operations.
- **Visualization events** describe UI replay state. They may be emitted through tracing logs for live observation, but should have stable operation/journal ids so log-viewer can replay them from log files, journals, or derived indexes.

The current move kernel's `MovePlan`/`MoveJournal` proves the viability of journaled operation execution for cross-store file mutations. The broader design should generalize that pattern into an `OperationJournal` model without weakening the existing move journal compatibility.

## Operation journaling contract

A generalized journal should support:

- read-only preflight planning before mutation
- explicit blockers and warnings
- ordered planned steps with stable ids
- phase transitions (`planned`, `locked`, `applied_step`, `validated`, `completed`, `failed`, `rolled_back`, or domain-specific refinements)
- touched entities, touched files, and previous content or inverse operations where rollback is supported
- resume and rollback entrypoints where the operation is reversible
- replay entrypoints for non-mutating visualizations and postmortems
- links to trace log sessions, benchmark/profile runs, tickets, specs, tests, docs, and agent/session ids

Not every journal is rollbackable. The journal schema must represent reversibility explicitly:

- `replayable`: can reconstruct what happened or what would happen
- `rollbackable`: can mechanically restore previous state
- `manual_recovery`: requires human follow-up but records enough context to guide it

### OperationJournal envelope (v1)

`OperationJournal` v1 is the canonical schema envelope for generalized journals and must be able to represent existing `MoveJournal` recovery data without loss.

Required top-level fields:

- `journal_id`, `operation_id`, `run_id`, optional `session_id`
- `schema_version` (initial value `operation-journal/v1`)
- `operation_kind` and `component`
- `preflight` block: inputs, blockers, warnings, and readiness state
- `steps[]`: stable `step_id`, planned mutation metadata, deterministic affected entities/files, optional inverse operation data for rollback-capable operations
- `phases[]`: ordered phase transitions with phase key and lifecycle state
- `reversibility`: one of `replayable`, `rollbackable`, `manual_recovery`
- `recovery`: resume/rollback guidance and failure context
- `links`: trace session ids, ticket/spec/doc/test/benchmark references, graph operation ids

Modeling requirements:

- Existing move apply/resume/rollback journals must map to this envelope without dropping inverse/recovery metadata.
- Non-mutating graph/search journals and rollbackable mutation journals must both be representable using the same envelope.
- Trace logs are linked diagnostics only; they are never a substitute rollback ledger.

### Storage and index ownership decision

- Journal artifacts are stored under `.log/<workspace_slug>/journals/` as JSON documents keyed by `journal_id`.
- `log-api` owns metadata indexing and query surfaces for journal records, including cross-store links and lifecycle filters.
- Domain stores continue to own domain state mutation logic; journal metadata storage/index ownership does not transfer mutation authority to `log-api`.

## Log sessions and metadata

A log file should be represented as a log session with stable metadata, not just an anonymous path. Required metadata includes:

- log id and session/run id
- component/domain and transport (`cli`, `mcp`, `http`, `test`, `bench`, `agent-session`, or `in-process`)
- operation/tool/route where applicable
- workspace root and store root where safe to record
- process id, started_at, ended_at when known, and current status for live logs
- file locator, media type, format, rotation policy, and byte offsets/checkpoints
- filter directives active for the session
- related operation journal ids and graph operation ids
- cross-store links to tickets, specs, docs, validation specs/executions, benchmark ids, and agent/session ids

`log-api` should evolve from validation-only captures into a general runtime-log store that can still specialize validation logs through links to `test-api` executions.

## Cross-store correlation-id contract

The observability contract uses four canonical identifiers to correlate transport spans, logs, journals, replay streams, and benchmark evidence without coupling deterministic replay payloads to profiling-only fields.

| Field | Semantics | Producer | Consumers | Cardinality/lifecycle |
|---|---|---|---|---|
| `operation_id` | Stable id for one logical operation execution (`scan`, `move_apply`, `search`, replay apply). | Operation orchestrator in memory/context domain code. | Transport spans, log session metadata, operation journals, graph replay streams. | One per operation attempt. New value on retry/resume attempt unless resume policy explicitly preserves lineage with `journal_id`. |
| `run_id` | Correlates all operations and transports in one benchmark/test/agent session or process run. | Transport/session bootstrap layer (CLI/MCP/HTTP/test/bench harness). | Log session index, benchmark reports, validation evidence links, cross-operation analytics. | One per run/session boundary. Reused across many `operation_id` values in the same run. |
| `journal_id` | Durable identity of an operation journal ledger record. | Journal store on preflight/create. | Resume/rollback/replay flows, log metadata linkage, evidence queries. | One per journal artifact, stable across apply/resume/rollback events for that journal. |
| `session_id` | External or user-facing session identity (agent session, validation execution session, viewer session). | Session manager / workflow layer. | Ticket/spec/test/log cross-store references, agent handoff correlation. | One per session scope, may span multiple runs depending on workflow policy. |

### Required-field matrix by subsystem

| Surface | Required ids | Notes |
|---|---|---|
| CLI command span/session metadata | `run_id`, `operation_id` | Attach `session_id` when invoked under managed agent/session workflow. |
| MCP tool lifecycle spans | `run_id`, `operation_id`, `session_id` | `session_id` should map to tool session or orchestrator session. |
| HTTP request spans | `run_id`, `operation_id` | If request/session cookie exists, map to `session_id`. |
| Operation journal envelopes | `journal_id`, `operation_id`, `run_id` | `session_id` optional unless operation is session-scoped. |
| Log session metadata rows | `run_id` | Include `operation_id` and `journal_id` when session is operation-bound. |
| Graph replay event streams | `operation_id`, `run_id` | Add `journal_id` when replay stream is derived from a journaled operation. |
| Benchmark evidence/report rows | `run_id`, `operation_id` | Include `session_id` when benchmark is tied to managed validation sessions. |

### Failure and absence handling

- Missing required identifiers in a required surface are validation failures for that surface and must be reported as explicit gaps in ticket/spec evidence.
- Fallback id generation is allowed only at transport/session boundaries; downstream layers must not silently invent replacement ids when an upstream required id is absent.
- Replay payloads must keep deterministic state fields timing-free. Correlation ids are allowed in replay/journal envelopes because they are identity metadata, not timing/profile measurements.
- Resume/rollback flows may keep `journal_id` stable while rotating `operation_id` for each execution attempt, but the relationship must be recorded in journal phase metadata.

## Profiling metadata retention and redaction policy

This policy governs runtime trace/log metadata and profiling evidence artifacts produced by CLI, MCP, HTTP, test, benchmark, and agent-session workflows.

### Retention and rotation defaults

- Active transport log files use rolling JSONL with size-based rotation enabled by default.
- Default retention window for routine diagnostic logs is 14 days; local development may shorten this window, but shared CI or validation evidence should not be retained for less than 7 days.
- Benchmark and validation evidence bundles linked from tickets/specs must retain at least one representative passing run per checklist item until the owning tracker is closed.
- Sampling applies only to high-volume `trace` details; `info`/`warn`/`error` lifecycle events and completion summaries are never sampled away.
- Rotation and retention must preserve log-session metadata rows so references from tickets/specs/journals remain resolvable even when underlying raw files are pruned.

### Redaction and privacy rules

- Redact or hash machine-local absolute paths before they are persisted in shared artifacts unless path precision is explicitly required for debugging.
- Never persist secrets or raw credentials in tracing fields, journal metadata, or benchmark evidence.
- Request/response payload bodies are excluded by default; if temporarily enabled for incident debugging, they must be scope-limited and excluded from long-term retained evidence.
- High-cardinality identifiers from user-controlled input must be normalized or hashed in `info`/`debug` fields; raw values are allowed only under targeted `trace` filters during short-lived debugging windows.
- Ticket/spec/test links remain first-class metadata and are not redacted; they are required for traceability.

### Governance and evidence obligations

- Each profiling-oriented ticket must document the effective retention and redaction settings used for its captured evidence.
- `ff6637f5` checklist evidence must include confirmation that retained artifacts honor this policy while preserving required run metadata and replay boundaries.
- Deviations from defaults (longer retention, disabled sampling, expanded field capture) require explicit ticket/spec notes and an expiry or rollback plan.

## Deterministic replay versus profiling evidence boundary

Replay and rollback correctness depends on separating deterministic state artifacts from diagnostic timing/profiling artifacts.

### Deterministic replay artifacts (allowed content)

- Operation and graph replay envelopes may include stable identity and ordering metadata such as `operation_id`, `journal_id`, `run_id`, step index, phase label, transition type, and deterministic entity references.
- Journal steps may include deterministic pre/post state descriptors, planned mutations, and reversible inverse metadata where rollback is supported.
- Validation-relevant counts that are deterministic for the same input and operation ordering may be included.

### Profiling-only artifacts (must stay out of replay payloads)

- Wall-clock timings (`*_ms`, duration histograms, percentiles such as `p50/p95/p99`).
- CPU/memory/system-load measurements and host-specific telemetry.
- Sampling-rate diagnostics and high-volume trace-only detail events used for performance analysis.

These profiling values belong in trace/log metadata, benchmark artifacts, and profile evidence records, not in deterministic replay journals.

### Boundary enforcement rules

- Replay/journal schemas must reject profiling-only timing fields in deterministic state sections.
- Any timing value needed for diagnostics must be emitted on linked log/profile surfaces keyed by correlation ids, not embedded into replay-critical payload state.
- Resume and rollback logic must derive behavior from deterministic journal state only; profile metadata may annotate but never drive mutation decisions.
- Evidence used for `ff6637f5` checklist closure must show that replay payload snapshots remain timing-free while linked profiling artifacts carry timing/percentile outputs.

## Canonical profiling and tracing phase taxonomy

All profiling-sensitive operations must emit phase keys from this canonical taxonomy so logs, journals, benchmark evidence, and validation checks can be compared across crates and transports.

### Phase-key naming contract

- Phase keys are lowercase snake_case and stable across releases unless an explicit migration note is recorded.
- Keys represent operation structure, not implementation detail names that may churn.
- When an operation does not execute a phase, the phase is omitted (do not emit synthetic zero-duration phases).

### Canonical phase keys by operation family

| Operation family | Canonical phase keys |
|---|---|
| `open_or_init` | `open_or_init.bootstrap`, `open_or_init.workspace_resolve`, `open_or_init.store_open`, `open_or_init.scan_roots`, `open_or_init.reconcile` |
| `scan` | `scan.discover_roots`, `scan.read_entries`, `scan.integrate_index`, `scan.compute_workflow_facts`, `scan.finalize` |
| Integration path | `integration.manifest_parse`, `integration.index_upsert`, `integration.edge_write`, `integration.description_read`, `integration.search_upsert` |
| Workflow recompute | `workflow.fetch_dependency_edges`, `workflow.fetch_dependency_tickets`, `workflow.compute_unresolved`, `workflow.write_facts` |
| Move apply/resume/rollback | `move.preflight_validate`, `move.lock`, `move.apply_steps`, `move.validate`, `move.resume`, `move.rollback`, `move.complete` |
| Graph replay | `graph.load_stream`, `graph.apply_delta`, `graph.snapshot_optional`, `graph.emit_visualization`, `graph.complete` |

### Required fields per phase emission

Each phase completion event/span must include:

- `phase_key` (canonical key from table above)
- `operation_id`, `run_id` (and `journal_id` when journal-backed)
- `component` and `operation_kind`
- `elapsed_ms` for profile evidence surfaces
- deterministic count fields when applicable (for example `entry_count`, `edge_count`, `step_count`)

Timing distributions (`p50/p95/p99`) are reported in benchmark/profile artifacts aggregated by `phase_key`; they are not replay payload fields.

### Validation and compatibility rules

- New instrumentation must map to existing canonical keys where possible; introducing a new key requires spec/ticket traceability notes.
- Deprecated keys must keep a compatibility window where old and new keys are both queryable in log search/index surfaces.
- `ff6637f5` checklist evidence should include at least one run demonstrating phase-key alignment for integration and workflow recompute families.

## Instrumentation contract

Domain crates should instrument major internal operations with structured spans/events, including:

- workspace/store discovery and resolution
- file reads/writes, scans, index reconciliation, and generated sidecar operations
- ticket/spec/doc/rule/audit/session/test/log create/read/update/delete/query flows
- graph traversal, dependency checks, board state changes, and move preflight/apply/rollback/resume flows
- operation journal preflight/apply/resume/rollback/replay flows
- search/index queries and result counts
- validation execution recording and evidence/link resolution
- benchmark/profiling run boundaries and measured durations
- transport dispatch, request/tool lifecycle, status codes, durations, and error classification

Returned profiling artifacts such as `phase_timings_ms`, root-entry count maps, and benchmark summaries remain useful as deterministic test/bench evidence, but they are not a substitute for tracing. When an operation already measures internal phases for reports or journals, the same phase boundaries should also emit tracing spans or completion events so transport logs, log-api indexing, and future correlation tooling can observe the work without parsing return values.

For profile-sensitive store operations such as `open_or_init`, `scan`, and journaled move apply/resume/rollback, prefer one outer `info` span per operation, nested `debug` spans/events per phase or root, stable completion events with counts and elapsed timing fields, and reserve per-record details for `trace` level only.

Granularity should be level-based:

- `error`: operation failed or state may be inconsistent
- `warn`: degraded behavior, stale data, skipped records, validation warnings
- `info`: lifecycle boundaries, high-level operation outcomes, request/tool summaries
- `debug`: branch decisions, counts, resolved paths, IDs, query predicates, elapsed timings
- `trace`: per-record or tight-loop details, enabled only with targeted filters

Events must use stable snake_case field names and avoid high-cardinality or sensitive data unless explicitly requested at `trace` level.

## Context-stack graph replay

`GraphOpEvent` already carries step, operation type, transition, path id, path graph snapshots, and graph mutation deltas through tracing logs for log-viewer. This is viable as a visualization event format, but not yet a complete operation journal.

The graph journaling design should define a replayable format that can:

- group graph events by operation id/path id/run id
- preserve deterministic step ordering
- distinguish observation-only search/read events from mutation-capable insert/update events
- carry graph deltas and optional snapshots for UI replay and validation
- link each graph replay stream to a trace log session and, when relevant, an operation journal
- support benchmark/profiling timing without polluting the replay state

### Graph replay stream format v1

The canonical replay artifact is `GraphReplayStream` with envelope version `graph-replay/v1`.

Top-level envelope fields:

- `schema_version`: fixed string `graph-replay/v1`
- `stream_id`: stable id for one replay stream artifact
- `operation_id`, `run_id`, optional `journal_id`, optional `session_id`
- `operation_kind`: `search`, `read`, `insert`, `update`, `delete`, or future extension
- `component`: context subsystem name (for example `context-search`, `context-insert`)
- `path_id`: logical path/traversal identity when applicable
- `started_at`, optional `ended_at`
- `steps[]`: ordered deterministic replay steps
- `links`: trace/log/session/spec/ticket/test/benchmark cross-store links

Step shape (`steps[]`):

- `step_index` (monotonic, zero-based)
- `transition_kind` (maps to existing `Transition` semantics)
- `event_kind`: `observation` or `mutation`
- `delta` (deterministic graph delta payload when available)
- optional `snapshot_ref` (reference to bounded snapshot payload)
- `entity_refs` (stable ids for affected graph entities)

Deterministic requirements:

- Replay ordering is defined only by `step_index`; wall-clock timestamps are advisory.
- `delta`, `transition_kind`, and `entity_refs` must be sufficient to reproduce graph state progression for the same input.
- Timing and percentile metrics are excluded from `steps[]` and carried only in profile evidence linked by `operation_id` and `run_id`.

### Observation-only vs mutation-capable streams

- Observation-only operations (`search`/`read`) are replayable but not rollbackable and may omit `journal_id`.
- Mutation-capable operations (`insert`/`update`/`delete`) require `journal_id` whenever an operation journal exists.
- For mutation streams, each replay step must preserve reversible context through either:
	- direct linkage to rollback-capable `OperationJournal` steps, or
	- explicit `manual_recovery` classification when rollback is not available.

### Snapshot-vs-delta policy

- Deltas are authoritative for deterministic replay.
- Snapshots are optional acceleration aids for UI and validation checkpoints.
- Snapshots must not replace missing deltas for mutation-capable streams.
- When both are present, consumers apply `delta` in step order and treat snapshots as consistency checkpoints.

### Storage and ownership decision

- Canonical replay stream ownership lives in context-stack/context-api emitters because they own graph semantics.
- `log-api` owns indexing, metadata search, and cross-store linkage for replay streams.
- `log-api` may materialize derived indexes or cached projections but must not become the semantic source of truth for replay content.

### Transport and viewer contract

- HTTP, MCP, and CLI surfaces expose replay streams by reference (`stream_id`, `operation_id`, `run_id`) and optional inline bounded step windows.
- log-viewer replay must work from either trace-derived stream events or stored replay artifacts when both carry matching ids.
- Any new replay field must include a compatibility note and remain queryable through legacy readers for one compatibility window.

## Live indexing and search

The first implementation can write JSONL files immediately. `log-api` should then add indexing/search that can operate on completed and active log files by tracking offsets and partial ingestion state.

Required capabilities:

- record and list log sessions with metadata
- record and list operation journals with metadata
- attach/update cross-store links after capture
- tail active JSONL files without corrupting partial records
- incrementally index new entries by file offset
- filter by level, target/module, span name, operation, request id, run id, ticket/spec/test/session links, journal id, graph operation id, time range, and arbitrary structured fields
- support text search and structured query predicates
- expose equivalent shared-library behavior through CLI, MCP, and HTTP transports

# Viability Assessment

The requirements are viable if the implementation keeps the artifact boundaries strict:

- Reuse `tracing` for observation and timing.
- Reuse and generalize the move kernel's journal concepts for planned/reversible operations.
- Treat context-stack `GraphOpEvent` as a visualization stream that can be indexed and linked, not as a replacement for operation journals.
- Let `log-api` own metadata, indexing, search, and cross-store links rather than duplicating this in every transport.

Primary risks:

- Overloading log files as rollback journals would make recovery unreliable.
- A shared logging crate can create dependency cycles if it depends on domain crates.
- High-volume `trace` events from graph operations can overwhelm live indexing without sampling, filters, or bounded retention.
- File paths, request payloads, and benchmark traces can leak sensitive or unstable machine-local data if redaction rules are not explicit.
- Replaying mutable graph operations requires deterministic deltas and schema versioning; snapshots alone are useful for visualization but weak for validation.
- Long-running servers need guard ownership and shutdown flushing; leaking guards or losing them can drop logs.

# First-Pass Domain Coverage Map

This first pass defines crate-by-crate instrumentation ownership, stable target prefixes, required summary fields, and journal expectations so follow-on implementation tickets can instrument without inventing field names.

| Crate | Major operation families | Span/event target prefix | Required summary fields (snake_case) | Journal requirement |
|---|---|---|---|---|
| `memory-api` | store open/init, scan/reconcile, move preflight/apply/resume/rollback | `memory_api.runtime` and `memory_api.move` | `operation_id`, `run_id`, `journal_id`, `phase_key`, `workspace_root`, `store_root`, `step_count`, `elapsed_ms`, `status` | rollbackable journal for move flows; replayable journal for non-mutating planning |
| `ticket-api` | CRUD/query, dependency graph/subgraph/topgraph, health/next, board check-in/heartbeat/check-out, move flows | `ticket_api.store`, `ticket_api.graph`, `ticket_api.board`, `ticket_api.move` | `operation_id`, `run_id`, `journal_id`, `ticket_id`, `dependency_count`, `dependee_count`, `edge_count`, `elapsed_ms`, `status` | rollbackable journal for cross-workspace move; log-only for CRUD/query/graph reads |
| `spec-api` | CRUD/query, section add/delete/get/list, refs validate, move flows | `spec_api.store`, `spec_api.sections`, `spec_api.refs`, `spec_api.move` | `operation_id`, `run_id`, `journal_id`, `spec_id`, `section_count`, `ref_count`, `elapsed_ms`, `status` | rollbackable journal for move; log-only for read/update paths |
| `doc-api` | CRUD/query, import/index sync, move flows | `doc_api.store`, `doc_api.index`, `doc_api.move` | `operation_id`, `run_id`, `journal_id`, `doc_id`, `entry_count`, `elapsed_ms`, `status` | rollbackable journal for move; replayable/manual_recovery journal for bulk import where needed |
| `rule-api` | CRUD/query, rule sync-targets generation, move flows, feedback recording | `rule_api.store`, `rule_api.sync`, `rule_api.move`, `rule_api.feedback` | `operation_id`, `run_id`, `journal_id`, `rule_id`, `target_count`, `generated_file_count`, `elapsed_ms`, `status` | rollbackable journal for move; manual_recovery journal for sync side effects outside store |
| `audit-api` | audit run creation, evidence ingest/query, move flows | `audit_api.store`, `audit_api.run`, `audit_api.move` | `operation_id`, `run_id`, `journal_id`, `audit_run_id`, `finding_count`, `elapsed_ms`, `status` | replayable journal for ingest pipelines; rollbackable for move |
| `session-api` | session create/update/list/get, predecessor linkage, move flows | `session_api.store`, `session_api.lifecycle`, `session_api.move` | `operation_id`, `run_id`, `journal_id`, `session_id`, `predecessor_session_id`, `elapsed_ms`, `status` | log-only for session lifecycle; rollbackable for move |
| `test-api` | validation spec/execution record/get/list, provenance links, move flows | `test_api.store`, `test_api.validation`, `test_api.move` | `operation_id`, `run_id`, `journal_id`, `validation_spec_id`, `execution_id`, `outcome`, `elapsed_ms`, `status` | replayable journal for evidence ingestion batches; rollbackable for move |
| `log-api` | runtime log session record/get/list/filter, operation-journal metadata index/query/tail | `log_api.session`, `log_api.journal`, `log_api.index` | `operation_id`, `run_id`, `journal_id`, `session_id`, `component`, `transport`, `offset`, `entry_count`, `elapsed_ms`, `status` | log-only for session metadata; replayable journal metadata envelope for indexed journals |

## High-volume filtering and sampling guidance (first pass)

- Emit one `info` completion event per operation with stable summary fields; never sample away completion events.
- Keep per-record loops at `trace` level only and gate with target-specific filters.
- Prefer aggregate counters on `debug` (`entry_count`, `edge_count`, `step_count`) over per-item identifiers.
- For graph and reconcile workloads, sample only `trace` details and preserve deterministic ordering metadata in journals.
- Hash or normalize user-controlled high-cardinality values before `info`/`debug` emission.
- Use `phase_key` consistently for profile rollups; keep percentile timing outputs in profiling artifacts, not deterministic journals.

## Per-operation baseline matrix (pass 2)

This matrix expands the first-pass crate map into operation-level defaults for span target naming, completion event shape, and journal class.

| Crate | Operation | Target prefix | Completion event | Required fields | Journal class |
|---|---|---|---|---|---|
| `memory-api` | `open_or_init` | `memory_api.runtime.open_or_init` | `memory_api_open_or_init_complete` | `operation_id`, `run_id`, `phase_key`, `workspace_root`, `store_root`, `elapsed_ms`, `status` | log-only |
| `memory-api` | `scan` and reconcile | `memory_api.runtime.scan` | `memory_api_scan_complete` | `operation_id`, `run_id`, `phase_key`, `entry_count`, `edge_count`, `elapsed_ms`, `status` | log-only |
| `memory-api` | `move_apply` | `memory_api.move.apply` | `memory_api_move_complete` | `operation_id`, `run_id`, `journal_id`, `phase_key`, `step_count`, `elapsed_ms`, `status` | rollbackable |
| `memory-api` | `move_resume` and `move_rollback` | `memory_api.move.resume` and `memory_api.move.rollback` | `memory_api_move_complete` | `operation_id`, `run_id`, `journal_id`, `phase_key`, `step_count`, `elapsed_ms`, `status` | rollbackable |
| `ticket-api` | `create/get/update/list/search` | `ticket_api.store.crud` | `ticket_api_crud_complete` | `operation_id`, `run_id`, `ticket_id`, `result_count`, `elapsed_ms`, `status` | log-only |
| `ticket-api` | `subgraph/topgraph/health/next` | `ticket_api.graph.query` | `ticket_api_graph_query_complete` | `operation_id`, `run_id`, `node_count`, `edge_count`, `elapsed_ms`, `status` | log-only |
| `ticket-api` | `board_check_in/heartbeat/check_out` | `ticket_api.board.lifecycle` | `ticket_api_board_lifecycle_complete` | `operation_id`, `run_id`, `ticket_id`, `entry_id`, `elapsed_ms`, `status` | log-only |
| `ticket-api` | `move_preflight/apply/resume/rollback` | `ticket_api.move.lifecycle` | `ticket_api_move_complete` | `operation_id`, `run_id`, `journal_id`, `phase_key`, `ticket_id`, `elapsed_ms`, `status` | rollbackable |
| `spec-api` | `create/get/update/list/search` | `spec_api.store.crud` | `spec_api_crud_complete` | `operation_id`, `run_id`, `spec_id`, `result_count`, `elapsed_ms`, `status` | log-only |
| `spec-api` | `section_add/section_delete/section_get/section_list` | `spec_api.sections.lifecycle` | `spec_api_sections_complete` | `operation_id`, `run_id`, `spec_id`, `section_count`, `elapsed_ms`, `status` | log-only |
| `spec-api` | `refs_validate` | `spec_api.refs.validate` | `spec_api_refs_validate_complete` | `operation_id`, `run_id`, `spec_id`, `ref_count`, `invalid_ref_count`, `elapsed_ms`, `status` | log-only |
| `spec-api` | `move_preflight/apply/resume/rollback` | `spec_api.move.lifecycle` | `spec_api_move_complete` | `operation_id`, `run_id`, `journal_id`, `phase_key`, `spec_id`, `elapsed_ms`, `status` | rollbackable |
| `doc-api` | `create/get/update/list/search` | `doc_api.store.crud` | `doc_api_crud_complete` | `operation_id`, `run_id`, `doc_id`, `result_count`, `elapsed_ms`, `status` | log-only |
| `doc-api` | `import/index_sync` | `doc_api.index.sync` | `doc_api_index_sync_complete` | `operation_id`, `run_id`, `entry_count`, `phase_key`, `elapsed_ms`, `status` | replayable or manual_recovery |
| `doc-api` | `move_preflight/apply/resume/rollback` | `doc_api.move.lifecycle` | `doc_api_move_complete` | `operation_id`, `run_id`, `journal_id`, `phase_key`, `doc_id`, `elapsed_ms`, `status` | rollbackable |
| `rule-api` | `create/get/update/list/search` | `rule_api.store.crud` | `rule_api_crud_complete` | `operation_id`, `run_id`, `rule_id`, `result_count`, `elapsed_ms`, `status` | log-only |
| `rule-api` | `sync_targets` | `rule_api.sync.targets` | `rule_api_sync_complete` | `operation_id`, `run_id`, `generated_file_count`, `target_count`, `elapsed_ms`, `status` | manual_recovery |
| `rule-api` | `move_preflight/apply/resume/rollback` | `rule_api.move.lifecycle` | `rule_api_move_complete` | `operation_id`, `run_id`, `journal_id`, `phase_key`, `rule_id`, `elapsed_ms`, `status` | rollbackable |
| `audit-api` | `record_run/list_runs/get_run` | `audit_api.run.lifecycle` | `audit_api_run_complete` | `operation_id`, `run_id`, `audit_run_id`, `finding_count`, `elapsed_ms`, `status` | log-only |
| `audit-api` | `ingest/query_evidence` | `audit_api.evidence.index` | `audit_api_evidence_complete` | `operation_id`, `run_id`, `entry_count`, `result_count`, `elapsed_ms`, `status` | replayable |
| `session-api` | `create/get/update/list` | `session_api.store.lifecycle` | `session_api_lifecycle_complete` | `operation_id`, `run_id`, `session_id`, `predecessor_session_id`, `elapsed_ms`, `status` | log-only |
| `session-api` | `move_preflight/apply/resume/rollback` | `session_api.move.lifecycle` | `session_api_move_complete` | `operation_id`, `run_id`, `journal_id`, `phase_key`, `session_id`, `elapsed_ms`, `status` | rollbackable |
| `test-api` | `record_spec/get_spec/list_specs` | `test_api.validation.specs` | `test_api_validation_specs_complete` | `operation_id`, `run_id`, `validation_spec_id`, `result_count`, `elapsed_ms`, `status` | log-only |
| `test-api` | `record_execution/get_execution/list_executions` | `test_api.validation.executions` | `test_api_validation_executions_complete` | `operation_id`, `run_id`, `execution_id`, `outcome`, `result_count`, `elapsed_ms`, `status` | replayable |
| `test-api` | `move_preflight/apply/resume/rollback` | `test_api.move.lifecycle` | `test_api_move_complete` | `operation_id`, `run_id`, `journal_id`, `phase_key`, `validation_spec_id`, `elapsed_ms`, `status` | rollbackable |
| `log-api` | `record/get/list_runtime_session` | `log_api.session.lifecycle` | `log_api_session_complete` | `operation_id`, `run_id`, `session_id`, `component`, `transport`, `elapsed_ms`, `status` | log-only |
| `log-api` | `record/get/list journal metadata` | `log_api.journal.index` | `log_api_journal_index_complete` | `operation_id`, `run_id`, `journal_id`, `operation_kind`, `entry_count`, `elapsed_ms`, `status` | replayable metadata |
| `log-api` | `tail/index/filter/search` | `log_api.index.query` | `log_api_query_complete` | `operation_id`, `run_id`, `offset`, `result_count`, `elapsed_ms`, `status` | log-only |

### Cross-crate event naming and field contract

- Completion events use `<crate>_<operation>_complete` and must include `status` and `elapsed_ms`.
- Failure events use `<crate>_<operation>_failed` and include `error_kind` plus correlation fields.
- Mutation-capable operations that emit `journal_id` must also emit `operation_id` and `run_id` on the same completion event.
- `phase_key` is required for profiled multi-phase operations and should use the canonical taxonomy defined earlier in this spec.
- Field names remain stable snake_case; if a rename is required, emit old and new fields during one compatibility window.

## Event drift reconciliation (current emissions -> canonical contract)

This mapping records currently emitted event names and the canonical names they should converge to. It is normative for migration planning and avoids one-off renames by transport.

| Surface | Current event name(s) | Canonical target name | Migration note |
|---|---|---|---|
| ticket move (CLI) | `ticket_cli_move_complete` | `ticket_api_move_complete` | Emit both during compatibility window; retire CLI-specific name after downstream query migration. |
| ticket move (HTTP) | `ticket_http_move_complete` | `ticket_api_move_complete` | Keep HTTP route metadata fields; normalize event family name. |
| ticket move (MCP) | `ticket_mcp_move_preflight_complete`, `ticket_mcp_move_apply_complete`, `ticket_mcp_move_resume_complete`, `ticket_mcp_move_rollback_complete` | `ticket_api_move_complete` with `phase_key` | Preserve mode in `phase_key` (`move.preflight_validate`, `move.apply_steps`, `move.resume`, `move.rollback`). |
| ticket store open/init | `ticket_store_open_profiled_complete`, `ticket_store_init_profiled_complete`, `ticket_store_open_or_init_complete`, `ticket_store_open_internal_complete` | `ticket_api_store_open_or_init_complete` | Keep profiled timing fields but converge to one family name plus `phase_key`. |
| ticket store scan/reconcile | `ticket_store_scan_complete`, `ticket_store_scan_once_complete`, `ticket_store_phase_complete`, `ticket_reconcile_once_complete` | `ticket_api_scan_complete` and `ticket_api_reconcile_complete` | Separate scan and reconcile families; map legacy phase labels into canonical `phase_key`. |
| ticket reconcile failures | `ticket_reconcile_pending_paths_full_scan_failed`, `ticket_reconcile_pending_paths_orphan_failed` | `ticket_api_reconcile_failed` | Preserve failure subtype in `error_kind`. |
| spec store lifecycle | `spec_store_open_complete`, `spec_store_init_complete`, `spec_store_open_or_init_complete` | `spec_api_store_open_or_init_complete` | Use single lifecycle family with operation-specific `phase_key`. |
| spec store scan/index | `spec_store_scan_complete`, `spec_store_rebuild_slug_index_complete` | `spec_api_scan_complete` and `spec_api_index_complete` | Keep index counts and result counts unchanged; rename family only. |
| spec id resolution | `spec_store_resolve_id_complete` | `spec_api_resolve_id_complete` | Direct rename with no field contract change. |
| ticket-http dotted failures | `ticket.validation_failed`, `internal.task_failed` | `ticket_api_request_failed` | Continue exposing HTTP response classes; normalize event name and carry subtype in `error_kind`. |
| doc-http dotted failure | `doc.artifact_read_failed` | `doc_api_artifact_read_failed` | Direct rename to snake_case family. |

### Compatibility-window policy

- Compatibility window length: two released milestones or 30 days of merged default-branch runtime coverage, whichever is longer.
- During compatibility window, emit both legacy and canonical event names for renamed families.
- Canonical event payload is source of truth for new tooling; legacy payload may be reduced but must preserve correlation fields.
- Required preserved fields during dual-emit: `operation_id`, `run_id`, optional `journal_id`, `status`, `elapsed_ms`, and `error_kind` for failures.
- Removal gate for legacy names: all dependent dashboards, log queries, validation specs, and benchmark/report parsers reference canonical names.
- Deprecation evidence must be linked from the owning ticket before removing legacy event names.

# Open Decisions

- Owning crate: should shared runtime logging live inside `log-api`, `memory-api`, or a small dependency-light crate used by both?
- Journal store: should generalized operation journals live under `.log`, each domain store, or a new `.journal` area indexed by `log-api`?
- Schema versioning: what stable envelope version covers log entries, operation journals, and graph replay events?
- Reversibility: which memory-store operations are rollbackable by design, and which are replay/manual-recovery only?
- Context-stack boundary: should graph journals live in `context-api` as domain artifacts, or be normalized into `log-api` only after capture?
- Retention: what default retention, rotation, and sampling policies apply to active server logs and graph trace streams?
- Privacy: what fields are always redacted or hashed by the shared facility?

These governance decisions are resolved by the profiling metadata retention and redaction policy section above and must be treated as the default contract for new observability work.
The journal storage/index decision and schema envelope version are resolved by the OperationJournal envelope (v1) and storage decision sections above.

# Acceptance Criteria

- A single shared initialization/configuration path exists for memory-api CLI, MCP, HTTP, tests, benchmarks, and long-running servers; new transports do not hand-roll subscribers.
- Logs, operation journals, and visualization events have distinct schemas and explicit links between them.
- A generalized operation journal contract exists, derived from the move kernel, with preflight, apply, resume, rollback, replay, and reversibility semantics.
- Every memory-domain crate has an instrumentation map identifying its major operations, required span targets, standard fields, expected result events, and journal requirements.
- Context-stack graph operations have a replayable graph event/journal contract suitable for log-viewer visualization.
- HTTP and MCP transports emit request/tool lifecycle spans that correlate transport work with domain-store operations and journal ids.
- Log files are written as structured JSONL with stable metadata and level/target filters.
- `log-api` can represent runtime log sessions and operation journal metadata, not only validation-log captures, and can link logs to test runs, benchmarks, tickets, specs, docs, journals, graph operations, and agent sessions.
- Active log files can be tailed or incrementally indexed without waiting for process exit.
- Search/filter APIs work over log metadata, journal metadata, and structured log entries, with compact CLI/MCP output for agent workflows.
- Documentation explains the shared field naming convention, journal/replay semantics, privacy rules, and how to enable/disable file and store capture.

# Traceability

- Owning tracker: `.ticket/tickets/73b2cd22-942b-4205-86e5-333df2373211`
- Related tracker: `.ticket/tickets/def88d4e-8a3c-45bc-82c8-bdacae01a479`
- Related context trace plan: `.ticket/tickets/61f78a57-6896-4ad7-9daa-0e9e805aa397`
- Related move kernel spec: `memory-api/.spec/specs/afcaccc9-5577-4556-ab6f-cfbe7a77e430`
- Related move kernel ticket: `memory-api/.ticket/tickets/0a510279-5482-4c4f-8dda-6d333dc1f222`
- Related journaled move execution ticket: `memory-api/.ticket/tickets/bc691249-5a2d-409e-8e7b-2602d80cf61e`
- Related workflow validation spec: `.spec/specs/a4f48d84-50ed-4769-a42f-38321ea9600c`
- Related benchmarking/profiling plan: `.spec/specs/c598ddb2-4d3a-4b81-90ea-8b25a54b8469`
- Correlation-id contract implementation ticket: `.ticket/tickets/529844ac-f7e5-4265-b087-5bd2b597155f`
- Profiling evidence checklist tracker: `.ticket/tickets/ff6637f5-01f6-46c3-b727-e1a19ee0f202`
- Retention/redaction governance ticket: `.ticket/tickets/72b3545c-ceb9-4cb2-a8d4-c146fc9b460a`
- Deterministic replay boundary ticket: `.ticket/tickets/8b1eab26-389b-4125-86ec-886c9d48702b`
- Phase taxonomy contract ticket: `.ticket/tickets/1c56033e-5c30-46bd-a0bd-2209b8841876`
- Operation journal schema/store contract ticket: `.ticket/tickets/6c859ac3-14c9-4d9d-b428-5b0cca03e23a`
- Profiling metadata and percentile evidence ticket: `.ticket/tickets/de8719bf-a58a-41d1-891e-2b87894e6c02`
- Profile evidence linkage validation ticket: `.ticket/tickets/87ff70d7-36a8-453d-9ce2-3fec830b163f`
