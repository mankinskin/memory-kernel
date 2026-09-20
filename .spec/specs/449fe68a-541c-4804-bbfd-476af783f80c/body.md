<!-- aligned-structure:v1 -->

# Summary

Define the domain-level digest input contract for generated memory-index entries so every generator derives a stable `IndexEntry` payload before calling `seal()`. Given identical source inputs, every generator must produce an identical digest across runs, platforms, and toolchains.

## Behavior Story

Define the domain-level digest input contract for generated memory-index entries so every generator derives a stable `IndexEntry` payload before calling `seal()`. Given identical source inputs, every generator must produce an identical digest across runs, platforms, and toolchains.

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

Define the domain-level digest input contract for generated memory-index entries so every generator derives a stable `IndexEntry` payload before calling `seal()`. Given identical source inputs, every generator must produce an identical digest across runs, platforms, and toolchains.

# Problem

`IndexEntry::compute_digest` already defines a generic, ordered SHA-256 over nine stable fields. But the *domain* layer that produces those fields — `title`, `summary`, `keywords`, `tags`, `scope`, `non_goals`, synthetic IDs, and relation ordering — is unspecified per domain. The current generators (`memory-api/src/index_generator/*`) use placeholder normalization (`summary = state`, keyword extraction from title only), so "digest stability" is a requirement with no executable per-domain plan.

# Generic digest contract (inherited, do not change)

From `model/index_entry.rs`, the digest hashes these fields in this fixed order, `\0`-separated:

1. `id` (UUID hyphenated)
2. `kind` (serde snake_case)
3. `source_path` (`/` separators)
4. `title`
5. `summary`
6. `scope` (empty string when absent)
7. `non_goals` (empty string when absent)
8. `keywords` sorted, joined with `,`
9. `tags` sorted, joined with `,`

Excluded: `digest`, `generated_at`, `source_modified_at`, all `relations`. This contract owns how each domain fills fields 1 and 3–9; it does not change the algorithm.

# Cross-domain normalization rules

These apply to every generator before `seal()`:

- **N1 source_path**: workspace-relative, `/` separators (`to_relative_slash`). Always points at the canonical source artifact (`ticket.toml`, spec TOML, `rule.toml`, finding path, or `<store>/index.toon` for synthetic roots).
- **N2 title**: source title trimmed; collapse internal runs of whitespace to single spaces; strip trailing/leading whitespace; truncate to 200 chars on a char boundary. Fallback to the entity id string when the source has no title.
- **N3 summary**: a normalized dense excerpt (see per-domain rules). Newlines and CR collapsed to single spaces, internal whitespace runs collapsed, trimmed; truncate to 500 chars on a char boundary. Never the bare state string (upgrades current placeholder).
- **N4 keywords**: lower-cased, each token trimmed of non-alphanumeric edges, tokens of length > 3 kept, de-duplicated, sorted ascending. Stop set may drop common stopwords; the stop set is fixed and documented per domain.
- **N5 tags**: lower-cased, de-duplicated, sorted ascending. Domain chooses the tag *sources* (state, category, slug-prefix, etc.); ordering/casing is universal.
- **N6 scope / non_goals**: when the source has explicit scope / non-goals fields, normalize like N3; otherwise `None` (serialized as empty string in the digest).
- **N7 relations**: excluded from the digest, but MUST be emitted in a stable order (sort `children` / `depends_on` / `related` by `entry_id`, then `relation_kind`) so generator output is diff-stable even though relations do not affect the digest.

# Stable synthetic IDs

Synthetic entries (no domain-store UUID) MUST use UUID v5 over a fixed per-domain namespace + a stable slug string, so the id never drifts between runs:

- **Audit root**: `deterministic_uuid(AUDIT_NS, "audit-root:<store_dir>")` (existing `AUDIT_NS`).
- **Workspace node**: `deterministic_uuid(WORKSPACE_NS, <store_dir>)` (existing `WORKSPACE_NS`).
- **Workspace neighbour refs**: `deterministic_uuid(WORKSPACE_NS, <relative_path>)`.
- **Agent-hook / index entries**: `deterministic_uuid(<domain_ns>, "<kind>:<stable-slug>")`.

