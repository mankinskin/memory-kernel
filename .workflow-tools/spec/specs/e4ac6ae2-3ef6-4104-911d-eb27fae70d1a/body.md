<!-- aligned-structure:v1 -->

# Summary

User and agent feedback is fragmented, hard to query at scale, and not consistently tied to remediation workflows. Teams need an inbox-like store with structured metadata, deep search, and reconciliation loops over large event corpora.

## Behavior Story

User and agent feedback is fragmented, hard to query at scale, and not consistently tied to remediation workflows. Teams need an inbox-like store with structured metadata, deep search, and reconciliation loops over large event corpora.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Problem

User and agent feedback is fragmented, hard to query at scale, and not consistently tied to remediation workflows. Teams need an inbox-like store with structured metadata, deep search, and reconciliation loops over large event corpora.

## Goals

- Add a feedback store that ingests human and privileged-agent feedback events.
- Normalize feedback metadata for scalable filtering, faceting, and deep search.
- Support reconciliation workflows that map raw impressions into actionable change sets.

## Required behavior

### Event model and metadata
- Feedback events capture author role, source channel, scope links, sentiment markers, and confidence metadata.
- Privileged-agent authored events are explicitly marked and auditable.
- Retention and redaction policy is part of the model contract.

### Privacy and retention controls
- retention windows are policy-driven and queryable by event class/source class.
- redaction behavior must preserve audit trace integrity while removing sensitive content.
- incident handling path defines detection, escalation, and post-incident remediation records.

### Explicit schema contract
- FeedbackEvent: `id`, `source_kind`, `source_actor_ref`, `channel`, `title`, `body_ref`, `submitted_at`.
- FeedbackMetadata: `feedback_id`, `sentiment`, `urgency`, `confidence`, `tags[]`, `store_scope`, `workspace_scope`.
- FeedbackLink: `feedback_id`, `artifact_ref`, `validation_ref?`, `interview_session_ref?`.
- FeedbackDisposition: `feedback_id`, `action`, `actor_ref`, `reason`, `acted_at`.

### Query and search
- Search supports keyword, metadata facet, time-window, and sentiment-driven queries.
- Deep search remains performant over large event sets.
- Results expose enough provenance for human audit and automated follow-up.

### Performance expectations
- define SLOs by query class (facet-heavy, full-text-heavy, hybrid).
- define index growth budgets and compaction behavior for sustained ingestion.
- document degradation behavior and operator alerts when SLOs are breached.

### Deterministic ranking and triage
- Query results include explainable score contributions (text relevance, urgency, sentiment pressure, recency).
- Tie-breaking is deterministic and versioned to prevent queue churn.
- Triage queues support configurable weighting profiles with recorded profile version.

### Reconciliation workflows
- Inbox views support triage, dedupe, grouping, and routing to ticket/spec/rule follow-up.
- Reconciliation events can produce structured remediation candidates.
- Feedback-to-action traceability is first-class.

### Governance rules
- privileged-agent feedback cannot auto-route to destructive actions without human disposition.
- duplicate merges must preserve backlink provenance to all merged source events.
- low-confidence feedback cannot directly trigger enforcement actions; it may trigger review recommendations.

### Cross-store signals
- Feedback outcomes can feed store-health metrics and cleanup prioritization.
- Feedback links can target ticket/spec/rule entries using shared reference identity conventions.

## Major risks

- unbounded ingestion without retention policy causes noisy and expensive search
- sentiment metadata drift from inconsistent labeling practices
- feedback loops becoming write-only without structured reconciliation outcomes

## Acceptance criteria

- event schema and indexing strategy are explicit and testable
- search and reconciliation contracts are defined with scalability constraints
- integration boundary with health scoring vector is explicit

## Traceability

- [b1e9e744 [feedback-api] Feedback inbox, metadata indexing, and deep search](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/b1e9e744-aeac-474a-91d9-07e3a362dc76/ticket.toml)
- [9c95c1e4 [feedback-api] Event ingestion, metadata normalization, and retention policy](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/9c95c1e4-3cdb-428e-b9de-800684651226/ticket.toml)
- [b7b84c10 [feedback-api] High-scale search, clustering, and reconciliation workflows](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/b7b84c10-8dc5-4087-87ad-6fe27ebbcd45/ticket.toml)
- [4f86d3d2 [feedback-api] Privileged feedback governance and abuse-boundary enforcement](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/4f86d3d2-2b2a-4c9d-9d46-5f2a437f91b7/ticket.toml)
- [c2d6a14a [feedback-api] Retention, redaction, and privacy incident controls](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/c2d6a14a-98b7-4f98-9f62-90a5ccf06d9e/ticket.toml)
- [3a1ec9f8 [feedback-api] Search latency and index growth SLOs](C:/Users/linus/git/graph_app/context-engine/memory-api/.ticket/tickets/3a1ec9f8-15ea-43f2-b6d3-89b88cbdcb17/ticket.toml)

## Validation

- focused tests for ingestion schema validation and retention/redaction policy
- benchmarked query tests for large feedback corpora with facet + fulltext filters
- integration tests for reconciliation outputs linked into ticket/spec/rule stores
- ranking determinism tests across equal-score and mixed-score datasets
