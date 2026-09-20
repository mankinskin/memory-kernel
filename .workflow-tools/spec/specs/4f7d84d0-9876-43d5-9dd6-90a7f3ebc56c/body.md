<!-- aligned-structure:v1 -->

# Summary

Migrate `memory-api` from bespoke README target structure to the shared schema and extend its generated tool README surfaces with parent links back to the repo root.

## Behavior Story

Migrate `memory-api` from bespoke README target structure to the shared schema and extend its generated tool README surfaces with parent links back to the repo root.

## Provided Surface Contracts

- README surfaces in scope follow an explicit rule-backed contract instead of one-off rollout prose.
- Parent and child README navigation stays repo-internal and mechanically derivable.
- README completeness and rollout status are verified by mechanical validation rather than manual review.

## Required Validation

- Contract clause validation: The migrated spec names the intended README structure and navigation behavior as explicit contract properties.
- Contract clause validation: The migrated spec names the validation path that checks the README contract mechanically.
- Contract clause validation: The migrated spec records enough evidence to tell whether the README contract is satisfied or blocked.
- The authored spec body documents the README contract, scope boundaries, and navigation expectations for this migration slice.
- The authored spec body documents the mechanical validation path required to prove this migration slice.
- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Summary

Migrate `memory-api` from bespoke README target structure to the shared schema and extend its generated tool README surfaces with parent links back to the repo root.

## Problem

`memory-api` is the cleanest existing README-generation pattern, but it still hard-codes its root target structure and its first-level tool READMEs do not yet participate in a parent-linked repo tree.

## Scope

This spec covers:

- adoption of the shared schema for `memory-api/README.md`
- parent-linked generated README targets for the in-scope CLI, MCP, and HTTP tool surfaces
- preservation of direct command-doc references during the migration

## Intended Behavior

- The root `memory-api` README inherits the shared schema.
- The generated tool READMEs under `tools/cli`, `tools/mcp`, and `tools/http` link back to `memory-api/README.md` through parent blocks.
- The resulting README tree remains fully repo-local and independently generatable.

## Assumptions To Prove

- Existing local rules in `memory-api` can satisfy the shared schema without losing readability.
- Parent links can be added across the generated tool READMEs without changing command coverage.
- The migration can preserve existing imported-child composition behavior.

## Test Strategy

1. Migrate the root `memory-api` README target to the shared schema.
2. Add one representative tool README parent block and validate it.
3. Roll that pattern across the remaining in-scope tool surfaces.

## Acceptance Criteria

- `memory-api/README.md` uses the shared README schema.
- The in-scope generated tool READMEs expose parent links to `memory-api/README.md`.
- `sync-targets --check` passes from the `memory-api` repo root.

## Traceability

- [088c8c40 memory-api rollout ticket](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/088c8c40-7615-486c-88bb-1534902377d1/ticket.toml)
