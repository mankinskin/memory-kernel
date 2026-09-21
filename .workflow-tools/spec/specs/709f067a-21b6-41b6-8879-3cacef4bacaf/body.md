<!-- aligned-structure:v2 -->

# Summary

Add the durable runtime context foundation for pinned entities and workspace/run identity without disturbing capture/archive storage.

## Motivation ("why")

A transcript does not preserve active attention state in a compact, mutable form. Agents need to resume the same logical workspace, retain selected entities, and start a distinct linked run without replaying prior turns.

## Dependent expectation

If this spec is implemented, dependents can rely on idempotent workspace initialization/resume, durable pinned entity URNs, distinct linked run IDs, immediate file-backed persistence, headers-only views, and optional non-blocking feedback emission. Once a workspace is finished, ordinary init is a byte-stable read and new run lineage is rejected.

## Guards

- `val-session-bootstrap-schema-validation`.
- `val-session-init-idempotency`.
- `val-session-context-capture-isolation`.
- `val-session-run-lineage`.
- `val-session-workflow-finish`.

## Positions

- Existing capture models: `implemented` at `memory-api/crates/session-api/src/model.rs`.
- Existing capture persistence: `implemented` at `memory-api/crates/session-api/src/store.rs`.
- Runtime context model and operations: `implemented` at `memory-api/crates/session-api/src/model.rs` and `memory-api/crates/session-api/src/store.rs`; validated by `exec-val-session-init-idempotency-20260714`, `exec-val-session-run-lineage-20260714`, `exec-val-session-context-capture-isolation-20260714`, and the latest `val-session-workflow-finish` execution.

## Governing-rule requirement

This contract is governed by `.agents/instructions/spec/spec-system.instructions.md` and its aligned-structure v2 requirement.

# Contract

- The Copilot `session_id` UUID identifies the durable session.
- Every agent execution has a distinct `run_id` and optional `predecessor_run_id`.
- Initialization is load-or-create and never clobbers pins or lineage.
- Finished-session ordinary init returns the persisted session manifest without updating timestamps or rewriting `session.json`; force-new-run init and resume reject.
- Init, resume, finish, and runtime mutations serialize under one OS-held exclusive workspace lock; lock age alone never permits reclamation.
- Pinned tickets, specs, and rules are cross-store URNs with relation/reason metadata.
- Mutations flush before returning within the platform guarantees declared by the workflow spec.
- Read/view returns short headers and metadata, never full entity bodies.
- Duplicate pin and missing unpin are idempotent no-ops.
- Feedback usage emission is optional through an injected sink; sink absence or failure cannot corrupt or reject a successful pin.
- Existing capture manifests and transcripts remain byte-identical when runtime state mutates, and runtime state remains byte-identical when capture persists.

# Non-goals

- Workflow graph persistence, rendering, handoff, and finish, owned by `c677182e-90da-4ac3-8b94-9e2e97c825cf`.
- Cascade auto-pinning.
- CLI/MCP transport.
- Full-body entity resolution.

# Acceptance Criteria

1. Fresh initialization creates a durable session manifest with session ID, schema version, timestamps, empty pins, and initial run lineage.
2. Resume preserves pins and adds a distinct linked run without reusing the outgoing run ID before finish; resume rejects after finish.
3. Plain init after finish is read-only and leaves the session manifest byte-identical.
4. Pin/unpin is idempotent and immediately persistent.
5. Headers-only view never includes full bodies.
6. Optional feedback sink receives successful pin usage when configured and cannot block persistence.
7. Capture/runtime-state byte-isolation regressions pass.
8. Focused `cargo test -p session-api` coverage validates the contract.

# Traceability

- Parent spec: `8c880efc-7083-4e1d-bf06-96b8254be913`.
- Implementation ticket: `412964a3-e1c3-47da-94ad-268ff20441c0`.
- Remediation ticket: `6b1edff1-bc32-40c7-b3a9-fb1292b0213f`.
- Downstream workflow ticket: `70cd7056-c342-4433-ad60-5bc798f61aa6`.
- Downstream transport ticket: `6b2dc497-188c-44f5-9106-bf35deecb7a1`.
