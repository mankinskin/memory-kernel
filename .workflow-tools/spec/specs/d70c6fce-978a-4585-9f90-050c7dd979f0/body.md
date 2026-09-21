# Entity Kernel Domain and Entity Contract

## Objective
Define the generic identity and ownership contract for all current entity-bearing domains: ticket, test, spec, rule, session, feedback, and audit.

## Contract
The kernel distinguishes `domain_id`, `entity_type_id`, `entity_id`, `domain_schema_version`, and `entity_type_schema_version`. Every entity has exactly one owning domain. Other domains may reference the canonical entity through typed edges or external URNs but do not become persistence, schema, or migration authorities.

The kernel owns generic manifest shape, entity-type schema validation, schema registry loading, store discovery, and cross-store edge classification. Domains own entity-type membership, domain adapters, migrations, hooks, commands, and projections. Workspaces own installed/active domain composition and capability reporting.

Audit is a hybrid domain: persisted finding folders may be kernel entities; repository-level indexes and generated catalogs remain projections.

## Acceptance Criteria
- The five identifiers are represented without collapsing domain and entity type.
- Ownership validation rejects two canonical owners for one entity.
- Cross-domain references resolve through typed edges or external URNs.
- Audit projection artifacts are not treated as canonical entity records.
- Existing domain business schemas remain unchanged.

## Non-goals
No domain-specific lifecycle redesign, new business fields, arbitrary domain functions, or implementation of the contract in this specification pass.

## Evidence
- `transcripts/15-09-2026_entity-kernel-all-domains/01-kernel-contract.md`
- `workflow-tools/memory-kernel/src/model/entity.rs`
- `workflow-tools/memory-kernel/src/model/schema.rs`
- `workflow-tools/memory-kernel/src/cross_store_edges.rs`
- `workflow-tools/audit/crates/audit-api/src/move_domain.rs`
