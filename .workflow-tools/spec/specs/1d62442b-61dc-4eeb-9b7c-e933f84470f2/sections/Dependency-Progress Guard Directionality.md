## Dependency-Progress Guard Directionality

### Normative rule

The dependency-progress guard (`enforce_dependency_progress` in [store.rs](memory-api/crates/ticket-api/src/storage/store.rs#L949)) governs only genuine forward advances, with two unconditional exemptions:

1. **Unconditional exemption — side-transitions.** `cancelled` and `on-hold` are exempt from the guard regardless of `target_rank` vs `current_rank`. Parking and cancelling are side-transitions, not progress claims, and must always be reachable no matter what state a ticket's dependencies are in.
2. **Directional guard — all other target states.** For every other target state, the guard applies only when `target_rank > current_rank` (a genuine forward advance) where rank is the array index of a state in [tracker-improvement.toml](memory-api/crates/ticket-api/schemas/tracker-improvement.toml) (`open=0, planned=1, in-implementation=2, in-review=3, on-hold=4, done=5, cancelled=6`). Demotions and no-ops (`target_rank <= current_rank`) are never blocked by this guard.
3. **Forward advances remain guarded.** When the guard applies, every `depends_on` ticket must be at a state whose rank is `>= target_rank`, else the transition is rejected.
4. **Invariant:** no ticket may be rendered immovable in every direction by this guard. A ticket must always have at least the `cancelled`/`on-hold` side-transitions available, independent of dependency state.

### Background / defect this corrects

- Because `on-hold` (rank 4) numerically outranks `in-review` (rank 3) and `in-implementation` (rank 2) in the schema's state array, a pure directional check (`target_rank > current_rank`) treats parking from `in-implementation` as forward progress and would still guard it — this was insufficient to fix the original defect, since ticket `27558fde` could not be parked from `in-implementation` under a directional-only rule.
- The landed fix instead makes `on-hold` unconditionally exempt, identically to `cancelled`, rather than relying on direction alone.
- The prior implementation (before any fix) compared only `target_rank` against each dependency's rank and never consulted the ticket's own current-state rank, so it had no notion of transition direction at all — any transition landing on a higher-ranked state was guarded, including demotions relative to intent (e.g. `in-review` → `on-hold`).
- Observed on 2026-07-31: 4 tickets (`322a4737`, `5f9542bf`, `79dd2d35`, `6e72756f`) were immovable in both directions because forward moves were blocked by unmet dependencies and backward/park moves were incorrectly blocked by the same guard.

### Known design smell / follow-up (not ticketed)

The `on-hold` exemption is a targeted workaround for `on-hold` being positioned at rank 4 in the linear `states` array in [tracker-improvement.toml](memory-api/crates/ticket-api/schemas/tracker-improvement.toml), even though it is a side-branch state rather than a progress state. Relying on array index conflates "position in a linear sequence" with "is this forward progress," which required a second per-state exemption (`on-hold` alongside `cancelled`) to patch around. The cleaner long-term model is to classify each state as progress vs. side-branch explicitly (e.g. a schema-level flag) instead of inferring intent from array position, which would remove the need for per-state exemptions entirely as new side-branch states are added.

### Acceptance criteria

- `cancelled` and `on-hold` transitions bypass the guard unconditionally, in both directions and regardless of dependency state.
- For all other target states, a transition with `target_rank <= current_rank` never fails due to dependency state.
- For all other target states, a transition with `target_rank > current_rank` continues to fail when any dependency's rank is `< target_rank`.
- No ticket can be rendered immovable in every direction: `on-hold`/`cancelled` remain reachable regardless of dependency state.

### Traceability

- Bug: [5718f77c [ticket-api][workflow] Dependency-progress guard blocks parking and demotion transitions](.ticket/tickets/5718f77c-fea0-47ca-883c-98361a821fb6/ticket.toml)
- Related spec: `agent-tooling/agent-workflow/iteration-loop` (`b71658f1-8de2-444a-9be1-64b1d8ecce70`)
- Validation evidence: `ticket-api` test suite — 207 tests passing, including `update_allows_demotion_and_parking_despite_lagging_dependency` and `update_still_guards_forward_transition_past_lagging_dependency` (both in `memory-api/crates/ticket-api/src/storage/store.rs` test module).