The namespace constants are fixed bytes and MUST NOT change once published (changing them re-IDs every synthetic entry). Mutable inputs (counts, timestamps, freshness summaries) MUST NOT feed the synthetic-ID slug — proven by the existing `workspace_entry_id_is_deterministic` test where a changed summary leaves the id unchanged.

# Per-domain digest input contract

## Ticket (`c5e9bb39`)
- id: ticket store UUID. kind: `Ticket`. source_path: ticket folder `ticket.toml`.
- title: ticket title (N2). summary: first paragraph of the ticket description, else the ticket state phrase (N3) — NOT bare state.
- keywords: N4 over title + description-derived terms.
- tags: ticket `state`, `priority`, `type` (N5).

## Spec (`b9757ba7`)
- id: spec UUID. kind: `Spec`. source_path: spec TOML.
- title: spec title (N2). summary: spec Goal/first-section excerpt (N3).
- scope: spec `scope` field (N6). non_goals: spec non-goals section when present (N6).
- keywords: N4 over title + goal terms. tags: spec `state`, component (N5).

## Rule (`9336a096`)
- id: rule UUID. kind: `Rule` (or `RuleCatalog` for catalog roots). source_path: `rule.toml`.
- title: rule title (N2). summary: rule body excerpt (N3).
- keywords: N4. tags: slug-prefix segments (D4 grouping, via `slug_prefix_tags`) + state (N5).

## Audit (`855a1e5d`)
- root: synthetic `WorkspaceSummary`, stable id (AUDIT_NS). title `Audit summary — <rating>`; summary = run summary (N3); tags `["audit"]`.
- finding: kind `AuditFinding`, stable id per finding id string; title from category; summary from finding summary (N3); tags `[severity, category]` (N5).

## Workspace (`c2409055`)
- id: synthetic WorkspaceSummary, stable id (WORKSPACE_NS over store_dir). title `<name> workspace`; summary = freshness/health string (N3); scope = store_dir; tags `[name, "workspace"]` (N5).
- relations: parent/child workspace refs sorted per N7 (DAG topology, excluded from digest).

# Validation evidence

At least one focused fixture/test per domain proving identical source inputs yield identical digests across two generation passes, plus a synthetic-ID stability test (mutable input changed -> id and digest of stable fields unchanged). The existing `digest_is_deterministic`, `seal_sets_digest_field`, and `workspace_entry_id_is_deterministic` tests are the seed; the contract requires one such proof per domain generator.

# Scope

- Per-domain mapping from source fields to the nine digest input fields.
- Universal normalization rules (N1–N7).
- Stable synthetic-ID derivation and the invariant that mutable inputs never feed IDs.
- Relation ordering for diff stability (relations excluded from digest).

# Non-goals

- Does not wire git hooks (git-hook-automation spec).
- Does not implement the generator binaries (per-domain generator tickets).
- Does not change the SHA-256 digest algorithm (`0dba399a`).

# Acceptance Criteria

- A per-domain normalization contract exists for ticket, spec, rule, audit, and workspace generators.
- The contract names the exact source fields and normalization rules used before `compute_digest()` / `seal()`.
- Stable-ID rules are documented for synthetic entries (audit root, workspace node, agent-hook), with the "mutable inputs never feed the ID" invariant.
- At least one focused validation artifact per domain proves unchanged source inputs yield unchanged digests.
- The contract is precise enough that generator implementers do not need to guess how to produce `summary`, `keywords`, `tags`, or ordering.

# Traceability

- Design ticket: [7f7fe4a8](.ticket/tickets/7f7fe4a8-a1d6-44b4-baf9-9500f6db40a5/ticket.toml)
- Parent spec: generated-context/index-hierarchy-semantic-refs (`18b6a9c5`)
- Depends-on spec: generated-context/rendering-pipeline-integration (`9109f12a`)
- Sibling spec: generated-context/thin-generator-architecture (`bf217ce5`)
- Schema ticket: [0dba399a](.ticket/tickets/0dba399a-4691-4173-b921-17e5e6f6ebb8/ticket.toml)
- Generator tickets: `c5e9bb39`, `b9757ba7`, `9336a096`, `855a1e5d`, `c2409055`
- Current code: `memory-api/crates/memory-api/src/index_generator/` (placeholder normalization to be upgraded)
