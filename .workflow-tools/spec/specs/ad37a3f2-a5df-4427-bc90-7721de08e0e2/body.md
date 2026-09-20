<!-- aligned-structure:v2 -->
# Feedback Analytics Interfaces

## Target Code Location

[feedback-cli](workflow-tools/feedback/crates/feedback-cli/) and [feedback-mcp server](workflow-tools/feedback/crates/feedback-mcp/src/server.rs)

## Naming Conventions

Use deterministic analytics report/query names with criterion ids `feedback-analytics-interface-*`.

## Requester Input

> Add general statistical tools to understand monitoring problems and resolutions.

## Reading Order

1. [Feedback History Analytics](.workflow-tools/spec/specs/835fbbfe-5eb5-4f15-a1a6-35330030813d/body.md) - shared invariants.
2. [Feedback Analytics Core](.workflow-tools/spec/specs/feedback-analytics-core) - provider results.

## Responsibility

Expose the shared analytics core through deterministic CLI reports and MCP query tools.

## Interfaces And Dependencies

Both adapters consume feedback-api analytics types and return equivalent fields for equal inputs.

## Behavior

The CLI supplies concise human-readable and machine-readable reports. MCP supplies programmatic analytics queries for counts, distributions, trends, and incident lifecycle output.

## Boundaries And Failure Cases

Invalid parameters fail clearly. Interfaces must not mutate records or duplicate core statistics.

## Provider/Consumer Contract

This child consumes the analytics-core provider criteria for aggregate and lifecycle result fields.

## Examples

The same fixture query for a tool's negative trend returns equal time buckets and counts through CLI and MCP.

## Evidence

`cargo test --manifest-path workflow-tools/Cargo.toml -p feedback-cli analytics` and `cargo test --manifest-path workflow-tools/Cargo.toml -p feedback-mcp analytics`.

## Scope

Not implemented. This child excludes schema migration.