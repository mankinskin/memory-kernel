<!-- aligned-structure:v2 -->

# Structured Ticket Entities: multi-file parts, plan freezing, typed refs, and projected reads

## Target Code Location

- [crates/ticket-api/src/model/ticket.rs](../../../../../ticket/crates/ticket-api/src/model/ticket.rs) — `TicketPart` and part-kind model.
- [crates/ticket-api/src/storage/store/parts.rs](../../../../../ticket/crates/ticket-api/src/storage/store/parts.rs) — shared part-write API and validation gate.
- [crates/ticket-api/src/storage/store/migration.rs](../../../../../ticket/crates/ticket-api/src/storage/store/migration.rs) — preflight all split-description segments before publishing parts.
- [crates/ticket-api/src/storage/store/part_layout_migration.rs](../../../../../ticket/crates/ticket-api/src/storage/store/part_layout_migration.rs) and its [planner](../../../../../ticket/crates/ticket-api/src/storage/store/part_layout_migration/planning.rs), [executor](../../../../../ticket/crates/ticket-api/src/storage/store/part_layout_migration/execution.rs), [integrity checks](../../../../../ticket/crates/ticket-api/src/storage/store/part_layout_migration/integrity.rs), and [read-back/undo operations](../../../../../ticket/crates/ticket-api/src/storage/store/part_layout_migration/verify_undo.rs) — bounded dry-run, apply, verify, and undo.
- [crates/ticket-api/src/storage/ticket_fs.rs](../../../../../ticket/crates/ticket-api/src/storage/ticket_fs.rs) and [ticket_fs/part_layout.rs](../../../../../ticket/crates/ticket-api/src/storage/ticket_fs/part_layout.rs) — filesystem persistence, freeze transition, read-only layout snapshots, and transactional publication.
- [crates/ticket-api/src/storage/ticket_fs/transaction.rs](../../../../../ticket/crates/ticket-api/src/storage/ticket_fs/transaction.rs) — ticket-local staged publication, journaling, and recovery.
- [crates/ticket-api/src/storage/ticket_fs/transaction/tests.rs](../../../../../ticket/crates/ticket-api/src/storage/ticket_fs/transaction/tests.rs) — publication-boundary recovery and conflict regressions.
- [crates/ticket-api/src/storage/tests/update_regression_tests.rs](../../../../../ticket/crates/ticket-api/src/storage/tests/update_regression_tests.rs) — regression coverage for lifecycle and persistence behavior.
- [CLI migration command](../../../../../ticket/src/cli/commands/migrate_part_layout.rs) and [batch guard](../../../../../ticket/src/cli/batch/dispatch.rs) — saved-report interface and prohibition on wrapping migration in `ticket batch`.

## Naming Conventions

- A part is addressed only by its stable opaque `TicketPart.id`; its disk path is not an identity.
- Core part kinds are `objective`, `requirements`, `design`, `examples`, `acceptance_criteria`, `review`, `validation`, `notes`, and `amendment`. Free-form kinds remain opaque attachments.
- New and migrated explicit part files use `parts/<kind-slug>.md`, then `parts/<kind-slug>-2.md`, `parts/<kind-slug>-3.md`, and so on for repeated normalized kinds. For example, repeated review parts use `review.md` and `review-2.md`.
- Slug normalization is deterministic and filesystem-safe: apply Unicode NFKC, full default non-Turkic case-folding, then NFKC again; retain lowercase ASCII letters and digits; convert every run of other characters to one hyphen; trim leading/trailing hyphens; use `part` if no ASCII token remains. Ordinals are assigned in manifest order among equal normalized base slugs. Existing paths participate only in collision checks and never select or renumber an ordinal.
- A target path already referenced by another retained part, or occupied by a filesystem entry not owned by the transaction, is a conflict. Fail before mutation rather than overwrite or silently choose another suffix.

## Requester Input

> There is a bug where ticket parts are created empty or failed part creation leaves an empty file. There are many tickets with empty parts because of this. Please investigate the issue, find any tracking tickets for this issue and find examples of empty part files. Then track how part files are created and plan a fix and refactor along this code path.
>
> I am also not happy with the pay parts are named, I think instead of unreadable identifiers, they should only be slug named. The disadvantages of the current format (opaque UUID) are that they are opaque and can be confused with full ticket entity IDs.
>
> Explore and plan a fix.

