<!-- aligned-structure:v2 -->
# Feedback Analytics Schema Migration

## Target Code Location

[migration module](workflow-tools/feedback/crates/feedback-api/src/migration.rs) and [feedback-api](workflow-tools/feedback/crates/feedback-api/)

## Naming Conventions

Use criterion ids `feedback-analytics-migration-*` for current-schema cutover and discarded-record reporting.

## Requester Input

> After verifying basic statistics on the old data, change the schema, migrate old records without a backup, and clean broken or unmigratable records.

## Reading Order

1. [Feedback History Analytics](.workflow-tools/spec/specs/835fbbfe-5eb5-4f15-a1a6-35330030813d/body.md) - shared cutover policy.
2. [Feedback Analytics Core](.workflow-tools/spec/specs/feedback-analytics-core) - validated source analytics.
3. [migration module](workflow-tools/feedback/crates/feedback-api/src/migration.rs) - current migration behavior.

## Responsibility

Migrate valid legacy records to the improved schema after current-schema analytics validation, then remove obsolete source history without backup retention.

## Interfaces And Dependencies

Consumes the validated analytics baseline and supplies updated schema records to analytics core and ingestion paths.

## Behavior

Valid records migrate to the new schema. Unmigratable records are permanently removed after reporting the total discarded count. Required baseline aggregate measures remain comparable after cutover.

## Boundaries And Failure Cases

Migration does not begin before analytics validation. No historic backup is retained. Cross-workspace feedback move behavior is outside scope.

## Provider/Consumer Contract

Analytics core consumes migrated records and compares required aggregates to the pre-cutover baseline.

## Examples

A malformed line increments the discarded total and is absent from the migrated store and all post-cutover statistics.

## Evidence

`cargo test --manifest-path workflow-tools/Cargo.toml -p feedback-api migration` and `cargo test --manifest-path workflow-tools/Cargo.toml -p feedback-api --test migration_integration`.

## Scope

Not implemented. This child owns schema migration only after the analysis interface is verified.