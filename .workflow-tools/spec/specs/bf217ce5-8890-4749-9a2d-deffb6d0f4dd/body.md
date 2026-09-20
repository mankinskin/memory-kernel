<!-- aligned-structure:v1 -->

# Summary

Define the architecture boundary for store-index generation so each domain owns a **thin** generator while `memory-api` exposes only reusable, domain-agnostic infrastructure. An implementer must be able to place new generation code without guessing which crate owns it.

## Behavior Story

Define the architecture boundary for store-index generation so each domain owns a **thin** generator while `memory-api` exposes only reusable, domain-agnostic infrastructure. An implementer must be able to place new generation code without guessing which crate owns it.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Goal

Define the architecture boundary for store-index generation so each domain owns a **thin** generator while `memory-api` exposes only reusable, domain-agnostic infrastructure. An implementer must be able to place new generation code without guessing which crate owns it.

# Problem

`memory-api` is the generic filesystem-backed entity-store backend (schema validation, SQLite indexing, Tantivy search, edges, board, workspace resolution). The memory-index track risks accreting specialized ticket/spec/rule/audit/workspace generation logic directly inside it.

The current tree already exhibits this drift: `memory-api/src/index_generator/{ticket,spec,rule,audit,workspace}.rs` embed domain normalization (e.g. ticket `state -> tags`, title-token keyword extraction) inside the generic backend. These generators read from the generic `IndexedEntity` (`storage::indexed`) rather than from domain crates, so they avoid a hard `memory-api -> ticket-api` dependency, but they still hard-code domain semantics in the wrong layer.

If domain-specific generators live centrally in `memory-api`:
- separation of concerns degrades (the generic backend knows ticket/spec/rule semantics),
- domain crates lose ownership of their own projections,
- reusable infrastructure becomes harder to extract and reuse cleanly.

# Contract

## C1: `memory-api` is generic infrastructure only

`memory-api` MUST NOT own domain-specific generator logic. It exposes only domain-agnostic building blocks:

- Schema types: `IndexEntry`, `IndexRef`, `IndexRelations`, `ContentKind`, `RelationKind` (`model/index_entry.rs`).
- Digest helpers: `IndexEntry::compute_digest`, `seal`, `is_digest_valid`.
- Sidecar codec + validator: `IndexSidecar`, `read_sidecar`, `write_sidecar`, `SidecarValidationIssue` (`model/index_sidecar.rs`).
- Generic rendering helpers: shared markdown/TOON emission utilities that take already-normalized `IndexEntry` values (`generated_markdown.rs`).
- Validation utilities: digest-consistency, sort-stability, and sidecar round-trip checks.
- Common test fixtures and helpers reused across domain generators.
- Generic source access: `EntityStore::list_indexed` / `IndexedEntity`, which expose store metadata without domain semantics.

`ContentKind` deliberately enumerates domain discriminants (`Ticket`, `Spec`, `Rule`, …). That is acceptable: it is a stable routing tag carried *through* the generic schema, not domain *behavior*. The rule is about logic placement, not enum membership.

## C2: Domain crates / CLIs own thin generators

The following responsibilities MUST live in the owning domain crate or its CLI, not in `memory-api`:

- Source loading from the domain store (which entities, which fields, filtering).
- Domain normalization: mapping source fields to `title`, `summary`, `keywords`, `tags`, `scope`, `non_goals` (the digest input contract — see sibling spec `generated-context/digest-input-contract`).
- Grouping/hierarchy decisions (spec tree folders, rule slug-prefix categories, workspace DAG edges).
- Domain-specific rendering choices (section ordering, headings, which views to emit).
- The generator entrypoint (the binary/subcommand that wires store open -> normalize -> seal -> render -> write).

A "thin" generator is one whose only non-trivial code is C2 work; everything else is a call into `memory-api` infrastructure.

## C3: Extension points between layers

The minimal shared surface `memory-api` provides so domain crates implement thin generators without duplicating infrastructure:

