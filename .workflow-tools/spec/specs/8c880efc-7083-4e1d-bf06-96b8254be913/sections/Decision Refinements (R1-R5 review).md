This section records refinements settled in the R1–R5 risk review. They take precedence over any earlier wording in the frozen body.

## Feedback model — single owner (refines D4/D9; resolves R4)
There is **no separate or parallel usage/feedback model**. The bootstrap-facing curation surface (usage counting + entity ratings, URN-addressed, specs + rules now, ticket extension point) is owned by the **full feedback-api program**:
- The session-bootstrap epic (`effba966`) and runtime (`412964a3`) now `depends_on` the feedback-api tracker `b1e9e744`.
- The prior parallel ticket `f8b447b7` is **cancelled**; its scope folded into feedback-api tracker `b1e9e744` and ingestion `9c95c1e4`.
- Spec `memory-api/curation/entity-usage-and-feedback` (`71b81a55`) is retargeted to component `feedback-api` and now describes the consumer-facing curation contract that program must satisfy.
- Direct spec feedback (memory-api `29bf9628`) is **subsumed** by this surface, not built in parallel.

Sequencing note: the epic depends on the *full* feedback-api program. See Open Decisions for the weight of that gate.

## Cross-store consolidation prerequisite (refines D2; resolves R1)
Edges cannot cross ticket stores, and the ticket-mcp server only exposes the `default` store. Resolution is to **consolidate misplaced entities into one store via move tooling** rather than rely solely on cross-store edges:
- move tooling: memory-api `505b2cd4` (+ children) — safe, journaled, ref-relinking cross-workspace moves.
- cleanup migration: memory-api `7599ed31` `depends_on 505b2cd4` — relocates misplaced `default`-store entities (session-bootstrap, URN, feedback-api tickets) into memory-api.
- hard-link work: memory-api `b03be2d5` / `f00291a3` now `depends_on 7599ed31`.
These are recorded textually in the default-store URN tracker `671d4e47` (cannot be graph-edged across stores).

## Cascade is provisional (resolves R2)
The cascade ticket `d8f76965` is explicitly provisional and carries a REQUIRED refinement note: re-scope it after the hard-link / cross-entity-edge work and the rule-URN shape land. Cascade implementation must not start before the hard links are real.

## Always-bootstrap confirmed (confirms D7/D8; resolves R3)
Removing the always-on `applyTo: "**"` instruction globs is intentional. Agents will be instructed to **always bootstrap**; converted guidance is discoverable/pinnable, not auto-injected. General chat with no relevant entities yields empty pins. This is the intended remedy for always-on bloat.

## Tooling bug gate (resolves R5)
The `update_ticket` state-reset / `transition_states` no-op bug is filed as memory-api `7f4aaa05` with detailed reproduction + test-validatable acceptance criteria. Treat it as a tooling prerequisite for reliable lifecycle automation.

## Open Decisions
1. **Weight of the full feedback-api gate.** The epic depends on the *entire* feedback-api program (including at-scale search/clustering, SLOs, abuse governance). Confirm whether the epic should instead gate on a named "core curation" milestone within feedback-api, or accept the full-program gate as the sequencing cost.
2. **Cross-store URN refs vs consolidation.** Once entities are consolidated into one store via move tooling, decide whether full `ce://` cross-store URN references are still required, or whether intra-store edges + a thin URN facade suffice (drives Phase C scope of `671d4e47`).
3. **Epic state vs prerequisites.** The epic is `in-implementation` while all hard prerequisites (feedback-api, URN, move/cleanup) are still `new`. Only design (`afa00b5c`) is genuinely active. Decide whether to keep the epic as a planning-only tracker or treat the state as a convergence warning to monitor.