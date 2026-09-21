<!-- aligned-structure:v1 -->

# Summary

Define the cascade that, at `session_init(ticket_id)`, proactively gathers selective context by following **hard ID links** from the ticket to directly related specs, rules, and tickets across stores, and pins them (as URNs) as suggestions.

## Behavior Story

Define the cascade that, at `session_init(ticket_id)`, proactively gathers selective context by following **hard ID links** from the ticket to directly related specs, rules, and tickets across stores, and pins them (as URNs) as suggestions.

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
Define the cascade that, at `session_init(ticket_id)`, proactively gathers selective context by following **hard ID links** from the ticket to directly related specs, rules, and tickets across stores, and pins them (as URNs) as suggestions.

# Problem
An agent starting on a ticket should not have to manually rediscover the spec it implements, the rules scoped to it, and its dependency tickets. Those entities live in different stores, so gathering them requires cross-store resolution. Without a hard-link source and a resolver, the cascade would either do nothing or fall back to noisy semantic matching.

# Scope
- Input: a ticket URN/id at `session_init`.
- Follow **hard ID links only** (D5): ticket→spec via spec↔ticket cross-entity edges; ticket→ticket via `depends_on` edges; rule→ticket/spec via explicit scoped attachment. Emit each as an auto-pin **suggestion** with a `reason`.
- Resolve every related entity to a URN `ce://<workspace>/<store>/<entity>` via the cross-store resolver.
- Return a structured suggestion set the runtime `init_context` persists into `pinned_entities` (each pin still emits a usage event).
- Degrade gracefully: a missing target store or unresolved link is reported as a per-suggestion diagnostic, not a hard failure.

# Non-goals
- Semantic auto-pinning of vague matches (agent-driven, out of scope).
- Creating the hard-link edges themselves (owned by the cross-entity-edge tickets in the memory-api store).
- Implementing the URN resolver (owned by the cross-store reference tickets).

# Dependencies
- **Hard prerequisite — cross-store references:** [82d6ada4 URN resolver](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/82d6ada4-ac35-45a7-9df6-7b7501d58e70/ticket.toml), [6bd67a7a multi-store discovery](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/6bd67a7a-2a76-4dd7-a897-b4d325476621/ticket.toml).
- **Hard-link source (memory-api store, recorded textually — not graph-edgeable from default store yet):** [b03be2d5 cross-entity edges](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/b03be2d5-5293-4dc7-ad11-cca2dbf32c8b/ticket.toml), [f00291a3 ticket↔spec integration](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/f00291a3-bd61-469e-a737-c44cb3911e3b/ticket.toml).

# Acceptance Criteria (test-validatable)
1. Given a ticket with a hard-linked spec, a `depends_on` ticket, and a scoped rule, the cascade returns exactly those three as suggestions with correct `relation`/`reason`. *(fixture-based unit/integration test)*
2. Entities loosely matching by text but **not** hard-linked are **not** suggested. *(negative test)*
3. Every suggestion carries a valid resolvable URN. *(test asserts URN parse + resolve)*
4. A hard link whose target store is missing yields a per-suggestion diagnostic and does not abort the cascade. *(test with absent store)*
5. Suggestions persisted through `init_context` each produce one usage event. *(test)*

# Traceability
- Parent: `memory-api/session-api/dynamic-session-bootstrapping`
- Ticket: [d8f76965 cascade context gathering](C:/Users/linus/git/graph_app/context-engine/memory-api/memory-api/.ticket/tickets/d8f76965-1ff3-4a0a-bb24-773b9637fae4/ticket.toml)
- Builds on: `memory-api/session-api/runtime-session-context`

# Validation
- ValidationSpec: fixture trees with hard-linked + unrelated entities across stores.
- ValidationExecution (planned): `cargo test -p session-api` (and cross-store resolver integration tests once available).
