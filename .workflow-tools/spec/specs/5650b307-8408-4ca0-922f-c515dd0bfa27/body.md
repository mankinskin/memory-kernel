<!-- aligned-structure:v1 -->

# Summary

Canonical specification for the ticket-viewer shell-level interaction contract that sits above the existing explorer and theme-settings specs.

## Behavior Story

Canonical specification for the ticket-viewer shell-level interaction contract that sits above the existing explorer and theme-settings specs.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# ticket-viewer: shell and header actions

Canonical specification for the ticket-viewer shell-level interaction contract that sits above the existing explorer and theme-settings specs.

This spec covers:

- the sidebar tree container and its long-list reachability behavior
- the list-route header action set
- how ticket-viewer aligns shared shell primitives with spec-viewer-style navigation affordances

It does not redefine explorer query/filter semantics from `ticket-viewer/explorer` or theme-settings content from `ticket-viewer/theme-settings`.

## Single-process startup contract

1. Starting `ticket-viewer` against a checkout that already contains local `.ticket/tickets/` manifests but lacks SQLite index artifacts MUST bootstrap or rebuild the local ticket index before serving routes.
2. That startup bootstrap MUST preserve existing ticket manifests already on disk; missing `tickets.db` is a recoverable local-state condition, not a fatal panic condition.
3. If store initialization still fails after bootstrap, the binary MUST return a contextual startup error instead of panicking via `expect(...)`.

## Sidebar tree container

1. The ticket-viewer sidebar tree is a primary navigation surface and MUST keep long lists reachable via wheel, trackpad, touchpad, and keyboard-assisted scrolling.
2. The sidebar tree container MUST preserve a stable scroll position when a user opens a ticket from deeper in the list and then returns focus to the sidebar.
3. The ticket-viewer sidebar SHOULD reuse the same shared container and tree-navigation primitives used by spec-viewer where that does not break ticket-specific file rows.
4. If ticket-viewer cannot literally reuse the same component surface, it MUST still match the spec-viewer contract for scroll reachability, focus visibility, and expand/collapse behavior.

## List-route header actions

1. The ticket-viewer list route MUST render only actions that have an immediate, meaningful effect in that route.
2. The header MUST expose a working theme-settings action.
3. The ticket-viewer list route MUST NOT render a home action when it resolves to the current route without changing state.
4. The ticket-viewer list route MUST NOT render a filter-toggle action when all ticket filtering is already owned by the visible sidebar explorer.
5. If the shell renders an info/help action, clicking it MUST open a visible ticket-viewer help/about surface. A non-functional decorative info icon is forbidden.
6. The visual affordance for theme settings MUST be clear enough that it is not mistaken for a dead info button.

## Execution plan

### Phase 0 — baseline and parity check

Before implementation, refresh the ticket-viewer artifact path that is actually under test and confirm the currently running managed viewer matches source expectations.

1. Refresh the static frontend bundle before browser validation with `viewer-ctl prepare ticket-viewer`.
2. If any server-side route, shell wiring, or embedded API behavior changes, rebuild and reinstall the managed server before validation.
3. Confirm whether the visible header controls in the managed viewer are current source behavior or stale installed artifacts.
4. Capture the current long-list sidebar behavior and current header action set before changing code.

### Track A — header actions cleanup

Primary ticket: `C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/6ea2c97c-0b41-4b90-91db-f0de9e8e4b8e`

1. Start at `memory-viewers/ticket-viewer/frontend/dioxus/src/routes/list/page.rs` and the shared `PageHeader` / `HeaderActions` surface.
2. Keep the route-level hamburger lead control.
3. Keep a working theme-settings affordance.
4. Remove header actions that are no-ops on the root list route.
5. If an info/help affordance remains, it must open a visible ticket-viewer help/about surface.
6. If ticket-viewer needs a different shared header semantic, coordinate the minimal upstream change with `C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/.ticket/tickets/bb1c32f5-5275-4e4f-85ae-a0fba09c522a`.

### Track B — sidebar tree parity and long-list scrolling

Primary ticket: `C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/c10cc92e-03b5-423b-a7ef-93879c253f7d`

1. Start at `memory-viewers/ticket-viewer/frontend/dioxus/src/components/ticket_tree/page.rs`, `rows.rs`, and the shared `Sidebar` / layout CSS.
2. Diagnose local explorer-body and row-stack overflow behavior before changing shared `Sidebar` semantics.
3. Preserve ticket-specific expanded file rows while restoring long-list reachability.
4. Add focused-row visibility or scroll restoration where keyboard navigation and deep-row selection need it.
5. Keep the parity target behavioral first: spec-viewer-like reachability, visibility, and stability, not a forced component rewrite.

