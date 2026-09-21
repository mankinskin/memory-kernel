<!-- aligned-structure:v2 -->

# Motivation

The session store needs useful retrospective ticket attribution without overstating what a recorded signal proves. Root-checkout sessions run on `main` cannot be attributed by branch-shaped capture-time inference, while recorded transcript tool calls contain attributable ticket evidence. The contract distinguishes explicit attribution, consulted tickets, and loose mentions so the session query remains honest about evidence strength.

# Dependent expectation

If this spec is implemented, dependents can rely on `session sessions-for-ticket` and `session_sessions_for_ticket` returning cumulative `strict`, `linked`, and `mentioned` results whose strength is determined by separate persisted fields, rather than treating every transcript occurrence as an explicit work claim.

# Attribution model

The attribution query is cumulative: `strict` results are included by `linked`, and `linked` results are included by `mentioned`. Each tier reads a different persisted field; preserving that field separation is required for the tiers to retain distinct meaning.

| Tier | Stored field read by the query | Evidence class |
| --- | --- | --- |
| `strict` | `metadata.ticket_id` | Explicit session attribution: explicit worktree check-in or a worktree branch that proves the ticket. |
| `linked` | `links.ticket_ids` | Resolved, structured association: capture-request links or transcript-mined ticket tool-call evidence. |
| `mentioned` | handoff record `target_tickets` | Loose association: a handoff declares the ticket as a target; prose-only transcript mentions must not be elevated above this strength. |

# Transcript evidence and write permissions

Transcript mining assigns evidence by source strength:

1. A ticket-mutating tool call, including `update_ticket`, `close_ticket`, `cancel_ticket`, or `board_check_in`, is strong evidence that the session worked on the resolved ticket. The backfill may append the resolved ticket to `links.ticket_ids` only.
2. A ticket-reading tool call, including `get_ticket`, `get_ticket_description`, `list_parts`, or `subgraph`, is moderate evidence that the session consulted the resolved ticket. The backfill may append the resolved ticket to `links.ticket_ids` only.
3. A ticket found only in loose prose text is weak evidence that the session mentioned the ticket. The backfill must preserve that evidence at `mentioned` strength only; it must not append the ticket to `links.ticket_ids` or write `metadata.ticket_id`.

**Binding constraint:** no transcript-mined evidence, including a mutating or board tool call, may write `metadata.ticket_id`. `metadata.ticket_id` remains reserved for explicit check-in attribution or worktree-branch proof. A mined guess in the strict field would collapse the distinction between the three tiers.

# Candidate resolution and persistence

The backfill accepts full ticket UUIDs and 8-hex candidate prefixes extracted from transcript evidence. Each candidate must resolve uniquely against the ticket store before persistence. Candidates that do not resolve are discarded silently and are never written to any session artifact. This resolution filters common unrelated 8-hex values such as git hashes and run identifiers; an accidental random 8-hex collision with a real ticket prefix has probability on the order of $10^{-7}$.

The backfill is dry-run by default. Persisting changes requires an explicit `write` flag. A write run is idempotent: rerunning the same backfill must not duplicate any ticket id already present in `links.ticket_ids`.

# Root-checkout limitation and non-goal

Capture-time worktree inference only attributes sessions when the branch shape proves a ticket. A session run in the root checkout on `main` does not meet that condition, so capture-time inference correctly writes no ticket attribution for such a session. Retroactive backfill is the only mechanism in scope that can attribute existing root-checkout sessions.

Passing session context into ticket-API calls so future sessions self-attribute is a non-goal. That idea may be considered separately and is not a requirement of this spec.

# Guards

No ValidationSpec guard exists yet. Until a validation guard is created, review evidence must be a before/after read-back of the affected `.session/sessions/<session-id>/session.json`, `transcript.json`, and handoff artifacts, with the invoked dry-run or write command recorded alongside the artifact paths.

# Positions

- `memory-api/crates/session-api/src/store/config/worktree_capture_inference.rs` — `implemented`: writes strict attribution only when a ticket-resolving worktree branch proves the ticket; root-checkout `main` sessions are intentionally inapplicable.
- `memory-api/crates/session-api/src/store/config/ticket_backfill.rs` — `partial`: structured branch and handoff backfill exists; transcript evidence-tier extraction and the strict-field prohibition require implementation alignment.
- `memory-api/tools/cli/session-cli/src/lib.rs` — `implemented`: backfill is exposed with a dry-run/default and explicit write mode; the resulting persisted-field behavior remains subject to this contract.

# Governing-rule requirement

A PolicyRule that introduces session-attribution guidance in-session must cite this spec and its `partial-with-gaps` readiness. No applicable PolicyRule identifier was available to this spec-authoring session; creation or linkage of that rule is required before the contract is presented as fully governed.

# Acceptance criteria

1. After a dry-run, reading every affected `session.json` confirms `metadata.ticket_id` and `links.ticket_ids` are unchanged from the pre-run artifacts.
2. After a write run, reading each affected `session.json` confirms every new `links.ticket_ids` entry resolves to an existing `.ticket/tickets/<uuid>/ticket.toml` artifact.
3. After a write run, reading each session artifact attributed only through transcript tool-call evidence confirms `metadata.ticket_id` remains absent or unchanged from its pre-run explicit/branch-proven value.
4. After a write run, reading a session artifact with a resolved mutating or reading ticket-tool argument confirms the ticket appears once in `links.ticket_ids`, and reading the corresponding `transcript.json` confirms the supporting tool call and arguments.
5. After a write run, reading a session artifact whose candidate does not resolve in the ticket store confirms the candidate appears in neither `metadata.ticket_id` nor `links.ticket_ids`.
6. After rerunning the same write backfill, reading each previously affected `session.json` confirms `links.ticket_ids` contains no duplicate ticket ids and has the same set of values as after the first write.
7. For a root-checkout session whose stored `metadata.worktree.branch` is `main`, reading `session.json` confirms capture-time inference has not populated `metadata.ticket_id`; any retrospective association is present only in the allowed non-strict tier.

# Traceability

- Ticket: [1d5f58eb Backfill session-to-ticket attribution from transcript tool-call signals](.ticket/tickets/1d5f58eb-1ab3-4a9e-8388-f5177a0bac98/ticket.toml)
- Related spec: [36fd7849 session-api hook ingestion and read query](../36fd7849-65eb-405e-8cc5-70440f0cb7c2/spec.toml)
- Required evidence: stored-artifact read-back from the session store and ticket store after dry-run and explicit-write backfill execution; compilation and test success alone are insufficient evidence for captured or persisted data.

# Non-goals

- Changing capture-time branch inference for root-checkout sessions on `main`.
- Passing session context into ticket-API calls for prospective self-attribution.
- Writing transcript-mined evidence into `metadata.ticket_id`.
- Treating unresolved 8-hex candidates as ticket associations.