## Reading Order

1. [Ticket filesystem model](../../../../../ticket/crates/ticket-api/src/model/filesystem.rs) — manifest schema for `[[parts]]` and `[[refs]]`.
2. [Ticket part model](../../../../../ticket/crates/ticket-api/src/model/ticket.rs) — stable part identity, kind, and paths.
3. [Shared part-write API](../../../../../ticket/crates/ticket-api/src/storage/store/parts.rs) — common write gate and caller behavior.
4. [Ticket filesystem persistence](../../../../../ticket/crates/ticket-api/src/storage/ticket_fs.rs) — part-file and manifest mutation, freeze, and recovery entry points.
5. [Ticket transaction layer](../../../../../ticket/crates/ticket-api/src/storage/ticket_fs/transaction.rs) — journal phases, generation commit point, and rollback/finalization.
6. [Part-layout migration planner and executor](../../../../../ticket/crates/ticket-api/src/storage/store/part_layout_migration.rs) — saved dry-run reports, apply, verify, and undo.
7. [Ticket CLI migration command](../../../../../ticket/src/cli/commands/migrate_part_layout.rs) — operation modes and report-file contract.
8. [Transaction recovery tests](../../../../../ticket/crates/ticket-api/src/storage/ticket_fs/transaction/tests.rs) — interrupted publication and conflict cases.
9. [Ticket CLI and transport package](../../../../../ticket/) — caller surfaces that must preserve the shared API contract.
10. [Planning dossier evidence](../../../../../../transcripts/12-10-2026_ticket-part-empty-files/ARTIFACTS.md) — observed empty files, tracking-ticket search, and traced write path.
11. [Requester decisions](../../../../../../transcripts/12-10-2026_ticket-part-empty-files/INTERVIEW.md) — filename ordinals and exact cleanup eligibility.

## Responsibility

### Motivation

Ticket parts are currently persisted as separate files plus manifest metadata. The `ready` transition can materialize absent planning parts with empty content, while part-file and manifest writes are separate operations without cross-file crash recovery. The current UUID-derived filenames also hide the part kind, although `TicketPart.id` already supplies stable identity. The observed empty-file inventory has multiple possible causes; the specification does not attribute every empty file to one defect.

### Dependent Expectation

If this specification is implemented, dependents can rely on all externally reachable part writes rejecting blank content before mutation; `ready` freezing only existing planning parts; part creation, overwrite, and migration recovering to a complete old or new state after interruption; and readable, collision-safe kind-slug paths without changing stable part identities or ordering. CLI, MCP, HTTP, migration, and legacy routes share the same validation and persistence guarantees. Existing typed references, projected reads, and lossless description migration remain available.

## Interfaces And Dependencies

- CLI, MCP, and HTTP part-write entry points call the shared `TicketStore::write_part` API in [parts.rs](../../../../../ticket/crates/ticket-api/src/storage/store/parts.rs); none may bypass its validation, lifecycle, or persistence rules.
- Filesystem operations use the existing per-ticket lock and manifest model in [ticket_fs.rs](../../../../../ticket/crates/ticket-api/src/storage/ticket_fs.rs). Multi-file consistency is provided by journaled recovery, not by claiming that one file rename is a transaction.
- Read projections and legacy reads consume the manifest and part model; migration and undo preserve the same stable IDs and manifest order.
- Typed references remain non-ticket references in `[[refs]]`. Ticket-to-ticket dependency edges remain in the edge model and are not replaced by references.
- Validation evidence is stored in test-api and linked to the implementing ticket and this specification.

## Behavior

### Parts, kinds, and identity

A ticket directory contains `parts/` and a `[[parts]]` manifest table. Each entry carries stable opaque `id`, `kind`, manifest-relative `path`, `frozen`, `created_at`, and optional `supersedes`. Manifest order remains display/creation order and is never an addressing key. Core kinds are schema-validated; free-form kinds round-trip as opaque attachments and remain listable and retrievable. `supersedes` is part of the model from the start; an amendment uses it to identify the frozen part it corrects.