### Track C — existing explorer hardening follow-up

Related tickets:

- `C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/a2f5460c-1e7e-481b-a250-e9def213ba55`
- `C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/0b7da330-a38d-49e5-853c-cf1d40633b6f`
- `C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/f121b24b-61b0-41b4-9567-8ffc2417d7cb`
- `C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.ticket/tickets/cccf5d99-d7e9-43e6-8aea-90480ad3cf0d`

1. Keep query/state semantics, multi-state chips, and keyboard navigation in their existing tickets.
2. Tracks A and B may run in parallel with this explorer work, but should not absorb those semantics into the shell tickets.
3. Shared helper extraction is allowed only after the shell-level behavior is stable enough to avoid reopening multiple slices at once.

### Final integration

1. Re-run focused release Playwright coverage for the touched shell and explorer flows.
2. Re-run external headed Chromium-family browser verification against the managed ticket-viewer.
3. Update the relevant ticket/spec summaries with implementation, validation, and documentation status before moving tickets to `in-review`.

## Rigorous test design gate

No shell or explorer implementation should begin until the responsible ticket records a concrete test design.

1. Each ticket MUST document a pre-implementation test design in its description or linked asset before moving to `in-implementation`.
2. The test design MUST capture baseline reproduction steps for the current failure, the exact route under test, and the selectors or test ids the validation will rely on.
3. The test design MUST enumerate happy-path, regression, and edge-case coverage before code changes begin.
4. Browser-facing work MUST include at least one focused release Playwright assertion and one external headed Chromium-family browser verification step.
5. Shared-contract changes to `PageHeader`, `HeaderActions`, `Sidebar`, or shared CSS MUST update cross-viewer assertions, not only ticket-viewer-local tests.
6. The validation design MUST distinguish frontend-bundle refresh requirements from managed-server rebuild/reinstall requirements so stale artifacts do not invalidate results.
7. Existing failing tests MUST be classified as same-slice blockers or unrelated background failures before implementation proceeds.

### Minimum shell test design requirements

#### Header-action work

- Define the exact final header-action matrix for the ticket-viewer root list route.
- Update the shared header assertions in `memory-viewers/ticket-viewer/frontend/dioxus/e2e-release/shared/common-viewer-suite.ts` to match the intended contract.
- Keep or update the theme-settings open/close assertions in `memory-viewers/ticket-viewer/frontend/dioxus/e2e-release/shared/dioxus-theme-suite.ts`.
- If an info/help action exists, add an explicit assertion for its visible effect.

#### Sidebar/tree work

- Prove that lower rows in a long sidebar list are reachable and interactive.
- Cover both pointer scrolling and keyboard-assisted movement to lower rows.
- Cover deep-row selection without losing useful sidebar position.
- Include at least one narrow manual check at desktop width and one narrow check at mobile-sidebar width.

## Related specs

- `ticket-viewer/explorer`
- `ticket-viewer/theme-settings`
- `spec-viewer/spec-tree`

## Traceability

- Ticket: `memory-viewers/.ticket/tickets/d7a27192-6c67-4446-9450-c946bf58747e`
- Viewer implementation: `memory-viewers/ticket-viewer/src/main.rs`
- Viewer validation passed:
	- `cargo test -p ticket-viewer --bin ticket-viewer startup_bootstraps_manifest_only_ticket_store -- --nocapture`
	- `./target/debug/ticket-viewer.exe --port 0` (started successfully and printed `TICKET_VIEWER_PORT=59348`)
- Viewer validation blocked by an existing unrelated failure in `cargo test -p ticket-viewer`: `memory-viewers/ticket-viewer/tests/workspace_resolution.rs` imports removed `ticket_api::workspace` symbols and fails before the full crate test suite completes.
- Planned ticket: C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/c10cc92e-03b5-423b-a7ef-93879c253f7d
- Planned ticket: C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/6ea2c97c-0b41-4b90-91db-f0de9e8e4b8e
- Related existing ticket: C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/a2f5460c-1e7e-481b-a250-e9def213ba55
- Related existing ticket: C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/0b7da330-a38d-49e5-853c-cf1d40633b6f
- Related existing ticket: C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/f121b24b-61b0-41b4-9567-8ffc2417d7cb
- Related existing ticket: C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.ticket/tickets/cccf5d99-d7e9-43e6-8aea-90480ad3cf0d
- Related upstream ticket: C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/.ticket/tickets/bb1c32f5-5275-4e4f-85ae-a0fba09c522a
