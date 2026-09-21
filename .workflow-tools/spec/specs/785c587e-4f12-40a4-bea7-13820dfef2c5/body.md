# Entity Kernel Events and Transactional Hooks

## Objective
Define a domain-neutral event envelope and transactional hook lifecycle for entity mutations.

## Contract
Every mutation emits a typed event envelope containing domain, entity type, entity id, operation, correlation id, pre-state and post-state metadata, and schema versions. Hooks execute in deterministic registration order. Every hook may veto; a veto or hook crash aborts the mutation and rolls back to the pre-mutation state. Observers and veto-capable hooks are explicit roles, but both remain part of the same ordered lifecycle.

Domain adapters may translate the generic envelope into existing ticket StoreHook or session CopilotHookEvent consumers without moving domain-specific behavior into the kernel.

## Acceptance Criteria
- Event envelopes identify domain, entity type, entity, operation, schema versions, and correlation.
- Hook order is deterministic and observable.
- Any veto or crash leaves no partial entity/index/history mutation.
- Existing ticket and session hook consumers can be adapted without business-schema changes.
- Rollback failures produce actionable diagnostics with the original mutation context.

## Non-goals
No new domain-specific hook behavior, no immediate migration of every existing consumer, and no business field/state changes.

## Evidence
- `transcripts/15-09-2026_entity-kernel-all-domains/04-events-hooks.md`
- `workflow-tools/ticket/crates/ticket-api/src/storage/store.rs`
- `workflow-tools/session/crates/session-api/src/hook.rs`
- `workflow-tools/memory-kernel/src/storage/watcher.rs`