### Content validation and `description_mode`

Every externally reachable part-content write rejects content for which `trim()` is empty before changing any part file, legacy `description.md`, manifest, history, journal, or transaction artifact. This applies to new parts, existing-part overwrites, amendments, undo writes, and the implicit legacy objective route. Clearing content requires a future explicit removal operation; a blank write is not removal.

When `update_ticket` includes a description, callers must explicitly specify `description_mode = replace` or `description_mode = append`. An omitted mode is an error, not a default to replace. This is an intentional compatibility change for every caller supplying a description.

### `ready` lifecycle

Entering `ready` freezes only planning-kind entries already present in the manifest: `objective`, `requirements`, `design`, `examples`, and `acceptance_criteria`. The transition creates no missing part entry or file. Leaving `ready` clears frozen flags as modeled by the lifecycle. Re-entering `ready` freezes the existing planning entries again.

At `ready`, overwriting a frozen part and creating a new planning-kind part are rejected with actionable errors. Non-planning parts and amendment parts remain writable. An amendment records the superseded frozen part ID; there is no force bypass. Planning edits require the normal transition back to a pre-`ready` state.

### Crash-recoverable persistence

Part content, manifest updates, and part-path changes use a ticket-local write-ahead transaction under the ticket lock. The transaction record identifies the transaction, operation (`create`, `overwrite`, `manifest-only`, or `path-migration`), old and new manifest state or recoverable generations, and all final, staged, and backup relative paths. Staged content and manifest generations are written to unique same-directory temporary files and flushed; the journal and containing directory are persisted before final names are published. Publication uses same-directory atomic renames. An overwrite retains its prior file as a transaction-owned backup until the new manifest is published and durable.

When an overwrite leaves `ticket.toml` bytes unchanged, the durable `.ticket-generation` marker is the commit point; when manifest bytes change, the new manifest generation is the commit point. Both paths preserve the complete old or new state during recovery.

All pre-existing targets are backed up before any staged replacement is published, so a path change can reuse a destination only when the same transaction vacates it.

Recovery runs under the ticket lock before ticket reads or mutations. If the new manifest generation was not durably published, recovery removes staged new files and restores transaction-owned backups, leaving the old state visible. If the new generation was durably published, recovery finishes recorded final renames, removes only transaction-owned backups and staging files, and removes the journal. Recovery is idempotent. No manifest entry points to a partially written file, and a failed create does not leave a user-visible orphan. Individual atomic renames are not described as multi-file atomicity; the journal and recovery procedure supply that guarantee. Failure injection covers every publication boundary for create, overwrite, manifest-only, and path-migration operations.

### Part-layout migration and empty-part cleanup

A dedicated part-layout migration provides separate dry-run, apply, and undo operations. Dry-run performs no writes and reports per ticket: proposed UUID-path to slug-path changes, normalized-base collisions, cleanup candidates, conflicts, and skipped entries. Apply consumes a fresh report, rechecks manifest and path fingerprints under lock, uses the recoverable transaction, and records sufficient migration metadata for undo. Undo restores only that migration's immediately prior paths and transaction-owned backups; stale or conflicting state is refused rather than overwritten.

Cleanup eligibility is exactly a manifest-referenced part with a planning kind, `frozen = true`, and a backing file of zero bytes. Apply removes its manifest entry and transaction-owned backing file as part of the recoverable migration. Undo restores the entry in its original manifest position and restores its zero-byte file. Empty non-planning parts, unfrozen parts, nonzero files, and all unreferenced/orphan files are preserved. The migration covers both discovered ticket stores. Inventory counts are diagnostic and do not prove that every empty file shares one cause.

### Description-to-parts migration

