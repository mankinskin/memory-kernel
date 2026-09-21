<!-- aligned-structure:v1 -->

# Summary

The built-in `tracker-improvement` ticket schema includes an optional `effort` field used to capture the estimated token budget required to complete the ticket.

## Behavior Story

The built-in `tracker-improvement` ticket schema includes an optional `effort` field used to capture the estimated token budget required to complete the ticket.

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
The built-in `tracker-improvement` ticket schema includes an optional `effort` field used to capture the estimated token budget required to complete the ticket.

# Requirements
- `effort` is schema-defined so create/update flows accept it without ad-hoc field handling.
- `effort` remains optional for backward compatibility with existing tickets.
- Ticket viewers display and allow editing the field anywhere schema-backed string fields are surfaced.
- Ticket listing and recommendation surfaces treat `effort` as sortable metadata.
- `ticket next` and board recommendations prefer lower-effort tickets by default when choosing between otherwise-eligible work.
- List-oriented CLI/HTTP/MCP ticket summaries surface parsed `effort` values and default to ascending effort ordering.

# Implementation Notes
- Added `effort` to the built-in Rust schema and the TOML schema file for `tracker-improvement`.
- Exposed `effort` in the ticket detail viewer editable string field list.
- Added shared `effort` parsing in workflow ranking and list ordering.
- Surfaced `effort` in CLI, HTTP, and MCP next/list payloads plus board recommendation output.

# Validation
- `cargo test --manifest-path memory-api/crates/ticket-api/Cargo.toml sort_candidates_prefers_lower_effort_before_newer_tickets`
- `cargo test --manifest-path memory-api/crates/ticket-api/Cargo.toml parse_effort_accepts_numeric_token_budgets`
- `cargo test --manifest-path memory-api/tools/cli/ticket-cli/Cargo.toml integration_board_cli -- --nocapture`
- `cargo test --manifest-path memory-api/tools/http/ticket-http/Cargo.toml listing -- --nocapture`
- `cargo test --manifest-path memory-api/tools/mcp/ticket-mcp/Cargo.toml next_tickets -- --nocapture`
