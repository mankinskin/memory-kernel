<!-- aligned-structure:v2 -->

## Motivation

The shared filesystem-backed entity storage, indexing, search, workspace, and board substrate previously lived in the legacy `memory-api` package. It now has to be independently versioned as `memory-kernel` so workflow domains can depend on a neutral shared kernel rather than an umbrella domain package.

Execution is tracked by [e12d8343 Extract shared storage kernel](.ticket/tickets/e12d8343-24f2-4b5d-8023-5a071238904a/description.md), with neutral naming coordination in [13912e44 Neutral naming migration map](.ticket/tickets/13912e44-fee8-4aa5-b28f-68bbc22af401/description.md) and [2b1279bd Neutral storage kernel migration](.ticket/tickets/2b1279bd-c42f-4b0e-8835-d0d645a733ab/description.md).

## Dependent expectation

If this spec is implemented, a workflow domain can use `memory-kernel` as an external Cargo dependency for neutral entity-store, index, search, workspace, board, and cross-store move operations without referencing the old `memory-api` workspace path.

## Guards

- `memory-kernel-standalone-extraction` in `.test/default/specs/memory-kernel-standalone-extraction.json` guards standalone formatting, linting, tests, and external path-dependent consumption.
- Passing execution: `.test/default/executions/memory-kernel-standalone-extraction-20260725.json`.

## Positions

- `memory-api/crates/memory-api/src/lib.rs`: deprecated - legacy source preserved as the extraction origin.
- `../memory-kernel/Cargo.toml`: implemented - standalone workspace and public `memory-kernel` package.
- `../memory-kernel/src/lib.rs`: implemented - neutral public library surface under `memory_kernel`.
- `../memory-kernel/src/interoperability.rs`: implemented - kernel-owned `InteroperableArtifact` contract for move journals.
- `../memory-kernel/README.md`: implemented - public compatibility and versioning contract.
- `../memory-kernel/.github/workflows/ci.yml`: implemented - formatting, tests, and inherited non-fatal clippy baseline.

## Governing-rule requirement

This spec is introduced under the repository ticket/spec workflow in `AGENTS.md` and `.agents/instructions/spec/spec-system.instructions.md`. The durable session’s pinned rule renderer is currently blocked by stale rule IDs; that infrastructure issue must not be treated as evidence that this extraction contract is optional.

## Non-goals

- This extraction does not migrate per-domain APIs or transport binaries.
- This extraction does not publish a crates.io release.
- Strict `-D warnings` clippy cleanup is separate inherited debt; it is not a functional extraction requirement.