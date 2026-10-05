<!-- aligned-structure:v2 -->

# Workspace And Domain-Store Contract

## Motivation

All active workflow-tools domains need consistent workspace selection and canonical store semantics without erasing domain-specific entities or operations. The existing resolver owns workspace/path resolution; this specification extends that contract with a kernel-owned store abstraction that domain APIs can implement independently.

## Target Code Location

- [Workspace resolver](../../../../src/workspace.rs)
- [MoveDomain trait precedent](../../../../src/storage/move_kernel_types.rs)
- [UrnResolver trait precedent](../../../../src/model/urn.rs)
- [Ticket API store](../../../../../ticket/crates/ticket-api/src/storage/store/store_open.rs)
- [Spec API store](../../../../../spec/crates/spec-api/src/store.rs)
- [Test API store](../../../../../test/crates/test-api/src/store.rs)
- [Session API store](../../../../../session/crates/session-api/src/store.rs)
- [Feedback API store](../../../../../feedback/crates/feedback-api/src/canonical.rs)
- [Audit API store](../../../../../audit/crates/audit-api/src/index.rs)
- [Log API store](../../../../../log/crates/log-api/src/store.rs)

## Naming Conventions

- `local_workspace` is the selected operation and write base; `parent_workspace` and `global_workspace` are discovery contexts only.
- The active domain set is Ticket, Spec, Test, Session, Feedback, Audit, and Log.
- The canonical store is `<local_workspace>/.workflow-tools/<domain>`; the legacy store is `<local_workspace>/.<domain>`.
- The kernel-owned core trait is `DomainStore`; operation capabilities are `CreateEntity`, `ReadEntity`, `UpdateEntity`, and `DeleteEntity`. `ListEntities` is a separate optional query capability.
- Criterion identifiers are `workspace-selector`, `canonical-store-root`, `legacy-read-write-policy`, `reference-owner-root`, `discovery-write-base`, `domain-store-core-trait`, and `typed-crud-capabilities`.

## Requester Input

> Provide a shared trait interface for domain APIs to implement a standard surface.

## Reading Order

1. [Entity-store namespace Spec](../../../../../../.workflow-tools/spec/specs/02c6700d-6ab2-44a8-aa69-4052ad2f76bb/spec.toml) defines canonical namespace and legacy-read behavior.
2. [Workspace resolver](../../../../src/workspace.rs) owns selection, canonical derivation, diagnostics, and discovery.
3. [MoveDomain](../../../../src/storage/move_kernel_types.rs) and [UrnResolver](../../../../src/model/urn.rs) are kernel-owned trait precedents.
4. The seven domain API store entry points above are consumers; their entity models and supported operations remain domain-owned.

## Responsibility

Define the shared contract for resolving, opening, and using domain stores from a selected local workspace. This specification is the workspace-resolution owner; it does not replace domain-specific storage contracts.

## Interfaces And Dependencies

`memory-kernel` owns the domain-neutral `DomainStore` core trait. The trait exposes domain identity, selected workspace/store resolution, structured diagnostics, and explicit read-only versus create/open behavior. Kernel-owned types must not depend on domain API crates.

The typed `CreateEntity`, `ReadEntity`, `UpdateEntity`, and `DeleteEntity` capability traits keep each domain's identifier, input, entity, patch, result, and error types. `ListEntities` keeps domain-specific query and page/result types. A domain implements only capabilities matching operations its current API supports; no universal entity schema or unsupported operation is introduced. Domain API crates retain their domain-specific extensions.

## Behavior

### Workspace selection and store resolution

