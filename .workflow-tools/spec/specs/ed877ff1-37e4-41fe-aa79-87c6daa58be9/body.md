<!-- aligned-structure:v2 -->
# Feedback Analytics Core

## Target Code Location

[feedback-api](workflow-tools/feedback/crates/feedback-api/)

## Naming Conventions

Use feedback analytic event, incident aggregate, and resolution signal types with criterion ids `feedback-analytics-core-*`.

## Requester Input

> Treat existing feedback history as a time series and analyze tool or operation problems plus positive or negative events.

## Reading Order

1. [Feedback History Analytics](.workflow-tools/spec/specs/835fbbfe-5eb5-4f15-a1a6-35330030813d/body.md) - shared invariants.
2. [feedback-api](workflow-tools/feedback/crates/feedback-api/) - provider implementation.

## Responsibility

Provide read-only statistical queries over raw feedback events and derived recurring incidents.

## Interfaces And Dependencies

The core reads existing NDJSON and canonical feedback records and provides the shared results consumed by CLI and MCP adapters.

## Behavior

Reports valid and malformed counts, daily volume, field distributions, tool/operation negative trends, first-seen, last-seen, explicit resolutions, and tentative resolutions after 10 quiet days with at least 100 valid events.

## Boundaries And Failure Cases

Malformed records do not enter statistical aggregates but increment data-quality totals. Current-schema analytics must not mutate production history.

## Provider/Consumer Contract

CLI and MCP adapters consume the same core result types and must not duplicate calculations.

## Examples

A negative tool incident with no matching negative event for 10 days and 100 other valid events receives a tentative, not explicit, resolution label.

## Evidence

`cargo test --manifest-path workflow-tools/Cargo.toml -p feedback-api analytics` and `cargo test --manifest-path workflow-tools/Cargo.toml -p feedback-api --test analytics_integration`.

## Scope

Not implemented. This child excludes schema cutover and adapter presentation.