1. **Entry construction**: domain code builds `IndexEntry` values directly (public struct) and calls `seal()`. No domain-aware constructor in `memory-api`.
2. **Sidecar assembly**: `IndexSidecar::new(kind, store_dir, entries)` + `sort()` produce a stable, sealed sidecar. Domain code supplies entries; `memory-api` guarantees ordering and digest sealing invariants.
3. **Rendering trait/helpers**: a domain generator provides its normalized entries and grouping; `memory-api` rendering helpers turn entries into markdown/TOON. The render integration contract is owned by sibling spec `generated-context/rendering-pipeline-integration`.
4. **Validation entrypoint**: `memory-api` exposes a reusable validation routine (digest validity, sort order, sidecar round-trip) callable by any generator's tests and by the git hook.

No extension point may require `memory-api` to import a domain crate. Data flows domain-crate -> `memory-api` (push model): the domain constructs entries and hands them to generic helpers.

## C4: Synthetic / agent-hook surfaces

Generated agent-hook and workspace-summary surfaces (D1 third surface, `ContentKind::AgentHook` / `WorkspaceSummary`) are requested from each domain generator, not produced by `memory-api`. `memory-api` only provides the `ContentKind` discriminant and stable-ID helpers (UUID v5 over a fixed namespace + slug) so synthetic entries are diff-stable. Stable-ID rules are owned by the digest-input-contract spec; this spec only fixes that the *placement* decision is domain-owned.

## C5: Migration of current code

The existing `memory-api/src/index_generator/*` modules are the "before" state. The target is to relocate each domain generator's normalization + entrypoint to the owning domain crate/CLI, leaving only generic helpers in `memory-api`. Migration is sequenced by the individual generator tickets; this spec only fixes the destination boundary, not the move mechanics.

# Scope

- The ownership boundary between `memory-api` (generic) and domain crates/CLIs (thin generators).
- The minimal extension-point surface required to keep generators thin.
- The push-model dependency direction (no `memory-api -> domain` edges).

# Non-goals

- Does not implement any generator (deferred to the per-domain generator tickets).
- Does not redefine `IndexEntry` or the TOON sidecar format (owned by `0dba399a` / `e7a0ee3c`).
- Does not decide git-hook triggers or performance budgets (sibling specs).
- Does not define the per-field digest normalization rules (sibling `digest-input-contract` spec).

# Acceptance Criteria

- The contract explicitly states `memory-api` is generic infrastructure only and does not own domain generators (C1).
- The split of responsibilities between `memory-api` and domain crates is enumerated (C1, C2).
- The required extension points are identified and shown to avoid any `memory-api -> domain` dependency (C3).
- The five generator tickets (`9336a096` rule, `b9757ba7` spec, `c2409055` workspace, `855a1e5d` audit, `c5e9bb39` ticket) target thin domain-owned generators built on shared infrastructure.
- The boundary is precise enough to place new code without guessing crate ownership (C2 "thin" definition).

# Traceability

- Design ticket: [94c56f3d](.ticket/tickets/94c56f3d-774a-4b55-a13e-69c782ce9707/ticket.toml)
- Parent spec: generated-context/index-hierarchy-semantic-refs (`18b6a9c5`)
- Sibling specs: `generated-context/digest-input-contract`, `generated-context/rendering-pipeline-integration`
- Schema ticket: [0dba399a](.ticket/tickets/0dba399a-4691-4173-b921-17e5e6f6ebb8/ticket.toml)
- Sidecar ticket: [e7a0ee3c](.ticket/tickets/e7a0ee3c-dc2f-42dd-8c02-5070a747c156/ticket.toml)
- Downstream generator tickets: `9336a096`, `b9757ba7`, `c2409055`, `855a1e5d`, `a72e3aca`, `c5e9bb39`
- Current code under migration: `memory-api/crates/memory-api/src/index_generator/`
