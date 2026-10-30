# memory-kernel

`memory-kernel` is the shared, filesystem-backed substrate for workflow-domain
tools. It provides neutral entity-store, indexing, search, workspace, board,
and cross-store move primitives.

## Public contract

Use the `memory_kernel` library crate from domain repositories. The public
surface is domain-neutral: domains supply their own schemas, entity semantics,
and transport adapters.

The package replaces the legacy `memory-api` shared-kernel crate. During the
repository extraction, consumers should migrate imports from `memory_api` to
`memory_kernel` without changing the neutral storage, model, workspace, and
board APIs. The `InteroperableArtifact` contract is now owned by this package
because move journals are kernel artifacts; domain-specific interoperability
traits remain with their domains.

## Shared lifecycle

`memory_kernel::lifecycle` owns the shared six-state vocabulary,
`LifecycleSchema` JSON/TOML parsing, `LifecycleDefinition` schema-driven
transition defaults, and `BasicLifecycle` pause/resume/cancel/complete rules.
`LifecycleSubject` supplies domain-owned identity, snapshot and error types
to the independent read, transition and capability traits. Unsupported
capabilities fail explicitly by default.

Domains implement `NativeLifecycle` to map their own state names. The kernel
does not know Ticket states, Mission identities, store paths, approval policy
or journal persistence. Consumers retain those adapters and authorization
checks; shared control defaults do not grant execution authority.

## Transport harness

The workspace also publishes the sibling `transport-harness` crate. Workflow
domain binaries enable only the transports they expose:

```toml
transport-harness = { path = "../memory-kernel/crates/transport-harness", optional = true, default-features = false }

[features]
cli = ["dep:transport-harness", "transport-harness/cli"]
mcp = ["dep:transport-harness", "transport-harness/mcp"]
http = ["dep:transport-harness", "transport-harness/http"]
```

The harness owns CLI parsing and output, MCP stdio startup, HTTP listener
startup, tracing, and transport error normalization. Domain crates retain
their command dispatch, MCP `ServerHandler`, and HTTP `Router` registration.

## Versioning

This package follows semantic versioning. Additive public APIs are minor
changes; breaking API or behavior changes are major changes. Compatibility
aliases, when needed for the workflow-tools migration, must be documented with
their removal version before being added.

## Development

```bash
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

The extracted history comes from `memory-api/crates/memory-api`. Per-domain
APIs and transport binaries intentionally do not belong in this repository;
only their transport-neutral startup harness is shared here.
