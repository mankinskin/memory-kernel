&lt;!-- aligned-structure:v2 --&gt;

# log-api v2 OperationJournal envelope contract

## Target Code Location

- [workflow-tools/memory-kernel/src/operation_journal.rs](../../memory-kernel/src/operation_journal.rs) — defines the shared `OperationJournal` envelope and the `from_move_journal` / `into_move_journal` projection boundary.
- [workflow-tools/log/crates/log-api/src/store.rs](../../log/crates/log-api/src/store.rs) — `LogStoreConfig::record_operation_journal`, `get_operation_journal`, `list_operation_journals`.
- [workflow-tools/log/crates/log-api/src/store_tests.rs](../../log/crates/log-api/src/store_tests.rs) — persistence/read-back/query test coverage.
- [workflow-tools/memory-kernel/src/storage/move_kernel/internal.rs](../../memory-kernel/src/storage/move_kernel/internal.rs) — `persist_journal` / `load_journal`, the move-kernel's own recovery persistence, distinct from `log-api`'s envelope store.

## Naming Conventions

- `OPERATION_JOURNAL_SCHEMA_VERSION = "operation-journal/v1"` is the envelope schema tag stored on every `OperationJournal.schema_version`; it is independent of this spec's own `format_version = 2`.
- `OperationJournal` fields use the vocabulary this contract governs: `operation_kind` (e.g. `"move"`), `component` (owning domain crate, e.g. `"memory-kernel"`), `preflight` (`OperationPreflight`), `steps` (`Vec<OperationJournalStep>`), `phases` (`Vec<OperationJournalPhase>`), `reversibility` (`OperationReversibility`: `Replayable` | `Rollbackable` | `ManualRecovery`), `recovery` (`OperationRecovery`), `links` (`OperationJournalLinks`), `domain_data` (opaque `serde_json::Value`).
- `LogStoreConfig` methods use the `record_*` / `get_*` / `list_*` naming convention shared with `ValidationLogCapture` and `RuntimeLogSession` in the same store.
- Criterion ids in this spec use the `<component>-crit-<slug>` scheme (e.g. `log-api-crit-envelope-fidelity`, `memory-kernel-crit-recovery-ownership`); evidence ids use `ev-<slug>`.

## Reading Order

1. [operation_journal.rs](../../memory-kernel/src/operation_journal.rs) — the envelope type and its only producer/consumer methods.
2. [log-api store.rs](../../log/crates/log-api/src/store.rs) — the persistence/query surface this contract binds.
3. [transcripts/12-09-2026_spec-v2-move-tooling/ROADMAP.md](../../../transcripts/12-09-2026_spec-v2-move-tooling/ROADMAP.md) — W1-W4 sequencing; this spec is the W1 deliverable and the gate W2-W4 tickets wait on.
4. [transcripts/12-09-2026_spec-v2-move-tooling/INTERVIEW-1.md](../../../transcripts/12-09-2026_spec-v2-move-tooling/INTERVIEW-1.md) — the two-part review gate this spec must clear (`spec_health` pass + durable reviewer approval note) before W2-W4 tickets are created.
5. [Memory-system observability and log-api runtime diagnostics](../../.workflow-tools/memory-kernel/.spec/specs/aa769a27-2721-4b9d-880c-5c4e2f8136a7/body.md) — neighboring `log-api`-component spec covering runtime diagnostics; distinct concern, no shared criteria with this contract.

## Requester Input

> Author a v2 Spec manifest for `log-api` that defines the `OperationJournal` envelope, field semantics, component ownership, acceptance criteria, evidence requirements, contract edges, and fulfillment expectations. Describe the current split accurately: move/recovery journals are persisted by `memory-kernel`, while `log-api` provides separate JSON operation-journal persistence and queries. Treat the manifest as governance and evidence contract; keep runtime validation in `log-api` code and tests.

## Responsibility

This contract governs the shared `OperationJournal` envelope schema and states, precisely, which component owns which part of operation recovery:

- `memory-kernel` (`workflow-tools/memory-kernel/src/operation_journal.rs`) owns `MoveJournal` domain state, phase transitions, and recovery decision-making. It is the only component that constructs an `OperationJournal` (via `from_move_journal`) or reconstructs a `MoveJournal` from one (via `into_move_journal`). `memory-kernel`'s own move-recovery persistence (`persist_journal` / `load_journal` in `storage/move_kernel/internal.rs`) is a separate, pre-existing legacy `MoveJournal` store and is unaffected by this contract.
- `log-api` (`workflow-tools/log/crates/log-api/src/store.rs`) owns durable JSON persistence and query of `OperationJournal` envelopes as opaque records. It does not construct, project, mutate, or interpret `domain_data`, and it performs no move/apply/rollback logic.

