<!-- aligned-structure:v1 -->

# Summary

Ticket CLI, ticket HTTP, and ticket MCP currently return different workflow and health answers for the same store because parity-critical domain behavior still lives in their interface crates instead of one ticket-api-owned minimal core.

## Behavior Story

Ticket CLI, ticket HTTP, and ticket MCP currently return different workflow and health answers for the same store because parity-critical domain behavior still lives in their interface crates instead of one ticket-api-owned minimal core.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Summary

Ticket CLI, ticket HTTP, and ticket MCP currently return different workflow and health answers for the same store because parity-critical domain behavior still lives in their interface crates instead of one ticket-api-owned minimal core.

This spec defines the planning baseline for consolidating that behavior into ticket-api, making the adapter boundaries explicit, and validating parity across CLI, HTTP, and MCP with one reproducible larger-integration routine.

# Goals

- Define one ticket-api-owned minimal core for selector-driven workflow discovery, health evaluation, and normalized result shaping.
- Reduce ticket-cli and ticket-http to transport responsibilities plus explicitly documented compatibility shims.
- Close the places where ticket-mcp is too thin to expose the real contract cleanly.
- Add a reusable parity routine that proves CLI, HTTP, and MCP agree on equivalent workflow and health behavior against the same seeded fixture store.
- Make review reject undocumented drift between surfaces.

# Non-Goals

- Consolidated ticket detail/context read parity. That remains adjacent follow-up work under [61cb6557 Add consolidated ticket detail/context read surface](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.ticket/tickets/61cb6557-e559-4eae-8e59-ea0d520a3bee/ticket.toml).
- Viewer UX changes.
- Reworking ticket lifecycle semantics unrelated to workflow and health parity.
- Replacing transport-specific envelopes where those differences are intentional and documented.

# Ticket Plan

## Upstream prerequisites

- [790df512 Specify scoped selector contract for board and next](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/790df512-d8a9-42bd-b3d6-6e2b4d5eda9c/ticket.toml)
- [68a08b34 Scope-aware board and next for multi-root workspaces](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.workflow-tools/ticket/tickets/68a08b34-000b-4585-8354-4b1a26a15f4b/ticket.toml)
- [10cf2a19 Expose workflow trees and actionable ordering metadata](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/10cf2a19-356c-4e69-b0f3-b930d68dc0ce/ticket.toml) is already done and serves as the current HTTP workflow baseline.

## Main execution track

- Tracker: [cf4246c3 Track workflow and health surface convergence](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/cf4246c3-6539-4f1c-a876-6d34073db7b3/ticket.toml)
- Shared selector work: [0e375356 Implement scoped selectors for board and next](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/0e375356-b74e-48c4-8f1d-77cd28e055bc/ticket.toml)
- Minimal core plus adapter boundary cleanup: [c031aeb0 Define minimal workflow and health core plus adapter responsibilities](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/c031aeb0-f374-4d57-9d46-2463dfa8571d/ticket.toml)
- Larger-integration parity routine: [6484d4b7 Build larger-integration parity routine for workflow and health surfaces](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/6484d4b7-e24b-4c13-999c-d0b00928d97c/ticket.toml)
- Supporting reference for authoritative workspace and path metadata: [07836f41 Make get/search/list workspace-aware across nested roots](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.ticket/tickets/07836f41-7fa5-4e41-8411-1c7cf8aeee75/ticket.toml)

# Implementation Order

1. Phase 0: finish [790df512 Specify scoped selector contract for board and next](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/790df512-d8a9-42bd-b3d6-6e2b4d5eda9c/ticket.toml) and [68a08b34 Scope-aware board and next for multi-root workspaces](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.workflow-tools/ticket/tickets/68a08b34-000b-4585-8354-4b1a26a15f4b/ticket.toml). These define the selector vocabulary, scope metadata, and compatibility behavior used by everything downstream.
2. Phase 1: implement [0e375356 Implement scoped selectors for board and next](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/0e375356-b74e-48c4-8f1d-77cd28e055bc/ticket.toml) across CLI, MCP, and HTTP. Exit when the three surfaces accept the agreed selector inputs and expose consistent selected-scope metadata.
3. Phase 2: complete [c031aeb0 Define minimal workflow and health core plus adapter responsibilities](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/c031aeb0-f374-4d57-9d46-2463dfa8571d/ticket.toml). Exit when parity-critical workflow and health logic lives in ticket-api and the responsibility matrix is explicit.
4. Phase 3: complete [6484d4b7 Build larger-integration parity routine for workflow and health surfaces](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/6484d4b7-e24b-4c13-999c-d0b00928d97c/ticket.toml). Exit when the automated and manual parity checks run against the same seeded fixture store.
5. Phase 4: move the tracker through review and closeout only after all prerequisite and child tickets are done and the evidence has been attached.

