<!-- aligned-structure:v1 -->

# Summary

`ticket board show`, `ticket next`, MCP `board_show`/`next_tickets`, and HTTP `/api/workflow/next` expose inconsistent, narrowly scoped discovery knobs. This spec defines the canonical, reusable **selector contract** that all workflow discovery surfaces must honour.

## Behavior Story

`ticket board show`, `ticket next`, MCP `board_show`/`next_tickets`, and HTTP `/api/workflow/next` expose inconsistent, narrowly scoped discovery knobs. This spec defines the canonical, reusable **selector contract** that all workflow discovery surfaces must honour.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Summary

`ticket board show`, `ticket next`, MCP `board_show`/`next_tickets`, and
HTTP `/api/workflow/next` expose inconsistent, narrowly scoped discovery knobs.
This spec defines the canonical, reusable **selector contract** that all
workflow discovery surfaces must honour.

**Tracking ticket:** [790df512 Specify scoped selector contract for board and next](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/790df512-d8a9-42bd-b3d6-6e2b4d5eda9c/ticket.toml)
**Convergence tracker:** [cf4246c3 Track workflow and health surface convergence](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/cf4246c3-6539-4f1c-a876-6d34073db7b3/ticket.toml)

---

# Selector Axes

Every workflow-discovery surface (`next`, `board show`) MUST support the following
named selector dimensions. All dimensions are optional and independently combinable.

## 1. Workspace / Index-Root

- **Field name (JSON):** `workspace`
- **CLI flag:** `--workspace <name>` (resolves via registered workspace labels)
- **Semantics:** narrows discovery to a single registered ticket root. When
  omitted the active root is used. Multi-root aggregation is a future extension
  and is not part of this contract.
- **Machine-readable output field:** `scope.workspace` (string) — the resolved
  workspace label used for this query.

## 2. Title-Prefix Filter

- **Field name (JSON):** `filter`
- **CLI flag:** `--filter <prefix>`
- **Semantics:** retains only tickets whose title starts with the prefix string
  (case-sensitive). This is the existing `--filter` behaviour, made explicit.
- **Backward compat:** preserved as-is; no migration required.

## 3. Graph-Root Scope

- **Field name (JSON):** `root`
- **CLI positional / flag:** `root <id>` (existing CLI positional for `next`)
- **Semantics:** limits candidates to the set of unresolved prerequisite tickets
  that remain in the reverse-dependency tree rooted at the given ticket. When
  provided, the response MUST include `scope.root` with the resolved UUID.
- **Backward compat:** preserved. CLI `ticket next [root]` continues to work.

## 4. Field Predicates *(Phase 1 extension — not in Phase 0)*

- **Field name (JSON):** `where`
- **CLI flag:** `--where key=value` (repeatable, intersection semantics)
- **Semantics:** filter by indexed fields (`component`, `state`, `type`, `tags`).
  This axis is specified here for naming stability but is NOT implemented in
  Phase 0. Surfaces that add it later MUST use these field names.

---

# Composition Semantics

1. All active selector dimensions are applied as **intersection** (AND).
   A ticket must satisfy every supplied dimension to appear in results.
2. If no selector dimensions are supplied, results are repository-wide for the
   active workspace.
3. An empty result set is valid and MUST be returned as `count: 0, items: []`.
   It is NOT an error.
4. Dimension omission means "no constraint on this axis", not "empty set".

---

# Machine-Readable Scope Metadata

Every workflow-discovery response MUST include a top-level `scope` object with
at minimum:

```jsonc
{
  "scope": {
    "workspace": "default",        // resolved workspace label (always present)
    "active_index_root": "/abs/path/to/.ticket",  // resolved store root path
    "filter": null,                // title prefix applied, or null
    "root": null                   // UUID string if a root was specified, else null
  }
}
```

**Rationale:** frontends (ticket-viewer, ticket-vscode) and downstream tools
must not reconstruct which store or scope produced a result; the producing
surface must declare it explicitly.

---

# Backward Compatibility Rules

