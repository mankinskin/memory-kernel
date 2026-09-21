<!-- aligned-structure:v1 -->

# Summary

Provide a first-class, opt-out workspace policy (`.ticket/workspace-policy.toml`) that governs which nested workspaces are discovered, scanned, and queried. Children are included by default; any child can opt out explicitly (glob or marker); fixture/test workspaces can be denied permanently; external (ancestor/sibling) stores cannot be indexed when `deny_external_paths` is set.

## Behavior Story

Provide a first-class, opt-out workspace policy (`.ticket/workspace-policy.toml`) that governs which nested workspaces are discovered, scanned, and queried. Children are included by default; any child can opt out explicitly (glob or marker); fixture/test workspaces can be denied permanently; external (ancestor/sibling) stores cannot be indexed when `deny_external_paths` is set.

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

Provide a first-class, opt-out workspace policy (`.ticket/workspace-policy.toml`)
that governs which nested workspaces are discovered, scanned, and queried.
Children are included by default; any child can opt out explicitly (glob or
marker); fixture/test workspaces can be denied permanently; external
(ancestor/sibling) stores cannot be indexed when `deny_external_paths` is set.

## Problem

Before this contract there was no explicit allow/deny policy — only default
discovery plus stored scan roots. Fixture/test workspaces (e.g. under
`memory-api/test-fixtures/`) and ancestor/sibling stores could leak into the
indexed ticket graph and inflate audit counts (notably `ticket_graph`
orphan_ticket_count).

## Scope

- Policy model + parser + save + compatibility-mode fallback:
  `memory-api/crates/memory-api/src/workspace_policy.rs`.
- Discovery filtering: `discover_workspace_scan_roots_with_policy` in
  `memory-api/crates/memory-api/src/workspace.rs`.
- Scan-root persistence metadata (`source`, `policy_decision`, `workspace_root`)
  + scan-time skip enforcement + `ScanReport.skipped_roots`:
  `memory-api/crates/memory-api/src/model/filesystem.rs`,
  `.../storage/index.rs`, `.../storage/index/auxiliary.rs`,
  `memory-api/crates/ticket-api/src/storage/store/scan.rs`.
- Query-time final guard tied to policy-allowed roots (`visible_scan_roots` /
  `is_ticket_visible`, applied in `list`, `list_extended`, and `search`):
  `memory-api/crates/ticket-api/src/storage/store/query.rs`.
- CLI surfaces (`ticket workspace policy show|set`, `ignore add|remove`,
  `include add|remove`, `rescan --apply-policy`), forbidden in `batch`:
  `memory-api/tools/cli/ticket-cli/src/cli/...`.

## Non-goals

- Audit-engine scoring internals (the `ticket_graph` trial already honors the
  same policy file and hardcodes `/test-fixtures/` exclusion for reference
  findings).

## Enforcement points (all three consult the policy)

1. Discovery time — before roots are added.
2. Scan time — roots with `policy_decision = ignored` are skipped and reported.
3. Query time — `list`/`list_extended`/`search` only surface tickets under
   active, policy-allowed roots (final defense even for stale index rows).

## Acceptance criteria

- [x] `.ticket/workspace-policy.toml` parsed into an in-memory policy object
  with documented defaults; absent file → compatibility mode + warning.
- [x] Discovery, scan, and query all consult the policy.
- [x] Scan roots persist `source` / `policy_decision` / `workspace_root`.
- [x] CLI can show/set policy, add/remove ignore & include rules, rescan with
  policy applied (reports `skipped_roots`).
- [x] Regression suite proves: child included by default, child ignored via
  marker, ignored via glob, include override wins, external path denied;
  compatibility mode preserved; query guard filters stale ignored-root rows.
- [x] Repo-root policy authored and audit rerun confirms fixture-root leakage
  does not contribute to `ticket_graph` counts.

## Traceability

Implementation tickets (all done):
- `.ticket/tickets/51d53f8f-...` slice 1 — parser + policy object + compat warning
- `.ticket/tickets/6312c5c4-...` slice 2 — discovery filtering
- `.ticket/tickets/eecbcee9-...` slice 3 — scan-root metadata + scan-time enforcement
- `.ticket/tickets/42094bd4-...` slice 4 — query-time final guard
- `.ticket/tickets/c5ff717e-...` slice 5 — CLI/API surfaces
- `.ticket/tickets/25677720-...` slice 6 — regression tests
- Tracker: `.ticket/tickets/65d5885b-...`
- Audit gate parents: `.ticket/tickets/edde88d6-...`, `.ticket/tickets/1a2b326d-...`

## Evidence

- `cargo test -p memory-api -p ticket-api` -> 247 passed (includes all policy
  discovery/scan/query/regression tests).
- `cargo test -p ticket-cli` -> 120 passed (includes `integration_workspace_policy`).
- Repo-root `.ticket/workspace-policy.toml` present; `ticket workspace policy
  show` reports `source: file`.
- `ticket workspace rescan --apply-policy` -> integrated=982, fixture roots
  excluded at discovery (not registered), stale submodule roots flipped to
  ignored (`skipped_roots`).
- `audit run . --json` post-policy: `ticket_graph` findings = 2 (both
  pre-existing dependency-convergence), orphan_ticket_count = 0, fixture
  leakage = 0, policy_excluded_reference_count = 0.
