<!-- aligned-structure:v1 -->

# Summary

Add a shared README schema layer to `rule-api` so repository README targets can inherit a standard structure, enforce required blocks, and expose enough validation detail to fail fast when navigation or command-doc coverage drifts.

## Behavior Story

Add a shared README schema layer to `rule-api` so repository README targets can inherit a standard structure, enforce required blocks, and expose enough validation detail to fail fast when navigation or command-doc coverage drifts.

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

Add a shared README schema layer to `rule-api` so repository README targets can inherit a standard structure, enforce required blocks, and expose enough validation detail to fail fast when navigation or command-doc coverage drifts.

## Problem

The current `rule-targets` model supports explicit node lists plus imported config fragments, but it does not support a reusable README schema. That leaves structurally similar README targets duplicated across workspaces and makes parent/child navigation rules easy to miss.

## Scope

This spec covers:

- shared README schema support in `rule-api`
- README-specific validation rules for required blocks
- explain and sync behavior needed to surface schema failures
- compatibility guarantees for existing targets that do not opt into the new schema

## Intended Behavior

- A README target can opt into a shared schema such as `repository-readme-v1`.
- A consuming target can extend or override parts of the inherited node structure without duplicating the full target.
- Required blocks can include `parent-readme`, `child-readmes`, `installable-content`, and `command-docs`.
- Repo-root targets may omit `parent-readme`; child README targets may not.
- `rule explain-target` and `rule sync-targets --check` can report schema failures deterministically.

## Assumptions To Prove

- Schema or inheritance metadata can be added to the target model compatibly.
- Required-block validation can fail fast without introducing a separate documentation linter first.
- Existing generated README and AGENTS targets remain stable until they explicitly adopt the shared schema.
- Imported child targets preserve their config-relative output semantics after schema support lands.

## Test Strategy

1. Add failing tests for inheritance, override behavior, and missing required blocks.
2. Implement the smallest parser and renderer changes needed to turn those tests green.
3. Validate the feature against representative README targets in root and nested workspaces.

## Acceptance Criteria

- Shared README schema support exists in `rule-api`.
- Missing required README blocks fail mechanically in the target workflow.
- Existing non-README targets continue to work unchanged.

## Traceability

- [9c6fd645 schema tracker](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/9c6fd645-3c50-47f2-b9bd-6de323de0ecc/ticket.toml)
- [ba37c1c6 failing schema tests](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/ba37c1c6-a853-4596-bf91-ab0b02f493ef/ticket.toml)
- [2750018f schema implementation](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/2750018f-ed82-4a3a-9347-1fc47e9658c8/ticket.toml)
