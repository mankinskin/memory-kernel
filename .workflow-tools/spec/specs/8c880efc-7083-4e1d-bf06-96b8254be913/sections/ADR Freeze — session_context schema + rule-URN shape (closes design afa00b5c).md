This section freezes the architectural decisions owned by design ticket `afa00b5c` so every implementation child has a stable contract to build against. It is the authoritative ADR record for the epic; D1–D9 (frozen body), the R1–R5 refinements, and the Q1–Q5 settlements above remain in force and are referenced here, not restated.

## ADR-1 — session_context.json schema is frozen (ref child spec)
The runtime document schema is **frozen at `schema_version: 1`** as defined in child spec `memory-api/session-api/runtime-session-context` (`709f067a`), section "session_context.json schema (frozen for this slice)". Summary of the frozen contract:
- Top-level: `session_id` (matches capture session id, D1), `schema_version: 1`, `created_at`, `updated_at` (bumped every mutation).
- `pinned_entities` has exactly three buckets: `tickets[]`, `specs[]`, `rules[]`. Each entry is URN-addressed (D2).
- No `current_mode` field (D8). All three buckets empty is a valid general-chat context (D8).
- Entry shapes: tickets `{urn, relation: primary_focus|blocked_by|related, reason?}`; specs `{urn, section?, reason?}`; rules `{urn, filter?, reason?}`.
Any schema change is a new `schema_version` and a spec revision on `709f067a`; implementation children target version 1.

## ADR-2 — rule-URN shape (resolves the last open design item)
Problem: rules historically attach by scope (`repo_scope`/`path_scope`/`section`/`slug`), but pinned references must be stable, addressable URNs. Each rule **entry** already carries a stable id and slug (e.g. `rule-api:entry id=604a8966-… slug=shared/…`).

Decision:
- A pinned rule references a **single rule entry**, never a scope/glob. Canonical URN: `ce://<workspace>/rule/<entry-id>`.
- The resolver also accepts the slug alias `ce://<workspace>/rule/<slug>`; storage normalizes to the entry-id form.
- The optional `filter` field in the `rules[]` bucket is **agent metadata** (the applyTo glob / scope the agent wants to constrain rendering by). It is NOT part of URN identity and is never used for resolution.
- Consistent with D7/R3: always-on `applyTo: "**"` injection is removed, so rules are pinned individually by entry URN (discoverable/pinnable), not auto-attached by scope.

This unblocks the rule bucket of ADR-1 and is the rule-URN shape the cascade re-scope (R2) waits on.

## ADR-3 — store placement & cross-store refs (already settled; recorded here for closure)
Resolved by Q2 (placement = lowest-owning store) and R1 (consolidate via move tooling, not cross-store edges alone). No further design decision required; the URN facade (D2) remains the addressing layer across whatever physical store an entity lands in. Cleanup/move sequencing is owned by `7599ed31` → `505b2cd4`.

## ADR-4 — always-bootstrap / instruction removal (already settled; recorded here for closure)
Resolved by R3 (always-bootstrap confirmed) and D7/D8. Removing the always-on globs is intentional; no design risk remains open.

## Design ticket closure
With ADR-1 (schema), ADR-2 (rule-URN shape), and the recorded closure of ADR-3/ADR-4, all "Remaining design work" items on `afa00b5c` are resolved. The design contract is frozen; implementation children (`412964a3`, `6b2dc497`, `b4a8dc5e`, `c7542933`) and the cross-store prerequisites (`82d6ada4`, `6bd67a7a`) can proceed against this ADR set. The cascade (`d8f76965`) stays provisional per R2 until the hard links are real.