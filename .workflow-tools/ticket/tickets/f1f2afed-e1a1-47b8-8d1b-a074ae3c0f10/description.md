## Objective
Implement the domain-neutral cross-repository move contract in `memory-kernel`.

## Requirements
- Add `allow_cross_repository`, defaulting to `true`, with an explicit false path.
- Make `reference_visibility.visible_from_destination == false` a hard blocker for every topology.
- Enforce destination-local connected-component closure for set moves.
- Preflight filesystem/device compatibility before rename.
- Preserve deterministic lock ordering and journal recovery semantics.
- Require sequential batch execution and expose reconciliation metadata without relying on uncommitted rewrites.

## Acceptance Criteria
1. Same-repository, parent/submodule, submodule/parent, and unrelated repository plans cover allow=true and allow=false.
2. Invisible references, incomplete batch closure, dirty files, incompatible filesystems, active leases, and non-terminal journals block before mutation.
3. Successful cross-repository moves retain source/destination read-back and resume/rollback guarantees.
4. Kernel unit tests cover all blocker and success paths.

## Validation
`cargo test --manifest-path workflow-tools/memory-kernel/Cargo.toml`
