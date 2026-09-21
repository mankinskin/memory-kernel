<!-- aligned-structure:v2 -->

## Motivation
Ticket schemas are duplicated, transitions are currently undirected, and schema loading cannot express the planned work-item hierarchy safely. This contract establishes directed lifecycle graphs, runtime inheritance, compatible TOML/JSON schema loading, and an auditable migration path.

## Dependent Expectation
If implemented, ticket, spec, and rule schema consumers can resolve an inherited schema deterministically, validate directed lifecycle transitions, and migrate legacy ticket records without losing an auditable transition history.

## Guards
Guards will be recorded by the linked implementation tracks before review: resolver contracts, loader parity fixtures, migration dry-run/idempotence/cutover checks, transport parity, and browser checks for client changes.

## Positions
- `memory-api/crates/memory-api/src/model/schema.rs`: partial; flat state strings and undirected transitions.
- `memory-api/crates/ticket-api/src/model/schema_registry.rs`: partial; flat TOML-only registry with replacement semantics.
- `memory-api/crates/ticket-api/schemas/`: partial; duplicated built-in workflows.
- `memory-api/tools/cli/ticket-cli`: partial; default type is legacy `tracker-improvement`.
- `memory-api/ticket-vscode`: partial; type picker is hard-coded to `tracker-improvement`.

## Contract
- Features are spec-owned. Ticket types are `epic`, `bug`, `research`, `planning`, `implementation`, `review`, `interview`, and `testing`; `task`, `feature`, and `tracker-improvement` are legacy until migration.
- Schemas use strict single-parent inheritance from a universal work-item lifecycle. Resolved lifecycle graphs use `plan`, `act`, and `verify` categories; every concrete type has a valid path through all three.
- Lifecycle graphs are directed schema transitions. Relation graphs are ticket links and have no lifecycle-category constraints.
- TOML and JSON custom schemas have identical semantics. Shipped ticket built-ins convert to JSON; any duplicate type ID in the same schema-kind registry hard-fails atomic load/reload.
- Legacy migration is transactional, auditable, and forward-repairable. Ambiguous classification requires a linked review-ticket decision.

## Governing-Rule Requirement
A policy rule introducing this spec and the validation guards must be added or linked by the implementation tracks before the spec is treated as implemented.