- An explicit `workspace="."` selects the caller's current directory and is normalized to its concrete local workspace before use or persistence. Omitted, blank, `default`, and `..` selectors are invalid for entity creation.
- Resolve canonical storage as `<local_workspace>/.workflow-tools/<domain>`. If only a legacy store exists, reads may use it and return a legacy diagnostic; writes never target it.
- If canonical and legacy layouts both exist, select canonical and return `BothLayoutsPresent`. Do not reject the workspace as ambiguous or silently write to the legacy path.
- Workspace-based writes target canonical storage. Preserve a direct concrete canonical-store override where supported. Reject legacy-shaped write overrides, including a nonexistent explicit legacy path, rather than creating legacy storage or redirecting the request.
- Read-only open/query operations do not initialize a missing store. Create/open operations make the chosen mode explicit.
- Normalize repository roots, direct store roots, and paths within a store to the owning workspace without changing the selected local write base. Apply path validation to relative, absolute, Windows/UNC, and traversal inputs.

### References and discovery

- Resolve a relative `store_root` on a persisted cross-store reference beneath that reference's recorded owner workspace, independent of the validator's selected local workspace. Preserve the reference fields and supported absolute override behavior.
- `parent_workspace` and policy-approved `global_workspace` roots may add read/search results. They never replace `local_workspace` for relative validation or writes.
- The nearest local store may be discovered for read compatibility, but discovery is not a write-base selector.

### Observable criteria

- `workspace-selector`: explicit dot selection resolves to and persists the concrete local workspace; invalid aliases fail before creation.
- `canonical-store-root`: each active domain derives its canonical path from the selected local workspace.
- `legacy-read-write-policy`: legacy-only reads remain compatible; canonical wins with `BothLayoutsPresent` when both layouts exist; all writes are canonical-only and reject legacy overrides.
- `reference-owner-root`: the same stored reference resolves to the same owner target and classification from aggregate and nested invocation roots.
- `discovery-write-base`: additional discovery roots affect reads/search only and do not alter subsequent write targets.
- `domain-store-core-trait`: `DomainStore` exposes domain identity, workspace/store resolution, diagnostics, and explicit open mode without depending on domain crates.
- `typed-crud-capabilities`: active domain APIs implement only supported typed operation capabilities and retain their existing entity contracts and extensions.

## Boundaries And Failure Cases

- Rule is excluded from the active domain set and from this shared trait contract.
- Do not merge or migrate legacy data as part of resolution. A caller requesting a legacy write path receives an actionable error.
- Do not auto-initialize a store for read-only access, and do not use a parent/global discovery result as a substitute for the selected local workspace.
- Do not force identical CRUD semantics across domains or change their serialized entity models.

## Provider/Consumer Contract

The Ticket, Spec, Test, Session, Feedback, Audit, and Log APIs consume the `DomainStore` workspace/store-resolution contract and implement only the typed operation capabilities they support. Each API remains the provider of its domain-specific entity semantics. The [entity-store namespace Spec](../../../../../../.workflow-tools/spec/specs/02c6700d-6ab2-44a8-aa69-4052ad2f76bb/spec.toml) supplies the shared canonical/legacy path vocabulary.

## Examples

- A caller using `workspace="."` from `/repo/child` resolves and persists `/repo/child` as `local_workspace`; a Ticket write targets `/repo/child/.workflow-tools/ticket`.
- If `/repo/.workflow-tools/spec` and `/repo/.spec` both exist, a Spec read selects the canonical directory and returns `BothLayoutsPresent`; a write still targets only the canonical directory.
- A relative `store_root` on a reference owned by workspace `default` resolves under the aggregate process root even when validation runs from a nested workspace.

## Evidence

- Existing guard: [val-memory-kernel-workspace-resolution](../../../../.workflow-tools/test/specs/val-memory-kernel-workspace-resolution.json), linked to `workspace-selector`, `canonical-store-root`, and `discovery-write-base`. Extend or add focused guards with the implementation work for the new trait and cross-domain criteria.
- Positions: [workspace resolution](../../../../src/workspace.rs) is partial; `DomainStore` and the typed capability traits are not yet implemented. The domain API entry points are existing consumers whose adaptation is implementation work, not claimed completion here.

## Scope

This contract covers workspace resolution and the shared API boundary for Ticket, Spec, Test, Session, Feedback, Audit, and Log. It does not implement source changes, rewrite domain entity contracts, invoke Rule surfaces, migrate entities, or execute entity distribution.