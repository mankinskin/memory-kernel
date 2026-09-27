<!-- aligned-structure:v2 -->
# Kernel Cross-Repository Move Safety

## Target Code Location
- [Move kernel](../../workflow-tools/memory-kernel/src/storage/move_kernel.rs)
- [Move kernel types](../../workflow-tools/memory-kernel/src/storage/move_kernel_types.rs)
- [Move kernel internals](../../workflow-tools/memory-kernel/src/storage/move_kernel/internal.rs)

## Naming Conventions
The public option is `allow_cross_repository`. The owning contract types are `MoveDomain`, `MovePlan`, `MoveSetPlan`, and `MoveBlocker`.

## Requester Input
> We should add `allow_cross_repository`, default it to true, and enable cross-repository moves while safe. Add universal reference-visibility blocking, batch closure, filesystem checks, sequential batches, reconciliation, and fixed commit order.

## Reading Order
1. [Parent safety contract](../../workflow-tools/.workflow-tools/spec/specs/7487a6b5-39c6-4114-a697-6ed0556e888e/body.md)
2. [Entity move execution rule](../../workflow-tools/.agents/instructions/workflow/entity-move-execution.instructions.md)
3. [Shared move kernel](../../workflow-tools/memory-kernel/src/storage/move_kernel.rs)

## Responsibility
Define the domain-neutral preflight and journal guarantees that make cross-repository moves safe by default.

## Interfaces And Dependencies
The kernel owns topology policy, reference visibility enforcement, destination-local batch closure, filesystem compatibility checks, deterministic lock ordering, and the `allow_cross_repository` option. Domain adapters provide target-store initialization and domain-specific reference checks.

## Behavior
Cross-repository planning proceeds by default when all safety checks pass. Setting `allow_cross_repository=false` restores an explicit caller-level restriction. Invisible references always produce a hard blocker. Set planning rejects incomplete destination-local relation components. Apply runs one batch at a time and records the checks used for reconciliation.

## Boundaries And Failure Cases
Block on missing code references, invisible references, dirty tracked files, active leases or board entries, incompatible filesystem devices, target stores outside the workspace, incomplete batch closure, or non-terminal journals. Never fall back to filesystem copy or deletion.

## Provider/Consumer Contract
The Spec-domain child consumes kernel topology, visibility, batch, and filesystem criteria; the test child consumes every observable blocker and journal phase.

## Examples
A cross-repository set with a related entity left at the source is rejected before apply. A complete set with visible references and compatible filesystems produces a supported plan.

## Evidence
Validation must include unit tests for same-repository, parent/submodule, sibling-repository, denied-option, invisible-reference, incomplete-batch, and filesystem-device scenarios, plus benchmark measurements for each topology.

## Scope
Owner: `memory-kernel`; implementation is ticket-backed and remains partial until the linked tests pass.
