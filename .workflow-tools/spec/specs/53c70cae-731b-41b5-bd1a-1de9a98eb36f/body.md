<!-- aligned-structure:v1 -->

# Summary

Define the repository-local git-hook automation contract for store-index regeneration so generator tickets have one concrete execution surface instead of each hand-waving at "pre-commit/post-commit hooks". This is repository hook automation under `.githooks/`, NOT Copilot/editor hook automation under `.github/hooks/hooks.json`.

## Behavior Story

Define the repository-local git-hook automation contract for store-index regeneration so generator tickets have one concrete execution surface instead of each hand-waving at "pre-commit/post-commit hooks". This is repository hook automation under `.githooks/`, NOT Copilot/editor hook automation under `.github/hooks/hooks.json`.

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

Define the repository-local git-hook automation contract for store-index regeneration so generator tickets have one concrete execution surface instead of each hand-waving at "pre-commit/post-commit hooks". This is repository hook automation under `.githooks/`, NOT Copilot/editor hook automation under `.github/hooks/hooks.json`.

# Problem

The track requires automatic regeneration of `.ticket/README.md`, `.ticket/index.toon`, `.spec/index.toon`, `.rule/index.toon`, `.audit/index.toon`, workspace summaries, and `.agents/` hook entries when source content changes. No git-hook branch performs this. Existing hook surfaces only cover rule sync, vscode-tasks, docs validation, terminal PWD, and session capture.

# Grounding: existing pre-commit pattern

`.githooks/pre-commit` (bash, `set -euo pipefail`) already follows a reusable shape: collect `git diff --cached --name-only`, a `staged_matches <regex>` guard, regenerate into a temp file, `diff` against the working copy, and on drift print a restage instruction and `exit 1`. The store-index branch MUST follow this exact shape so it composes with the existing checks (rule sync, vscode-tasks).

# Contract

## H1: Execution surface

A new, clearly-delimited branch in `.githooks/pre-commit` owns store-index regeneration. It is distinct from `.github/hooks/hooks.json` (editor/Copilot hooks). The branch is guarded so it no-ops cleanly when a domain's generator entrypoint is not yet wired (see H5 gating).

## H2: Trigger matrix

Per domain, the staged-path patterns that trigger regeneration and the outputs owned:

| Domain | Staged-path trigger (regex) | Generated outputs |
|---|---|---|
| ticket | `^\.ticket/` | `.ticket/index.toon`, `.ticket/README.md` |
| spec | `^\.spec/` | `.spec/index.toon`, spec tree READMEs |
| rule | `^\.rule/` | `.rule/index.toon` |
| audit | `^\.audit/` | `.audit/index.toon` |
| workspace | any of the above store roots | per-store workspace summary entries, `.agents/` hook entries |

## H3: Shared regeneration entrypoint

Each domain exposes one deterministic wrapper command (a CLI subcommand, e.g. `<domain> store-index regen` or a `--check` variant) that the hook invokes. The hook never embeds generator logic; it only dispatches to the domain entrypoint (consistent with the thin-generator-architecture spec). A `--check` mode renders to a temp location and reports drift without writing, mirroring `rule sync-targets --check`.

## H4: Drift behavior

When a domain's staged paths change, the hook regenerates that domain's outputs. If the regenerated output differs from the staged working copy, the hook:
1. writes the regenerated output,
2. prints the touched output paths,
3. prints a restage instruction (`git add <paths>` and re-commit),
4. exits non-zero.

Idempotence (rendering-pipeline spec R4) guarantees a clean re-run after restaging produces no further drift.

## H5: Performance gating and fallback

The pre-commit branch only runs a domain inline when that domain's recorded incremental p95 is within the budget defined by the benchmarking spec (`98bc6b1c` / `c598ddb2`). Otherwise the domain uses a post-commit regeneration fallback, or is explicitly documented as hook-exempt. The per-domain decision is recorded next to the hook branch. Until a domain's generator entrypoint exists and its budget is recorded, the branch leaves that domain disabled (guarded no-op) so commits are never broken by a missing generator.

## H6: Implementation sequencing

The `.githooks/pre-commit` edit lands incrementally: the branch scaffold + trigger matrix + dispatch + `--check` semantics are defined here; each domain is switched from guarded-no-op to active by its generator ticket once that domain's entrypoint (H3) and budget evidence (H5) exist. This avoids invoking non-existent generators and keeps the dependency story coherent (generator tickets depend on this contract, but flip their own domain's hook flag on completion).

# Scope

- The contract for the `.githooks/pre-commit` store-index branch (trigger matrix, dispatch, drift, fallback).
- The separation from Copilot/editor hooks.
- The incremental enablement model gated on per-domain generator + budget evidence.

# Non-goals

- Does not define the semantic digest inputs (digest-input-contract spec).
- Does not redesign `IndexEntry` / `IndexSidecar` (`0dba399a` / `e7a0ee3c`).
- Does not replace docs/session/editor hooks in `.github/hooks/hooks.json`.

# Acceptance Criteria

- `.githooks/pre-commit` has (or will have, per H6) an explicit repository-local branch for store-index generation, scaffolded as a guarded no-op until generators exist.
- The trigger matrix names staged-path patterns and generated outputs per domain (H2).
- Git hooks are clearly distinguished from editor/Copilot hooks (H1).
- On drift the hook regenerates, reports touched outputs, and exits non-zero with a restage instruction (H4).
- The over-budget fallback (post-commit or documented exemption) is defined (H5).
- Profiling evidence per domain is captured via the benchmarking spec before a domain is enabled by default (H5).

# Implementation status

Contract defined. The concrete `.githooks/pre-commit` edit is intentionally deferred to the point where at least one domain generator entrypoint (H3) exists, to avoid wiring the hook to non-existent commands. Each generator ticket activates its own domain's hook branch on completion (H6).

# Traceability

- Design+impl ticket: [52dfd793](.ticket/tickets/52dfd793-6fd4-463f-8c0e-7a8e5c67dd48/ticket.toml)
- Parent spec: generated-context/index-hierarchy-semantic-refs (`18b6a9c5`)
- Depends-on specs: generated-context/thin-generator-architecture (`bf217ce5`), generated-context/rendering-pipeline-integration (`9109f12a`), generated-context/benchmarking-profiling-plan (`c598ddb2`)
- Existing hook: `.githooks/pre-commit`
- Generator tickets that flip their domain flag: `c5e9bb39`, `b9757ba7`, `9336a096`, `855a1e5d`, `c2409055`
