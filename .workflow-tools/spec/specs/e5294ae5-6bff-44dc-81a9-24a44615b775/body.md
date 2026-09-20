<!-- aligned-structure:v2 -->

## Motivation

Workflow domains expose CLI, MCP, and HTTP binary targets. `transport-harness`
is the canonical, transport-neutral crate that prevents every workflow domain
from reimplementing argument parsing, server startup, HTTP error mapping,
tracing setup, and output mechanics. This spec is the authoritative contract for
the crate; consuming repositories (including context-engine) reference it and
must not duplicate these normative requirements.

## Responsibilities

The harness owns the shared, transport-neutral mechanics:

- Shared `Output` representation (`Text` / `Json`) and `write_output`, which
  emits exactly one trailing newline.
- Normalized `HarnessError` at the transport boundary: `Domain`, `Arguments`,
  `Serialization`, `Io`, and transport-tagged `Transport { transport, message }`.
- Tracing initialization to stderr via an env-filtered subscriber.
- Runtime setup (current-thread Tokio runtime) for the MCP and HTTP transports.
- CLI parsing and dispatch mechanics (`cli::run`, `cli::run_from`).
- MCP stdio server startup (`mcp::run`) for a domain-owned `ServerHandler`.
- HTTP listener startup (`http::run`) plus a stable JSON error envelope
  (`http::HttpError`) with an explicit `StatusCode` -> response mapping.

## Non-goals

- Domain command schemas, MCP tools, and HTTP routes are NOT standardized by
  this crate. Domains keep command dispatch, MCP tool/handler registration, and
  HTTP router registration.
- Frontend viewers and VS Code extensions remain outside the harness.
- The harness does not own domain error types; it wraps them via
  `HarnessError::domain` without coupling to their concrete type.

## Features

`default = []`. CLI, MCP, and HTTP are independently selectable:

- `cli = ["dep:clap"]`
- `mcp = ["dep:rmcp", "dep:tokio"]`
- `http = ["dep:axum", "dep:tokio"]`

A slim library builds with no transport dependencies; each transport pulls only
its own dependencies. This slimness (`default = []`) is a normative guard.

## Public API boundaries

- `Transport` enum (`Cli` / `Mcp` / `Http`) with `Display`.
- `Output`, `Output::json`, `write_output`.
- `HarnessError` and `HarnessError::domain`.
- `cli::run`, `cli::run_from`, `cli::startup_error`, re-exported `clap`.
- `mcp::run`, re-exported `rmcp`.
- `http::run`, `http::HttpError` (with `status`), re-exported `axum`, `Router`,
  and `StatusCode`.

Each transport module is compiled only under its feature; the harness
re-exports the feature-specific registration crates so domains do not need
duplicate direct dependencies to define commands, handlers, or routers.

## Guards

- The harness test suite verifies feature-gated CLI/MCP/HTTP behavior:
  single-newline output, JSON serialization, transport-tagged error context,
  CLI dispatch of a parsed command, and HTTP error status retention.
- A domain-crate reference workspace compiles and runs all three gated binaries
  through the production harness.
- Reference-proof integration tests (owned by memory-kernel) exercise one
  realistic domain operation across all three transports, asserting both the
  success output and the harness error envelope plus HTTP status mapping.

## Positions

- `crates/transport-harness/Cargo.toml`: implemented - production crate with
  empty defaults and independent CLI/MCP/HTTP features.
- `crates/transport-harness/src/lib.rs`: implemented - shared output, error
  normalization, tracing/runtime setup, CLI dispatch, MCP stdio startup, and
  HTTP startup/error mapping.

## Validation

Validation evidence for this crate is owned by memory-kernel. The harness
all-feature suite passes its unit tests and the full memory-kernel workspace
test run is the authoritative gate. Consuming repositories link to this spec and
its memory-kernel validation evidence rather than embedding inline results.