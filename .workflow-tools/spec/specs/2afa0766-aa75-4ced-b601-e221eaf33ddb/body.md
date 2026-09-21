# Entity Kernel Migration Orchestration

## Objective
Define one safe, resumable migration protocol for all domain adapters.

## Contract
The protocol is dry-run -> apply -> journal -> resume -> rollback. Dry-run is side-effect free and produces the only accepted apply plan. Apply stages changes, publishes atomically, and persists a durable journal. Resume continues from the last durable phase. Rollback removes only artifacts created or changed by the migration and never deletes unrelated legacy source records.

Only active entity types are maintained by a domain migration. Inactive types with retained records are reported as present, untouched, inactive. Cross-workspace references remain external URNs and may include an optional entity-type schema-version constraint; violated constraints are dry-run findings.

## Acceptance Criteria
- Dry-run produces a deterministic report without live writes.
- Apply requires the dry-run plan and persists resumable phases.
- Interrupted apply resumes without duplicate or partial canonical records.
- Rollback is journal-bounded and preserves unrelated legacy data.
- Active/inactive behavior and external URN version constraints are reported.
- Ticket and test migration compatibility suites remain green.

## Non-goals
No rewrite of immutable history logs, no literal code-sharing refactor of existing adapters, and no business-schema migration in this specification pass.

## Evidence
- `transcripts/15-09-2026_entity-kernel-all-domains/03-migration-orchestration.md`
- `workflow-tools/ticket/crates/ticket-api/src/storage/store/migration.rs`
- `workflow-tools/test/crates/test-api/src/migration.rs`
- `workflow-tools/memory-kernel/src/cross_store_edges.rs`