## Interfaces And Dependencies

- `log-api` depends on the `OperationJournal` type exported by `memory-kernel` (`use memory_kernel::OperationJournal;` in `log-api/src/store.rs`); it does not depend on `memory-kernel`'s move-kernel internals.
- `LogStoreConfig::record_operation_journal(&OperationJournal) -> Result<PathBuf, LogError>` persists one envelope at `<root>/<workspace_slug>/journals/<journal_id>.json`.
- `LogStoreConfig::get_operation_journal(&str) -> Result<OperationJournal, LogError>` reads one envelope by its durable `journal_id`.
- `LogStoreConfig::list_operation_journals(&OperationJournalQuery) -> Result<Vec<OperationJournal>, LogError>` filters by `operation_kind` and/or `component`, sorted by `journal_id`.
- No caller today drives `log-api`'s operation-journal APIs from `memory-kernel`'s move-kernel path; W2 ([01-journal-contract-and-log-integration.md](../../../transcripts/12-09-2026_spec-v2-move-tooling/01-journal-contract-and-log-integration.md)) decides and implements that integration. This spec does not assume W2 exists yet.

## Behavior

- **Dependent expectation:** if this contract holds, a dependent can persist an `OperationJournal` produced anywhere in the repository through `log-api` and read it back — including every nested field (`preflight`, all `steps`, all `phases`, `reversibility`, `recovery`, `links`, and the full `domain_data` payload) — with no field loss or silent coercion, and can query the stored set by `operation_kind` and/or `component` and get a deterministic, sorted result.
- `from_move_journal` is the sole legitimate way to produce an `OperationJournal` for a move operation today; it derives `journal_id` from the source `MoveJournal.id`, synthesizes `operation_id`/`run_id` via `Uuid::new_v5`, sets `operation_kind = "move"`, `component = "memory-kernel"`, and serializes the entire `MoveJournal` into `domain_data` so `into_move_journal` can restore it exactly.
- `reversibility` is derived, not asserted: `ManualRecovery` when `manual_followups` is non-empty, otherwise `Rollbackable`.
- `log-api` treats `domain_data` as an opaque `serde_json::Value` at all times; it never inspects, mutates, or validates its internal shape.

## Boundaries And Failure Cases

- `into_move_journal` rejects an envelope whose `schema_version` is not `"operation-journal/v1"` or whose `operation_kind` is not `"move"` (`MoveError::Domain`); this is the only domain-shape check `memory-kernel` performs, and `log-api` performs no equivalent check on read.
- The Spec system does **not** enforce runtime JSON validity for `OperationJournal` values persisted through `log-api`. This contract states the intended shape; conformance to it is proven only by the `log-api` and `memory-kernel` tests linked as evidence below, not by anything the spec store itself checks at write time.
- A missing envelope on read returns `LogError::OperationJournalNotFound(journal_id)`; `log-api` never falls back to reconstructing a journal from `memory-kernel`'s legacy `MoveJournal` store.
- This contract does not cover filesystem-level rollback, cross-store migration, batch/set-level move operations, or transport (CLI/MCP/HTTP) behavior — those are W2-W4 concerns per [ROADMAP.md](../../../transcripts/12-09-2026_spec-v2-move-tooling/ROADMAP.md) and are explicitly out of scope here.

## Provider/Consumer Contract

- `log-api` (consumer) → `memory-kernel` (provider): `log-api` consumes the `OperationJournal` type and schema semantics that `memory-kernel::operation_journal` defines and produces; it satisfies `log-api-crit-envelope-fidelity` by round-tripping every field the provider defines, and it relies on `memory-kernel-crit-recovery-ownership` to guarantee `domain_data` is never something `log-api` itself must interpret.
- No component today consumes `log-api`'s persisted `OperationJournal` records as an input to further domain mutation; W2-W4 introduce and must satisfy that edge when they land.

## Examples

A move operation's recovery is projected, persisted, queried, and restored through the two components exactly as follows:

