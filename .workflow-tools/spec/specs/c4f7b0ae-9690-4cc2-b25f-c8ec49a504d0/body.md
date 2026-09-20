<!-- aligned-structure:v1 -->

# Summary

Define the `peek-cli` consumption and level-of-detail (LOD) validation plan for generated store indexes, so generators are reviewed against token-efficient agent consumption, not just correctness. Generated markdown and TOON outputs must be outline-able, greppable, and window-readable without forcing full-file reads.

## Behavior Story

Define the `peek-cli` consumption and level-of-detail (LOD) validation plan for generated store indexes, so generators are reviewed against token-efficient agent consumption, not just correctness. Generated markdown and TOON outputs must be outline-able, greppable, and window-readable without forcing full-file reads.

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

Define the `peek-cli` consumption and level-of-detail (LOD) validation plan for generated store indexes, so generators are reviewed against token-efficient agent consumption, not just correctness. Generated markdown and TOON outputs must be outline-able, greppable, and window-readable without forcing full-file reads.

# Problem

The track has no plan for validating that generated index artifacts are efficiently consumable. Without it, generators could emit correct-but-unscannable files that force expensive full-file reads, defeating the token-efficiency goal of the memory-index work.

# Grounding: peek-cli capabilities

`tools/cli/peek-cli` (`peek`) provides bounded inspection: `--count`, `--grep <pat>` (returns matching line numbers), `--window N` (context around a match), `--start/--end` (line window), `--head/--tail`, `--skeleton` (signatures only), and `--all` (explicit full read). The validation plan exercises generated outputs through these modes.

# Contract

## V1: LOD surfaces the rendering must support

Generated markdown indexes MUST be structured so peek can extract increasing detail tiers without a full read:

- **Tier 0 — digest/header line**: one greppable line per entry carrying `id`, `title`, and `digest` prefix, so `peek --grep` locates any entry and `peek --window` reads just its block.
- **Tier 1 — section summary**: a heading + one-line summary per entry (the normalized `summary` field), reachable by `peek --start/--end` on the entry's block.
- **Tier 2 — full entry detail**: scope, non-goals, keywords, tags, and relation links, only read when explicitly windowed.

The TOON sidecar provides the machine tier: each `IndexEntry` is one record; `peek --grep` on `id`/`digest` locates a record, and the compact encoding keeps full-record reads cheap.

## V2: Cross-domain validation matrix

For each of ticket, spec, rule, audit, workspace, the validation suite proves:

| Check | Markdown README | TOON sidecar |
|---|---|---|
| `peek --count` returns bounded size | yes | yes |
| `peek --grep <id>` locates an entry | yes (Tier 0 line) | yes (record) |
| `peek --window` reads one entry without neighbours bleeding | yes | yes |
| digest inspectable without full read | yes (Tier 0) | yes |

This is the minimum matrix; generator tickets reference it rather than inventing domain-specific checks.

## V3: LOD vs digest/diff stability

LOD rendering is a *presentation* concern and MUST NOT affect the digest. The digest is computed from normalized fields (digest-input-contract spec), not from rendered markdown. Re-rendering at a different LOD tier produces the same `IndexEntry` digest. Diff stability (rendering-pipeline spec R4) still holds: a given LOD tier renders byte-identically across runs for unchanged source.

## V4: Evidence requirement

Each generator ticket must produce a validation artifact (a test or a recorded `peek` transcript) demonstrating the V2 matrix on a representative fixture. The artifact is linked from the generator ticket and from this spec's downstream evidence.

# Scope

- LOD tiers the markdown rendering must expose for peek consumption.
- The cross-domain peek validation matrix (markdown + TOON).
- The invariant that LOD presentation never changes digest or breaks diff stability.

# Non-goals

- Does not implement LOD rendering (generator tickets).
- Does not benchmark runtime performance (benchmarking spec `98bc6b1c`).
- Does not change the digest schema (`0dba399a`).

# Acceptance Criteria

- An explicit peek-cli validation plan exists for generated index artifacts.
- LOD surfaces/tiers are named clearly enough for generator implementers (V1).
- Validation covers both human-readable markdown and TOON sidecar outputs (V2).
- Generator tickets reference the shared validation matrix instead of inventing per-domain checks (V2, V4).

# Traceability

- Design ticket: [d3a95908](.ticket/tickets/d3a95908-fc43-4bbe-9572-998cc61d9102/ticket.toml)
- Parent spec: generated-context/index-hierarchy-semantic-refs (`18b6a9c5`)
- Depends-on specs: generated-context/rendering-pipeline-integration (`9109f12a`), generated-context/digest-input-contract (`449fe68a`)
- peek-cli: `tools/cli/peek-cli`
- Generator tickets: `c5e9bb39`, `b9757ba7`, `9336a096`, `855a1e5d`, `c2409055`, `a72e3aca`
