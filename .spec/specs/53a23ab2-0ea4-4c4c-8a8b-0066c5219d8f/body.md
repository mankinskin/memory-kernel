<!-- aligned-structure:v2 -->

## Reference

The canonical, normative contract for the shared transport harness now lives in
the memory-kernel repository, which owns the `transport-harness` crate.

- Canonical spec: `memory-kernel/.spec/specs/e5294ae5-6bff-44dc-81a9-24a44615b775/spec.toml` (slug `transport-harness`, id e5294ae5-6bff-44dc-81a9-24a44615b775), discoverable through the `memory-kernel/` submodule.
- Implementation: `memory-kernel/crates/transport-harness/`.
- Validation evidence: owned by memory-kernel (harness unit suite plus the full memory-kernel workspace run).

This context-engine spec is intentionally a pointer only. Do not duplicate the
harness responsibilities, non-goals, features, public API boundaries, or guards
here; consult the canonical memory-kernel spec for those normative requirements.
The compiling consumer example remains at
`workflow-tools-contract-reference/`, resolving the harness through a
branch-pinned Git dependency on memory-kernel.