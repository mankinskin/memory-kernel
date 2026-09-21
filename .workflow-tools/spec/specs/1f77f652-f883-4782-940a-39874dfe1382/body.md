## Motivation

`update_ticket` previously always overwrote `description.md` on any description write, with no way to append and no guarantee the prior text was recoverable. This spec documents the implemented contract: an explicit replace/append mode, and unconditional history capture of the pre-update description so an overwrite is never silently destructive.

## Dependent expectation

If this spec is implemented, dependents (ticket-mcp, ticket-cli, and any future transport) can rely on:
- `description_mode` defaults to `Replace` (byte-for-byte overwrite of `description.md`) when omitted.
- `Append` concatenates as `{existing}\n{new}` when an existing non-empty description is present, otherwise behaves like `Replace`.
- Whenever a `description` value is supplied (either mode), the **pre-update** description is written to the ticket's history revision under `DESCRIPTION_HISTORY_KEY` (`"__previous_description__"`), unconditionally — not just on overwrite.
- The captured previous description is recoverable: `TicketStore::apply_revert` reads `DESCRIPTION_HISTORY_KEY` from the history revision and restores `description.md`, enabling `--undo` / `undo: true` to reverse a destructive replace.

## Provided Surface Contracts

### Core (`memory-api/crates/ticket-api/src/storage/store.rs`)
- `DescriptionUpdateMode` enum, `Replace` (default) / `Append` — [store.rs L79-86](memory-api/crates/ticket-api/src/storage/store.rs#L79-L86)
- `TicketStore::apply_manifest_update(&self, ticket_path, patch, new_state, transition_path, description: Option<&str>, description_mode: DescriptionUpdateMode)` computes `previous_description` and the final text per mode — [store.rs L599-641](memory-api/crates/ticket-api/src/storage/store.rs#L599-L641), replace/append branch at [store.rs L627-636](memory-api/crates/ticket-api/src/storage/store.rs#L627-L636)
- Unconditional history capture of the pre-update description whenever `description.is_some()`, regardless of mode — [store.rs L511-523](memory-api/crates/ticket-api/src/storage/store.rs#L511-L523)

### Recovery (`memory-api/crates/ticket-api/src/storage/store/lifecycle.rs`)
- `TicketStore::apply_revert` reads `DESCRIPTION_HISTORY_KEY` from the supplied history-revision fields and restores it as the live description — [lifecycle.rs L104-121](memory-api/crates/ticket-api/src/storage/store/lifecycle.rs#L104-L121)

### Transport surfaces
- MCP: `UpdateTicketInput.description_mode: Option<String>` (`"replace"` | `"append"`, default `"replace"`) — [ticket-mcp types.rs L165-168](memory-api/tools/mcp/ticket-mcp/src/server/types.rs#L165-L168); parsed/validated (rejects unknown values with `invalid_params`) — [ticket-mcp mutations.rs L39-50](memory-api/tools/mcp/ticket-mcp/src/server/mutations.rs#L39-L50)
- CLI: `--description-mode` (default `"replace"`) — [ticket-cli operations.rs L253-256](memory-api/tools/cli/ticket-cli/src/cli/args/operations.rs#L253-L256); parsed and wired into `store.update_with_options` — [ticket-cli crud.rs L188-204](memory-api/tools/cli/ticket-cli/src/cli/commands/crud.rs#L188-L204)

## Non-Goals

- Does not change the shape or format of `description.md` beyond simple concatenation with `\n`.
- Does not add a third merge/diff mode; only `Replace` and `Append` are supported.

## Required Validation

- `update_with_replace_mode_overwrites_description` — [update_regression_tests.rs L446](memory-api/crates/ticket-api/src/storage/tests/update_regression_tests.rs#L446)
- `update_with_append_mode_concatenates_description` — [update_regression_tests.rs L481](memory-api/crates/ticket-api/src/storage/tests/update_regression_tests.rs#L481)
- `update_captures_previous_description_in_history_regardless_of_mode` — [update_regression_tests.rs L520](memory-api/crates/ticket-api/src/storage/tests/update_regression_tests.rs#L520)
- `undo_restores_previous_description` — [update_regression_tests.rs L559-602](memory-api/crates/ticket-api/src/storage/tests/update_regression_tests.rs#L559-L602)
- Command: `rtk cargo test -p ticket-api` → 135 passed, 0 failed
- Landed in memory-api submodule commit `143ea13`

## Related Implementation Tickets

- [bf62e2f9 [ticket-api] Add explicit replace/append mode to description update and always capture pre-overwrite description in history](c:/Users/linus/git/graph_app/workflow-tools/.workflow-tools/ticket/tickets/bf62e2f9-7bdb-471d-a8c3-e160fe87e610) — state: in-review, blocked on this spec for the in-review→done traceability gate.

## Background Knowledge References

- `DESCRIPTION_HISTORY_KEY` constant is defined in `memory-api/crates/ticket-api/src/storage/store.rs` (near L71) and shared between the update path and `apply_revert`.
