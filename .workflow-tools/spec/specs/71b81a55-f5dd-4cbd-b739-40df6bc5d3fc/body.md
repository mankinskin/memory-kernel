<!-- aligned-structure:v1 -->

# Summary

Define the **bootstrap-facing curation surface** of the feedback-api program: a URN-addressed, entity-type-agnostic usage-counting and feedback-rating capability that session bootstrapping feeds. Each pin records a usage event, and agents can rate entities before responding. Specs and rules are covered now; tickets use the same model later.

## Behavior Story

Define the **bootstrap-facing curation surface** of the feedback-api program: a URN-addressed, entity-type-agnostic usage-counting and feedback-rating capability that session bootstrapping feeds. Each pin records a usage event, and agents can rate entities before responding. Specs and rules are covered now; tickets use the same model later.

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
Define the **bootstrap-facing curation surface** of the feedback-api program: a URN-addressed, entity-type-agnostic usage-counting and feedback-rating capability that session bootstrapping feeds. Each pin records a usage event, and agents can rate entities before responding. Specs and rules are covered now; tickets use the same model later.

This surface is **owned by the feedback-api program** ([b1e9e744](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/b1e9e744-aeac-474a-91d9-07e3a362dc76/ticket.toml)). There is no separate or parallel "generic memory-api" model — the prior parallel ticket (f8b447b7) was cancelled and folded into the feedback-api program. The session-bootstrap epic depends on the full feedback-api program; this spec captures the consumer-facing contract that program must satisfy.

# Problem
There is no signal for which store entities are actually useful. Rule-entry feedback exists; direct spec feedback is planned; but there is **no usage-frequency counter** and **no single model** spanning entity types. Without this, the session-bootstrap curation loop (frequently-pinned = useful; never-pinned/low-rated = obsolete) cannot function.

# Scope
- A `EntityRef` (URN-addressed `ce://<workspace>/<store>/<entity>`) usage + feedback model delivered by the feedback-api program, independent of the concrete store.
- **Usage counting:** record a usage event for an entity URN (emitted on each session pin). Expose an aggregate count and last-used timestamp per entity.
- **Feedback ratings:** attach `helpful`/`mixed`/`not-helpful` plus optional note to an entity URN, with optional `session_id`/`agent_or_user_id`.
- Query surface: list entities by usage frequency and by low rating / unresolved notes, to drive curation and obsolescence detection.
- Wire spec and rule entities to this model; leave a clear extension point for ticket entities (no ticket wiring required now).

# Non-goals
- The feedback-api program's heavyweight ingestion-at-scale / search-clustering / SLO / abuse-governance slices (tracked separately under b1e9e744's other children).
- UI/dashboards for curation.
- Ticket-entity feedback wiring (extension point only).

# Relationship to existing planning
- Reuses the rule-entry feedback shape (rule feedback already done).
- **Subsumes** direct spec feedback [29bf9628](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/29bf9628-1dc5-4bb4-ae00-b7410dd52db5/ticket.toml) (memory-api store) — spec feedback is delivered through this surface, not built in parallel.
- Implemented by the feedback-api program: tracker [b1e9e744](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/b1e9e744-aeac-474a-91d9-07e3a362dc76/ticket.toml), ingestion [9c95c1e4](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/9c95c1e4-3cdb-428e-b9de-800684651226/ticket.toml).

# Acceptance Criteria (test-validatable)
1. Recording a usage event for an entity URN increments its aggregate count and updates last-used. *(unit test)*
2. The same model records usage for both a spec URN and a rule URN without type-specific code paths. *(unit test over two entity types)*
3. Attaching a rating + note to an entity URN persists and is queryable. *(unit test)*
4. A query returns entities ordered by usage frequency, and a query returns low-rated / unresolved-note entities. *(unit test)*
5. The ticket-entity extension point compiles against the generic model without a concrete ticket binding. *(type-level/compile test)*

# Traceability
- Owned by: feedback-api program ([b1e9e744](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/b1e9e744-aeac-474a-91d9-07e3a362dc76/ticket.toml)); ingestion ([9c95c1e4](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/9c95c1e4-3cdb-428e-b9de-800684651226/ticket.toml)).
- Consumed by: session-bootstrap runtime ([412964a3](C:/Users/linus/git/graph_app/context-engine/memory-api/memory-api/.ticket/tickets/412964a3-e1c3-47da-94ad-268ff20441c0/ticket.toml)) — pins emit usage events; end-of-session ratings use this model.
- Parent: `memory-api/session-api/dynamic-session-bootstrapping`

# Validation
- ValidationSpec: generic usage + feedback unit tests across spec and rule entity types.
- ValidationExecution (planned): `cargo test` for the owning feedback-api crate.