1. `memory-kernel` builds a `MoveJournal` while planning/executing a move and calls `OperationJournal::from_move_journal(&journal)`, producing an envelope with `operation_kind = "move"`, `component = "memory-kernel"`, and `domain_data` holding the full serialized `MoveJournal`.
2. A caller persists that envelope with `LogStoreConfig::record_operation_journal(&envelope)`, which writes `<root>/<workspace_slug>/journals/<journal_id>.json` and never inspects `domain_data`.
3. A later caller may query `list_operation_journals(&OperationJournalQuery { operation_kind: Some("move".into()), component: Some("memory-kernel".into()), limit: None })`; the API defines filtering and `journal_id` sorting, while the current `records_and_queries_operation_journal_by_kind` test exercises only the single-record `operation_kind` filter. Multi-record ordering and combined-filter behavior remain pending W2 evidence in [store_tests.rs](../../log/crates/log-api/src/store_tests.rs).
4. `memory-kernel` (not `log-api`) later calls `envelope.into_move_journal()` to restore the original `MoveJournal` for recovery, exactly as proven by `move_journal_projection_round_trips_recovery_data` in [operation_journal.rs](../../memory-kernel/src/operation_journal.rs).

## Evidence

- **Guards (existing, already passing):** `move_journal_projection_round_trips_recovery_data` ([operation_journal.rs](../../memory-kernel/src/operation_journal.rs)) and `records_and_queries_operation_journal_by_kind` ([store_tests.rs](../../log/crates/log-api/src/store_tests.rs)) already pass; no `test-api` `ValidationSpec` id is registered for them yet, so this spec references them directly as evidence rather than as a governed guard.
- **What `records_and_queries_operation_journal_by_kind` actually proves (narrowed, per independent review):** it constructs and persists exactly one `OperationJournal` fixture, reads it back via `get_operation_journal`, and calls `list_operation_journals` with only `operation_kind: Some("move")` set. It does not construct more than one record, does not exercise the `component` filter alone or in combination with `operation_kind`, and — because only one record ever exists in the test — proves nothing about deterministic `journal_id` ordering across multiple persisted records. Full-field round-trip fidelity is demonstrated only for the fields present on that single fixture, not exhaustively for every `OperationJournal` field independent of fixture shape.
- **Positions:** `log-api-crit-envelope-fidelity` — implemented for the single-fixture case exercised today; full-field fidelity across arbitrary envelopes remains an intended property requiring a future multi-record test. `log-api-crit-no-domain-mutation` — implemented (no domain logic exists in `LogStoreConfig`'s operation-journal methods). `memory-kernel-crit-recovery-ownership` — implemented (`from_move_journal`/`into_move_journal` are the only projection points). `log-api-crit-query-filter` — implemented for single-record, `operation_kind`-only filtering; `component` filtering, combined filters, and deterministic multi-record ordering remain intended properties requiring additional evidence.
- **Governing rule:** [spec-system.instructions.md](../../.agents/instructions/spec/spec-system.instructions.md) requires this spec to be introduced by a governing rule keyed to its readiness; the narrowed positions above are what the introducing rule may present as live and dependable today — the stronger full-fidelity/multi-filter/ordering claims remain "pending", not "coming soon" fiction, but not yet proven either.
- Independent reviewer approval is intentionally not recorded here: [INTERVIEW-1.md](../../../transcripts/12-09-2026_spec-v2-move-tooling/INTERVIEW-1.md) requires a durable approval note from an independent review pass, which this authoring session does not perform. This amendment (W1 repair pass) is itself a correction made in response to an independent review and does not itself constitute the required approval; approval remains a separate, still-pending step.
- Validation commands: `cargo test --manifest-path workflow-tools/spec/crates/spec-api/Cargo.toml`; `cargo test --manifest-path workflow-tools/spec/Cargo.toml --features cli --test bootstrap_tests`; `cargo test --manifest-path workflow-tools/memory-kernel/Cargo.toml`; `cargo test --manifest-path workflow-tools/log/crates/log-api/Cargo.toml`.

## Scope

**In scope:** the `OperationJournal` envelope shape and field semantics; the `memory-kernel` (projection/recovery ownership) vs `log-api` (opaque persistence/query) responsibility split; acceptance criteria and evidence for that split as it exists today.

**Out of scope (non-goals):** implementing or changing any runtime validation in `log-api`; making the Spec system reject arbitrary runtime JSON (it does not, and this spec must not be read as claiming otherwise); executing a live migration; authoring the additional multi-record, multi-field, combined-filter test needed to fully evidence `ac-envelope-fidelity` and `ac-query-determinism` beyond today's single-fixture, single-filter coverage; W2's journal/log integration decision, W3's move lifecycle/batch behavior, and W4's transport alignment — each tracked separately once this spec's review gate closes, per [ROADMAP.md](../../../transcripts/12-09-2026_spec-v2-move-tooling/ROADMAP.md).

No related ticket is linked yet: per [INTERVIEW-1.md](../../../transcripts/12-09-2026_spec-v2-move-tooling/INTERVIEW-1.md), W2-W4 ticket creation is deferred until this spec's `spec_health` pass and reviewer approval note both close the W1 gate.
