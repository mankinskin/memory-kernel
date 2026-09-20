<!-- aligned-structure:v1 -->

# Summary

Define the shared rendering-pipeline integration for generated store indexes so README/index files route through one rendering paradigm instead of per-domain ad hoc renderers. Store-index generation must align with the existing `rule-api`/`spec-api` rendering path, not fragment it.

## Behavior Story

Define the shared rendering-pipeline integration for generated store indexes so README/index files route through one rendering paradigm instead of per-domain ad hoc renderers. Store-index generation must align with the existing `rule-api`/`spec-api` rendering path, not fragment it.

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

Define the shared rendering-pipeline integration for generated store indexes so README/index files route through one rendering paradigm instead of per-domain ad hoc renderers. Store-index generation must align with the existing `rule-api`/`spec-api` rendering path, not fragment it.

# Problem

The memory-index track risks each generator inventing its own file-rendering path. The repository already has a shared, provenance-marked markdown renderer; a parallel system would create multiple rendering paradigms side-by-side and make future ticket/spec structured rendering harder to unify.

# Current-state finding (grounding)

The shared renderer already exists and is in production use:

- `memory-api::generated_markdown` provides the generic primitives:
  - `GeneratedMarkdownSnippet { id, slug, body }`
  - `GeneratedMarkdownConfig::new(file_comment, entry_prefix)` (with `skip_provenance_for_yaml_frontmatter`)
  - `render_markdown_file(snippets, config) -> String`
  - `prepare_generated_output(rendered, existing)` (newline normalization + preserve existing CRLF/LF style)
- `rule-api/src/render.rs` wraps it with `file_comment = "<!-- rule-api:file generated=true -->"` and `entry_prefix = "rule-api:entry"`.
- `spec-api/src/store.rs` also renders through the same `GeneratedMarkdownConfig` + `render_markdown_file`.

So the "one rendering paradigm" already physically exists in `memory-api`. This spec fixes that store-index generators MUST reuse it rather than introduce a new path.

# Contract

## R1: One rendering paradigm

All generated store-index README/index markdown files (folder READMEs, `.ticket/`, `.spec/`, `.rule/`, `.audit/` indexes, and `.agents/` hook surfaces) MUST be rendered through `memory-api::generated_markdown::render_markdown_file`, using a per-domain `GeneratedMarkdownConfig`. No generator may hand-roll markdown emission or provenance markers.

## R2: Per-domain provenance config

Each domain generator declares its own `file_comment` and `entry_prefix` constants (mirroring `rule-api:file` / `rule-api:entry`), e.g.:

- ticket index: `<!-- ticket-index:file generated=true -->` / `ticket-index:entry`
- spec index: `<!-- spec-index:file generated=true -->` / `spec-index:entry`
- rule catalog: reuse `rule-api:*` where the catalog is rule-owned, else `rule-index:*`
- audit summary: `<!-- audit-index:file generated=true -->` / `audit-index:entry`
- workspace summary: `<!-- workspace-index:file generated=true -->` / `workspace-index:entry`

The `entry_prefix` carries the entry `id` (UUID) and `slug` so generated files remain diff-stable and re-parseable.

## R3: Snippet construction from sealed entries

A domain generator converts its already-normalized, **sealed** `IndexEntry` values (digest input contract sibling spec) into `GeneratedMarkdownSnippet`s. The body of each snippet is the human-readable rendering of the entry; the `id`/`slug` come from the entry. Rendering consumes normalized data — it never re-normalizes or re-derives digests.

## R4: Write-time stability

Generators MUST pass output through `prepare_generated_output(rendered, existing)` before writing, so:
- newlines are normalized to LF internally, then matched to the existing file's line-ending style,
- re-running a generator with unchanged source produces a byte-identical file (no spurious diffs).

This is the same idempotence guarantee the rule sync pipeline relies on, enabling a `--check` mode in the git hook (sibling git-hook-automation spec).

## R5: TOON sidecar is separate but parallel

The machine-readable `.toon` sidecar is rendered via the `IndexSidecar` codec (`e7a0ee3c`), not through `generated_markdown`. R1–R4 govern the human-readable markdown surface only. Both surfaces are emitted from the same sealed `IndexEntry` set so they never diverge.

## R6: Future structured rendering reuse

Because `render_markdown_file` is generic over snippets + config, future structured rendering for ticket descriptions, spec bodies, or acceptance-criteria summaries reuses the same path by supplying different snippet bodies and a different config. No new rendering stack is needed for those later needs, satisfying the "leave room for future rendering" requirement.

# Scope

- The requirement that store-index markdown routes through `memory-api::generated_markdown`.
- Per-domain provenance config conventions.
- Idempotent write contract enabling hook `--check`.
- Relationship between the markdown surface and the TOON sidecar surface.

# Non-goals

- Does not implement any generator's rendering (per-domain generator tickets).
- Does not change rule entry semantics or the rule sync pipeline.
- Does not define digest normalization rules (digest-input-contract spec).
- Does not define git-hook triggers (git-hook-automation spec).

# Acceptance Criteria

- The track names exactly one rendering paradigm (`memory-api::generated_markdown`) for generated index files (R1).
- The relationship to the existing `rule-api`/`spec-api` rendering pipeline is explicit and shown to be the same shared module (current-state finding).
- Follow-on generator tickets have a concrete integration target: build `GeneratedMarkdownSnippet`s from sealed entries and call `render_markdown_file` + `prepare_generated_output` (R3, R4).
- The plan supports future ticket/spec structured rendering without a redesign (R6).

# Traceability

- Design ticket: [db667eed](.ticket/tickets/db667eed-f507-49ee-b1b6-b7b3edca98ce/ticket.toml)
- Parent spec: generated-context/index-hierarchy-semantic-refs (`18b6a9c5`)
- Depends-on spec: generated-context/thin-generator-architecture (`bf217ce5`)
- Sibling specs: `generated-context/digest-input-contract`, `generated-context/git-hook-automation`
- Sidecar ticket: [e7a0ee3c](.ticket/tickets/e7a0ee3c-dc2f-42dd-8c02-5070a747c156/ticket.toml)
- Shared renderer: `memory-api/crates/memory-api/src/generated_markdown.rs`
- Existing consumers: `memory-api/crates/rule-api/src/render.rs`, `memory-api/crates/spec-api/src/store.rs`
