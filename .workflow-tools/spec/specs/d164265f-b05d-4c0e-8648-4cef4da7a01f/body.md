# Entity Kernel Workspace Composition and Capabilities

## Objective
Define workspace-level discovery, domain activation, dependency resolution, and structured unavailable capabilities.

## Contract
A workspace distinguishes installed domains from active domains. Domain manifests expose dependencies and capabilities. Workspace composition resolves dependencies using existing workspace policy and discovery rules. Operations that require an unavailable optional domain return a structured unavailable-capability result naming the operation, missing/inactive domain, and dependency reason. CLI/HTTP surfaces may render that result as an actionable user-facing error while independent domain operations continue.

Cross-workspace entity references remain external URNs. Workspace composition reports availability and optional version constraints without taking ownership of referenced entities.

## Acceptance Criteria
- Installed, active, inactive, and unavailable domain states are distinguishable.
- Dependency resolution is deterministic and reports unsatisfied dependencies.
- Missing optional spec capability does not prevent ticket operations.
- Ticket/spec tracking reports a structured unavailable result when spec is absent.
- Existing workspace-policy discovery compatibility remains green.

## Non-goals
No mandatory installation of optional domains, no generalized plugin system, and no domain business-schema changes.

## Evidence
- `transcripts/15-09-2026_entity-kernel-all-domains/05-workspace-composition.md`
- `workflow-tools/memory-kernel/src/workspace.rs`
- `workflow-tools/memory-kernel/src/workspace_policy.rs`
- `workflow-tools/memory-kernel/src/cross_store_edges.rs`