# Ownership Boundary

## ticket-api owns

- Selector-driven workflow evaluation built on the shared selector contract.
- Health finding generation and normalization.
- Normalized workflow result shaping and shared metadata assembly needed for parity-critical answers.
- Focused fixture-based tests for the minimal core contract.

## ticket-cli owns

- Argument parsing, help text, stdout and stderr formatting, exit semantics, and local bootstrap behavior.

## ticket-http owns

- Routing, query decoding, handler wiring, and HTTP status or envelope mapping.

## ticket-mcp owns

- MCP tool registration, schema aliases, envelope shaping, and session or store wiring.
- Full exposure of the selector and health contract rather than partial tool shapes that force clients to reconstruct behavior externally.

# Review Workflow

1. Contract review gate: move [790df512 Specify scoped selector contract for board and next](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/790df512-d8a9-42bd-b3d6-6e2b4d5eda9c/ticket.toml) and [68a08b34 Scope-aware board and next for multi-root workspaces](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.workflow-tools/ticket/tickets/68a08b34-000b-4585-8354-4b1a26a15f4b/ticket.toml) through `in-review` first. Their review packet should freeze selector names, scope metadata, compatibility behavior, and the regression matrix used downstream.
2. Selector implementation gate: review [0e375356 Implement scoped selectors for board and next](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/0e375356-b74e-48c4-8f1d-77cd28e055bc/ticket.toml) only after the phase-0 tickets are done. This review must compare CLI, MCP, and HTTP selector inputs and selected-scope metadata side by side.
3. Core-boundary gate: review [c031aeb0 Define minimal workflow and health core plus adapter responsibilities](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/c031aeb0-f374-4d57-9d46-2463dfa8571d/ticket.toml) after the selector surface is stable. This review must include the responsibility matrix and explicitly call out any remaining adapter-local behavior.
4. Integration gate: review [6484d4b7 Build larger-integration parity routine for workflow and health surfaces](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/6484d4b7-e24b-4c13-999c-d0b00928d97c/ticket.toml) last. This review must include the automated parity run plus the manual fixture-based checklist.
5. Tracker closeout gate: move [cf4246c3 Track workflow and health surface convergence](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/cf4246c3-6539-4f1c-a876-6d34073db7b3/ticket.toml) to `in-review` only after all prerequisite and child tickets are done and the spec links the exact ticket files, docs, and validation evidence. Close it only after the final reviewer confirms there is no undocumented behavior drift left in workflow or health surfaces.

# Validation Strategy

## Automated

- Add focused ticket-api tests that assert normalized workflow and health results from a shared fixture store.
- Add CLI, HTTP, and MCP parity tests that compare equivalent discovery and health flows against that same fixture.
- Ignore only documented transport-envelope or human-formatting differences in the parity assertions.
- Record exact test commands, harness entrypoints, or targets in review notes before moving a ticket to `in-review`.

## Manual Review Checklist

1. Seed the shared fixture store with dependency edges, board state, and enough workspace metadata to expose scope and ownership differences.
2. Run the CLI discovery and health flows against that fixture, including `ticket next`, `ticket board show`, and the relevant health command.
3. Run the HTTP workflow and health flows against the same fixture and capture normalized candidate ids, ordering, warnings or exclusions, findings, and metadata.
4. Run the MCP discovery and health flows against the same fixture and capture the same normalized fields.
5. Compare only intentional transport-local differences separately: CLI human formatting, HTTP status or envelope mapping, and MCP tool envelopes or alias names.
6. Re-run the automated parity suite and attach the passing commands or targets before review sign-off.

# Review Gate

Review should reject any implementation that leaves selector semantics, health findings, workflow result shaping, or required MCP contract fields duplicated or missing across interface crates without a documented transport-only reason.

# Implementation Notes

The expected implementation center of gravity is the existing ticket-api workflow layer and the interface-specific workflow and health entrypoints in ticket-cli, ticket-http, and ticket-mcp. The work should move those interfaces toward thin adapters rather than adding another shared wrapper outside ticket-api.