| Existing input | Status | Notes |
|---|---|---|
| `ticket next [root]` | Preserved | Root positional stays; also exposed as `scope.root` in JSON |
| `ticket next --filter` | Preserved | Also reflected as `scope.filter` in JSON |
| MCP `next_tickets.workspace` | Preserved | |
| MCP `next_tickets.filter` | Preserved | |
| HTTP `?filter=`, `?root=` | Preserved | |

No existing field is renamed or removed. New fields are additive.

When board state hides otherwise actionable tickets, `next` surfaces MUST keep
the visible `items` list board-aware and surface the hidden tickets separately
in `excluded_by_board`, alongside any board warnings needed to explain the
filtered result. HTTP Phase 1+ must match the same behavior already used by CLI
and MCP rather than returning the raw unfiltered candidate set.

---

# Surfaces in Scope (Phase 0)

| Surface | Change |
|---|---|
| CLI `ticket next` | Add `scope` object to JSON output |
| CLI `ticket board show` | Add `scope` object to JSON output |
| MCP `next_tickets` | Add `root` input param; add `scope` object to response |
| MCP `board_show` | Add `scope` object to response |

HTTP `/api/workflow/next` is Phase 1 (depends on `0e375356`).

---

# Regression Matrix

| Scenario | CLI | MCP | HTTP |
|---|---|---|---|
| Workspace-only query | scope.workspace correct | scope.workspace correct | Phase 1 |
| root-scoped next | scope.root = resolved UUID | scope.root = resolved UUID | Phase 1 |
| filter-scoped next | scope.filter = prefix | scope.filter = prefix | Phase 1 |
| No selector (wide) | scope fields null | scope fields null | Phase 1 |
| board-aware next exclusion | filtered `items`, `excluded_by_board`, `warnings` | filtered `items`, `excluded_by_board`, `warnings` | Phase 1 parity with CLI/MCP |
| board show scope | scope.active_index_root set | scope.active_index_root set | Phase 1 |

---

# Related Tickets

- [790df512 Specify scoped selector contract for board and next](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/790df512-d8a9-42bd-b3d6-6e2b4d5eda9c/ticket.toml) — this spec ticket
- [68a08b34 Scope-aware board and next for multi-root workspaces](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.workflow-tools/ticket/tickets/68a08b34-000b-4585-8354-4b1a26a15f4b/ticket.toml) — CLI and MCP scope-aware selector rollout
- [0e375356 Implement scoped selectors for board and next](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/0e375356-b74e-48c4-8f1d-77cd28e055bc/ticket.toml) — Phase 1 selector surface across CLI, MCP, and HTTP
- [c031aeb0 Define minimal workflow and health core plus adapter responsibilities](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/c031aeb0-f374-4d57-9d46-2463dfa8571d/ticket.toml) — Phase 2 shared ticket-api workflow and health core
- [6484d4b7 Build larger-integration parity routine for workflow and health surfaces](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/6484d4b7-e24b-4c13-999c-d0b00928d97c/ticket.toml) — Phase 3 parity fixture and cross-interface regression suite
- [4a48b371 Unify board-aware next filtering across workflow surfaces](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/4a48b371-7dc0-4bf2-badb-747a8f00a0fc/ticket.toml) — follow-up parity fix aligning HTTP with the shared board-aware next contract

# Validation Evidence

Automated review evidence for tracker closeout:

- `cargo test -p ticket-api apply_board_filter -- --nocapture`
- `cargo test -p ticket-http --test integration_parity -- --nocapture`
- `cargo test -p ticket-mcp next_tickets_excludes_board_active_and_surfaces_wip_warning -- --nocapture`

Audit outcome:

- ticket health for [cf4246c3 Track workflow and health surface convergence](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/cf4246c3-6539-4f1c-a876-6d34073db7b3/ticket.toml) returned zero findings.
- spec health and refs validation for this spec returned `ok` with zero issues.
- Focused review of the shared backend and HTTP/MCP adapter call sites found no undocumented workflow or health drift after the board-aware next follow-up.