The existing one-shot description migration remains lossless. It recognizes `## Review`, `## Status`, `## Validation`, and `## Handoff`, moves those sections into typed parts, creates one `notes` part per matched heading without merging repeated notes sections, and preserves heading provenance. Content that cannot be confidently classified—including unrecognized headings, mid-description asides, and prose that fits no core kind—stays verbatim in `objective`; the migration does not guess. Dry-run reports, for every affected ticket, which sections move and which content remains in `objective`; apply is separate. Any part it emits uses the common nonblank, lifecycle, and recoverable-persistence gates. If the ticket is already `ready`, the migration follows the normal transition to a pre-`ready` state before splitting and re-enters `ready` afterward so existing planning parts are frozen again.

### Typed references

`ticket.toml` supports a `[[refs]]` table for non-ticket entities. Each entry has `kind`, canonical URN, and optional `note`; the six supported kinds are `spec`, `test_execution`, `log`, `rule`, `file`, and `commit`. Existing `related_specs` values migrate into `[[refs]]` without loss. Ticket-to-ticket dependency structure remains in the edge model.

### Projected reads and legacy tickets

Reads accept named profiles or an explicit part list. The profiles return exactly:

| Profile | Returned content |
|---|---|
| `summary` | Metadata and `objective` |
| `plan` | Metadata, `objective`, `requirements`, `design`, `examples`, `acceptance_criteria`, and refs |
| `review` | Metadata, `acceptance_criteria`, `review`, and `validation` |
| `full` | Everything, including free-form parts |

`--parts a,b,c` returns exactly the requested set. A legacy ticket without `[[parts]]` remains readable: profiles and explicit selection treat `description.md` as its sole `objective` part.

## Boundaries And Failure Cases

- Blank or whitespace-only writes fail before any persistent artifact changes; no success-shaped fallback or partial empty file is allowed.
- Slug collisions, path traversal/unsafe output, pre-existing foreign files, stale migration fingerprints, and unowned backup targets fail explicitly before destructive publication.
- Recovery must distinguish unpublished from durably published manifest generations and must not delete files not recorded as transaction-owned.
- Failed migration and undo preserve later edits and unrelated empty/orphan files.
- A missing, malformed, or conflicting journal is surfaced as an error; recovery does not silently assume success or discard user data.
- `ready` does not imply that absent planning kinds should be synthesized. Frozen content has no force bypass.

## Provider/Consumer Contract

| Consumer | Provider | Consumed criteria |
|---|---|---|
| CLI part writes | Shared `TicketStore::write_part` API | `nonblank-writes`, `ready-lifecycle`, `recoverable-persistence` |
| MCP and HTTP part writes | Shared `TicketStore::write_part` API | `nonblank-writes`, `ready-lifecycle`, `recoverable-persistence`, `transport-consistency` |
| Part listing and projected reads | Manifest and `TicketPart` model | `parts-model`, `slug-paths`, `read-projections`, `legacy-reads` |
| Part-layout migration and undo | Ticket filesystem transaction layer | `slug-paths`, `recoverable-persistence`, `path-migration`, `empty-cleanup` |
| Description-to-parts migration | Shared part-write and transaction APIs | `nonblank-writes`, `description-compatibility`, `description-mode`, `recoverable-persistence` |
| Ticket reference consumers | `[[refs]]` manifest model | `typed-refs` |

## Examples

1. A ticket has two review passes. The manifest retains each opaque stable part ID and its creation order; the files are `parts/review.md` and `parts/review-2.md`. A third pass becomes `parts/review-3.md`. A foreign file already occupying `review-2.md` causes a conflict rather than replacement.
2. A ticket enters `ready` with `objective` present but no `requirements` entry. Only the existing objective is frozen; no requirements file is created. A new planning requirements part is rejected while the ticket remains `ready`, while a validation part or amendment remains permitted.
3. A cleanup dry-run reports a zero-byte, manifest-referenced, frozen `design` part as eligible. It does not report an empty unfrozen review part or an orphan file as eligible. Apply removes only the eligible part; undo restores its original manifest position and zero-byte file if no conflicting later change exists.
4. An interrupted overwrite before durable manifest publication recovers the prior complete part and manifest. An interruption after durable publication finishes the new complete part state. Neither outcome exposes a manifest-referenced partial file.

## Evidence

### Guards

The following validation specs are required gates:

