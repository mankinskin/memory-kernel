<!-- aligned-structure:v2 -->

# Summary

Add a durable logical session workspace that carries pinned entities, an evolving execution roadmap, run lineage, and structured handoff state across agent runs.

## Motivation ("why")

Captured transcripts preserve evidence but do not give a resumed agent an authoritative roadmap. A session needs durable attention state and a mutable workflow that survives handoff without replaying the transcript or copying ticket state into a second source of truth.

## Dependent expectation

If this spec is implemented, dependents can rely on one durable `workspace_session_id` spanning multiple capture `run_id` values; a persisted workflow containing ticket-backed, spec-backed, validation, and generic descriptive nodes; live resolution of ticket and spec state; discoverable workflow enums and lifecycle; structured handoff records; terminal and Mermaid rendering; and explicit graph-gated finish.

## Guards

- `val-session-workflow-persistence`.
- `val-session-workflow-rendering`.
- `val-session-handoff-continuity`.
- `val-session-workflow-finish`.
- `val-session-cli-suite`.
- `val-session-mcp-suite`.

## Positions

- Runtime session capture and transcript store: `implemented` at `memory-api/crates/session-api/src/store.rs`.
- Runtime session model: `partial` at `memory-api/crates/session-api/src/model.rs`.
- Workflow persistence and mutation: `implemented` at `memory-api/crates/session-api/src/model.rs` and `memory-api/crates/session-api/src/store.rs`; validated by `exec-val-session-workflow-persistence-20260714`.
- Node-kind taxonomy: `implemented` at `memory-api/crates/session-api/src/model/workflow.rs`. Behavioral kinds are a closed validated set (`ticket`, `validation`, `spec`); descriptive nuance lives on an open free-text `category` field and the generic `task` kind; deprecated `action`/`decision`/`checkpoint` deserialize as `task`.
- Terminal and Mermaid rendering: `implemented` at `memory-api/crates/session-api/src/store.rs`; validated by `exec-val-session-workflow-rendering-20260714`.
- Structured handoff/resume and finish: `implemented` at `memory-api/crates/session-api/src/store.rs`; validated by `exec-val-session-handoff-continuity-20260714` and the latest `val-session-workflow-finish` execution. Finish is authoritative: required validation outcomes come only from test-api executions, ticket-backed and spec-backed nodes require live terminal state, and unavailable resolution fails closed. Finished workspaces reject mutations and new lineage; ordinary idempotent init is read-only. Runtime changes and finish serialize under an OS-held exclusive file lock with no age-only reclamation, and releasing an owner does not unlink the stable lock file used by successors.
- Durable JSON writes: `implemented` at `memory-api/crates/session-api/src/store_helpers.rs`. The temp file is synced before `std::fs::rename`; replacement errors preserve the previous destination. Unix parent-directory sync errors are propagated. Windows replace-existing atomicity and directory-entry power-loss durability are not guaranteed by this contract beyond tested failure preservation.
- CLI/MCP surfaces: `implemented` at `memory-api/tools/cli/session-cli/src/lib.rs` and `memory-api/tools/mcp/session-mcp/src/server.rs`; workflow mutation schemas advertise legal enum values, rejections enumerate the allowed set, the lifecycle and enums are discoverable via `session_capabilities`, and `workspace_session_id` is returned top-line and echoed by every workflow/runtime result; validated by `exec-session-cli-suite-20260714` and `exec-session-mcp-suite-20260714`.

## Governing-rule requirement

This contract is governed by `.agents/instructions/spec-system.instructions.md` and its aligned-structure v2 requirement.

# Contract

## Identity and lineage

- `workspace_session_id` identifies the durable workspace and is reused across handoffs.
- Every execution receives a distinct `run_id` and optional `predecessor_run_id`.

## Workflow model

- Node `kind` separates two orthogonal axes. Behavioral kinds are a closed, validated set that finish gating branches on and that carry required side-data: `ticket` (`ticket_urn`, gated on live ticket terminal state), `spec` (`spec_urn`, gated on live spec terminal state, symmetric to `ticket`), and `validation` (`validation_spec_id`, gated on authoritative execution outcome).
- Descriptive classification lives on an open free-text `category` field and the node `title` that no gating logic branches on. The generic non-gating `task` kind is the descriptive bucket; the deprecated `action`, `decision`, and `checkpoint` kinds deserialize as `task` for back-compat with persisted contexts.
- Ticket nodes persist authoritative ticket URNs plus cached display metadata; current state resolves live. Spec nodes persist authoritative spec URNs; a required spec node fails finish closed when live spec state is unavailable and completes only on a terminal spec state (`verified`, `deprecated`, or `cancelled`).
- Nodes have stable IDs, required/optional classification, status, and timestamps.
- Directed edges express dependency or execution order and may be added during execution.
- Promotion to a ticket preserves the session node identity and records the resulting ticket URN.

