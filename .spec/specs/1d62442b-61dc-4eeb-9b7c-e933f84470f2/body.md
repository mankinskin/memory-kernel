<!-- aligned-structure:v1 -->

# Summary

Unify ticket state transition handling so `update_ticket` and `close_ticket` share one schema-validated transition-path implementation, remove reliance on caller-supplied `from_state`, and support explicit intermediate `transition_states` in update requests.

## Behavior Story

Unify ticket state transition handling so `update_ticket` and `close_ticket` share one schema-validated transition-path implementation, remove reliance on caller-supplied `from_state`, and support explicit intermediate `transition_states` in update requests.

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

Unify ticket state transition handling so `update_ticket` and `close_ticket` share one schema-validated transition-path implementation, remove reliance on caller-supplied `from_state`, and support explicit intermediate `transition_states` in update requests.

## Problem

`ticket update` currently validates only a single direct edge and accepts `from_state`, which duplicates the authoritative current state already stored in the ticket. That makes legitimate multi-step progressions fail even when every intermediate state is valid. `close_ticket` already walks an intermediate path separately, so the system duplicates transition logic instead of sharing one workflow path implementation.

## Scope

This spec covers:

- a shared ticket-api transition-path helper used by both `update` and `close`
- `update_ticket` support for `transition_states` intermediate steps before `to_state`
- removal or deprecation of `from_state` across CLI, HTTP, and MCP ticket update surfaces
- preservation of `required_states` and terminal-state validation

## Intended Behavior

- The store determines the current ticket state from the persisted ticket entry.
- `update_ticket` can accept `transition_states` plus a final `to_state`, and the full path is schema-validated.
- `close_ticket` uses the same transition walker rather than a separate implementation.
- Required-state and terminal-state rules are enforced on the final path outcome, not bypassed.

## Assumptions To Prove

- The transition path can be derived and validated from stored current state plus caller-requested intermediates.
- `from_state` is unnecessary for correct schema validation once the store state is authoritative.
- Existing single-step update callers remain compatible when `transition_states` is omitted.
- All ticket transports can surface the new request model without breaking response compatibility.

## Test Strategy

1. Add failing tests for multi-step update transitions and shared path handling.
2. Refactor ticket-api to share the transition path logic between update and close.
3. Update CLI, HTTP, and MCP tests for the new `transition_states` request shape and sparse update behavior.

## Acceptance Criteria

- `ticket update` can progress through valid consecutive intermediate states in one call.
- `close_ticket` and `update_ticket` share the same transition-path logic.
- `from_state` is removed or ignored in favor of store-derived current state.
- Focused ticket-api, ticket-cli, ticket-http, and ticket-mcp tests pass.

## Output Format Preferences

For token-efficient agent workflows, prefer `--toon` output over verbose `--json`:

- **Compact by default**: CLI commands should produce compact output by default; use `--verbose` or `--json` only when needed.
- **TOON vs JSON**: Prefer `--toon` for machine-readable output between tools; `--json` is verbose (40–80% token overhead) and should be used only when piping to external tools like `jq`.
- **Transport consistency**: HTTP and MCP endpoints should support both TOON and JSON, with TOON as the recommended default for agent-to-agent communication.
- **Documentation guidance**: Update all spec and rule examples to show `--toon` usage first, with `--json` as a fallback for debugging or external integration.