- `val-ticket-parts-api-tests` — `cargo test -p ticket-api` from `workflow-tools/ticket`; covers `parts-model`, `ready-lifecycle`, `nonblank-writes`, `slug-paths`, `recoverable-persistence`, `path-migration`, `empty-cleanup`, `description-compatibility`, `legacy-reads`, `description-mode`, `typed-refs`, and `read-projections`.
- `val-ticket-parts-transport-tests` — `cargo test -p ticket --all-features` from `workflow-tools/ticket`; covers `transport-consistency` and shared write-gate behavior at transport boundaries.
- `val-ticket-parts-format` — `cargo fmt --check -p ticket-api` from `workflow-tools/ticket`; formatting gate.

### Positions

- Part model and stable-ID addressing: `implemented` at [TicketPart](../../../../../ticket/crates/ticket-api/src/model/ticket.rs#L16-L28); stable UUID identity is independent of the deterministic kind-slug path.
- Shared part write gate: `implemented` at [TicketStore::write_part](../../../../../ticket/crates/ticket-api/src/storage/store/parts.rs#L10-L115); blank-write validation, lifecycle checks, and path planning share one API.
- Filesystem persistence and manifest publication: `implemented` at [TicketFs::write_part](../../../../../ticket/crates/ticket-api/src/storage/ticket_fs.rs#L538-L612), [transaction::publish](../../../../../ticket/crates/ticket-api/src/storage/ticket_fs/transaction.rs#L54-L81), and [transaction::recover](../../../../../ticket/crates/ticket-api/src/storage/ticket_fs/transaction.rs#L280-L337); journaled writes stage and back up parts and manifests, with a durable generation commit point and idempotent recovery.
- Plan freeze: `implemented` at [TicketFs::apply_plan_freeze](../../../../../ticket/crates/ticket-api/src/storage/ticket_fs.rs#L717-L755); only existing planning parts are frozen and absent kinds remain absent.
- Part-layout migration tooling: `implemented` at the [planner](../../../../../ticket/crates/ticket-api/src/storage/store/part_layout_migration/planning.rs), [apply executor](../../../../../ticket/crates/ticket-api/src/storage/store/part_layout_migration/execution.rs), [verify/undo operations](../../../../../ticket/crates/ticket-api/src/storage/store/part_layout_migration/verify_undo.rs), [TicketFs adapter](../../../../../ticket/crates/ticket-api/src/storage/ticket_fs/part_layout.rs), and [CLI command](../../../../../ticket/src/cli/commands/migrate_part_layout.rs). The approved live-store application remains pending W4.

### Validation evidence

Validation executions are recorded and read back from the canonical test store:

- Local test-store execution `exec-vt-ticket-parts-api-tests-20261013-w2` passed: `cargo test -p ticket-api`, 242 tests across five suites.
- Local test-store execution `exec-vt-ticket-parts-format-20261013-w2` passed: API and ticket CLI formatting checks completed without a diff.
- Local test-store execution `exec-vt-ticket-parts-transport-tests-20261013-w2` failed: `cargo test -p ticket --all-features` reported 98 passed and 48 failed. The reported failures include test setup with states `open` and `planned` rejected by the current schema; transport compilation passed. The transport guard remains unverified.

The execution records live in the workspace's ignored `.workflow-tools/test` store and were read back by ID.
The current 19 code references validate against the meta-workspace root, and the spec health check reports zero issues.

### W3 validation

- `cargo test -p ticket-api` passed: 250 tests across five suites; execution [exec-vt-ticket-parts-api-tests-20261013-w3](../../../test/executions/exec-vt-ticket-parts-api-tests-20261013-w3.json).
- `cargo fmt --check -p ticket-api` and `cargo fmt --check -p ticket` passed; execution [exec-vt-ticket-parts-format-20261013-w3](../../../test/executions/exec-vt-ticket-parts-format-20261013-w3.json).
- Focused migration API tests passed (8 tests); focused migration CLI and batch-guard tests passed (7 tests total).
- `cargo test -p ticket --all-features` remains blocked: 105 passed and 48 failed. Explicit failures in the CLI fixture tests use schema-invalid `open` or `planned` states, matching the failure class reproduced in W2; the transport guard remains unverified. The run is recorded as [exec-vt-ticket-parts-transport-tests-20261013-w3](../../../test/executions/exec-vt-ticket-parts-transport-tests-20261013-w3.json).
- W3 is blocked on the required full ticket-package test gate. W4 has not started, and no inventoried store has been migrated.

## Scope

In scope: preserve the established structured part kinds and stable IDs; correct blank-write and `ready`-freeze behavior; journaled create/overwrite/manifest/path persistence and idempotent recovery; slug/ordinal path migration; cleanup limited to manifest-referenced zero-byte frozen planning parts in both discovered stores; explicit `description_mode`; lossless description migration; all six typed reference kinds and `related_specs` migration; projected reads and legacy-ticket reads; consistent CLI/MCP/HTTP behavior; focused tests and directly related documentation.

Out of scope: reworking ticket-to-ticket edges or free-form edge kinds; expanding the projection surface beyond profiles and explicit part lists; history compaction; changing part UUID identity, manifest order, timestamps, or supersedence; deleting empty non-planning or unfrozen parts or any orphan files; a force bypass for frozen planning content; implementation, migration execution, or cleanup before the roadmap is formally reviewed and explicitly approved.

## Acceptance Criteria

- `parts-model`: Every part retains stable opaque `id`, `kind`, manifest-relative `path`, `frozen`, `created_at`, and optional `supersedes`; manifest order is display/creation order, core kinds are schema-validated, and free-form kinds round-trip as opaque attachments.
- `ready-lifecycle`: Entering `ready` freezes only existing planning-kind manifest entries and creates no missing entries/files; leaving `ready` clears flags; new planning kinds and frozen-part overwrites are rejected at `ready`, while non-planning and amendment behavior remains available with no force bypass.
- `nonblank-writes`: Ticket creation descriptions, part creation, overwrite, amendment, undo, and implicit legacy objective writes reject empty or whitespace-only content before changing part files, `description.md`, manifest, history, journal, or transaction artifacts; multi-segment description migration prevalidates all segments before its first write.
- `slug-paths`: Every new or migrated explicit part path follows the deterministic kind-slug and manifest-order ordinal rule, has a safe nonempty fallback, and fails on collisions before mutation; stable IDs and manifest order are unchanged.
- `recoverable-persistence`: Create, overwrite, manifest-only, and path-migration operations stage and publish under a durable journal with backups and idempotent recovery. Failure injection at every publication boundary yields the complete old or complete new state, never partial content, an unreferenced failed-create artifact, or an unrecoverable file/manifest split.
- `path-migration`: Dry-run makes no writes and reports proposed paths, collisions, candidates, conflicts, and skipped entries; apply rechecks fingerprints and uses recoverable publication; undo restores only its recorded pre-state and refuses stale/conflicting state without overwriting later edits.
- `empty-cleanup`: Migration removes only manifest-referenced zero-byte frozen planning parts, restoring original manifest position and file on valid undo; empty non-planning parts, unfrozen parts, nonzero files, and all orphans are preserved. Both discovered ticket stores are included.
- `description-compatibility`: Description migration preserves heading provenance, splits `## Review`, `## Status`, `## Validation`, and `## Handoff` into separate typed parts (one `notes` part per matched heading), preserves unclassified content verbatim in `objective`, and uses shared write and recovery gates.
- `legacy-reads`: Tickets without `[[parts]]` remain readable through profiles and explicit selection, with `description.md` as the sole `objective`.
- `description-mode`: `update_ticket` with a description and no explicit `replace` or `append` mode is rejected; both explicit modes retain their defined behavior.
- `typed-refs`: `[[refs]]` round-trips `spec`, `test_execution`, `log`, `rule`, `file`, and `commit`, and existing `related_specs` migrate without loss; ticket-to-ticket edges remain separate.
- `read-projections`: Each named profile returns exactly its specified metadata/parts/refs; explicit `--parts` returns exactly the requested set, including supported free-form parts under `full`.
- `transport-consistency`: CLI, MCP, HTTP, migration, amendment, undo, and legacy entry points route through or faithfully enforce the shared write, lifecycle, and persistence contract; tests prove blank rejection and error propagation without partial filesystem changes.