## Discoverability and handle

- Workflow mutation tool schemas advertise their legal enum values (`kind`, `requirement`, edge `kind`, `status`); invalid values are rejected with the allowed set enumerated. Advertised values match the `session-api` enums exactly.
- The session lifecycle flow (`runtime_init` → `pin`/`view` → `workflow_*` → `render_*` → `handoff`/`finish`) and its enums are discoverable from a self-describing capability catalog (`session_capabilities`) without source-diving.
- `workspace_session_id` — the handle required by every workflow/runtime call — is returned as a prominent top-line field by init/resume and echoed by every workflow/runtime tool result, so it never has to be re-fetched from a spilled resource file.

## Persistence and rendering

- Workflow state is stored separately from transcript capture and flushed per mutation.
- A live runtime lock cannot be stolen solely because it is old, and lock release cannot remove a successor's lock instance.
- JSON writes sync the temporary file before rename. Rename failure preserves the prior destination; Unix parent-directory sync is checked. No stronger Windows replace-existing or power-loss guarantee is implied.
- Terminal and Mermaid renders are deterministic, equivalent, safely escaped, and read-only.

## Handoff and finish

- Handoff persists before rendering and includes workspace ID, outgoing run, handoff ID, pins, roadmap state, blockers, validation state, and exact resume command.
- Finish is explicit and idempotent; required nodes and validation must pass.
- Finish and all mutation/init/resume paths serialize under the same runtime lock.
- After finish, ordinary init returns persisted state without rewriting runtime files; init that creates a run, resume, and all other mutations reject.
- Optional nodes may remain incomplete only when explicitly deferred with a reason.

# Non-goals

- Copying ticket lifecycle state into the session store.
- Requiring feedback-api for context or workflow persistence.
- Semantic auto-pinning or replacing the ticket graph.
- Claiming untested Windows replacement or crash/power-loss durability semantics.

# Acceptance Criteria

1. A workspace initializes, mutates, reloads, hands off, and resumes under the same workspace ID with distinct linked runs.
2. Ticket-backed, spec-backed, validation, and generic descriptive nodes can be added, updated, linked, and promoted without duplicate identity.
3. Ticket and spec state resolve live; unavailable references produce diagnostics without corruption and fail required-node finish closed.
4. Terminal and Mermaid renders deterministically represent the same graph.
5. Handoff persistence precedes rendering and always provides exact resume flow.
6. Finish enforces required work and validation and records terminal success.
7. Feedback emission is optional and non-blocking.
8. Deterministic regressions prove aged live locks remain exclusive, release is ownership-safe, finished init is byte-stable, and finish excludes mutation/init/resume interleavings.
9. Behavioral node kinds are a closed validated set (`ticket`, `validation`, `spec`); descriptive classification is an open field no gating logic branches on; deprecated kinds deserialize for back-compat.
10. Workflow mutation schemas advertise legal enum values, invalid values return the allowed set, the lifecycle and enums are discoverable from a capability catalog, and `workspace_session_id` is exposed top-line and echoed.

# Traceability

- Parent spec: `8c880efc-7083-4e1d-bf06-96b8254be913`.
- Runtime context spec: `709f067a-21b6-41b6-8879-3cacef4bacaf`.
- Handoff prompt spec: `9e04ff58-9160-4766-b307-74c0fb32a92c`.
- Workflow persistence ticket: `70cd7056-c342-4433-ad60-5bc798f61aa6`.
- Rendering ticket: `cc4b0289-b6fd-412f-a97a-497f05f572f4`.
- Core handoff ticket: `0647a212-9d2e-4943-9627-f854ce3f14c4`.
- Transport ticket: `6b2dc497-188c-44f5-9106-bf35deecb7a1`.
- Prompt update ticket: `9577b114-ec11-431b-8740-c488bef05fc9`.
- Remediation ticket: `6b1edff1-bc32-40c7-b3a9-fb1292b0213f`.
- Node-kind taxonomy ticket: `203248cb-0694-481b-a634-ba7d70962750`.
- Workflow enum schema advertisement ticket: `7f1ed44f-73f3-40c9-9647-d899c64ec507`.
- Invalid-enum recovery contract ticket: `8bb97b73-9dbc-43ee-9939-46b3ddf2612f`.
- Capability catalog ticket: `5ad77aba-c7f7-4058-854e-dd0412746c7c`.
- Inline session-handle ticket: `3eaceaae-254e-4a9f-ab19-c1eed2080931`.
