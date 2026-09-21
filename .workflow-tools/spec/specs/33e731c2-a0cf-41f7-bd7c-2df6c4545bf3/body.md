<!-- aligned-structure:v1 -->

# Summary

Canonical specification for the ticket-viewer navigation surfaces that let a user find and open tickets: the sidebar explorer and the quick-search overlay.

## Behavior Story

Canonical specification for the ticket-viewer navigation surfaces that let a user find and open tickets: the sidebar explorer and the quick-search overlay.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# ticket-viewer: explorer interactions

Canonical specification for the ticket-viewer navigation surfaces that let a user find and open tickets: the sidebar explorer and the quick-search overlay.

## Sidebar explorer

The sidebar explorer combines:

- a free-text query input
- state filter chips
- a sorted ticket list / tree surface
- optional expanded file rows under a ticket

### Filtering contract

1. Query text and state filters combine with AND semantics.
2. Multiple selected states combine with OR semantics.
3. The empty state-filter set means "All states".
4. Explorer persistence includes query text, selected states, sort key, and selected ticket.
5. Free-text terms search ticket titles and description/body content, including partial-word substring matches such as `cracker` -> `Firecracker`.
6. Within a query string, whitespace-separated terms combine with AND semantics, and quoted phrases remain a single term.
7. Supported explicit predicates are `id:<value>`, `title:<value>`, `state:<value>` / `status:<value>`, and `type:<value>` / `ticket_type:<value>`.
8. `title:<value>` searches title content only and still supports substring matches for single terms such as `title:cracker` -> `Firecracker`.
9. The sidebar and quick-search hint copy must advertise only the supported predicate keys and the title/body free-text scope.

### Transport contract

The explorer issues `GET /api/tickets` requests with:

- `workspace=<name>`
- optional `query=<text>`
- zero or more repeated `state=<value>` parameters
- optional `limit=<n>`

Repeated `state` parameters are the canonical multi-state format. A single `state` parameter remains valid as the one-state case.

## Keyboard behavior

### Sidebar explorer

When the sidebar tree itself owns focus:

- `ArrowDown` / `ArrowUp` move between visible ticket rows.
- Phase 1 ticket navigation applies to ticket rows, not expanded file child rows.
- `Enter` opens the focused ticket.
- Text inputs in the explorer own their own keystrokes and suppress row-navigation shortcuts while focused.

### Quick-search overlay

When quick-search is open:

- `ArrowDown` / `ArrowUp` move a highlighted result index through the visible result set.
- `Enter` opens the highlighted result.
- `Escape` closes the overlay.
- Typing in the search box keeps focus in the input and does not trigger background viewer shortcuts.

## Acceptance behavior

- The sidebar can show tickets from multiple workflow states at the same time.
- A query combined with selected states only returns tickets that match the query and at least one selected state.
- If multiple `/api/tickets` requests overlap, only the newest request may update the visible sidebar list, loading state, and error state.
- A unique phrase that appears only in a ticket description/body is discoverable from the sidebar query and the quick-search overlay.
- A partial-word substring query such as `cracker` surfaces tickets containing `Firecracker` in the title or description/body.
- A `title:` predicate such as `title:cracker` surfaces tickets containing `Firecracker` in the title without relying on description/body matches.
- The quick-search overlay must not silently truncate broad backend matches to eight rows; when a query returns more than eight hits, the overlay still surfaces at least the first nine ranked matches and allows scrolling through the list.
- The sidebar and quick-search both expose deterministic visible keyboard focus.
- The visible search hints document the supported predicate keys, quoted-phrase behavior, and conjunction semantics without mentioning unsupported fields.
- Opening a different ticket from the sidebar updates the content-panel body and URL hash for the newly selected ticket; previously rendered description content must not remain visible after the selection changes.
- Reloading the page restores the persisted explorer filter state without corrupting older saved state.
- If `/api/tickets` or `/api/stream` fails, the sidebar keeps rendering its error or empty-state copy without panicking the WASM app.
- After those offline failures, clicking or reconnect retries must not crash the explorer; the route stays interactive and can recover when the backend returns.

## Related shared specs

- `viewer-api/components/tree-view`
- `viewer-api/keyboard-interaction-model`

## Traceability

- Ticket: `.ticket/tickets/fcced2f3-c32c-4533-9743-56543f428222`
- Related ticket: `C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/.ticket/tickets/02723a9b-23ff-47b1-8306-0480be087ddd`
- Related ticket id: `b00b945b-045f-4124-9c69-ea15346b144f`
- Viewer implementation: `memory-viewers/ticket-viewer/frontend/dioxus/src/search_syntax.rs`, `memory-viewers/ticket-viewer/frontend/dioxus/src/components/search/page.rs`, `memory-viewers/ticket-viewer/frontend/dioxus/src/components/ticket_tree/header.rs`, `memory-viewers/ticket-viewer/frontend/dioxus/src/routes/list/page.rs`, `memory-viewers/ticket-viewer/frontend/dioxus/src/routes/list/panels.rs`, `memory-viewers/ticket-viewer/frontend/dioxus/e2e-release/mixed-workspace-root-route.spec.ts`, `memory-viewers/ticket-viewer/frontend/dioxus/e2e-release/sidebar-query-state-filter.spec.ts`
- Viewer docs: `memory-viewers/ticket-viewer/frontend/dioxus/src/components/search.rs`
- Viewer validation passed:
	- `cargo check --manifest-path memory-viewers/ticket-viewer/frontend/dioxus/Cargo.toml --target wasm32-unknown-unknown`
	- `viewer-ctl prepare ticket-viewer && viewer-ctl restart ticket-viewer && node target/tmp/firecracker_probe.mjs`
	- `cargo build -p ticket-viewer --release && viewer-ctl stop ticket-viewer && viewer-ctl install ticket-viewer && viewer-ctl start ticket-viewer && curl http://127.0.0.1:3002/api/tickets?workspace=default&limit=20&query=cracker`
	- `npm run test:e2e:release -- keyboard-navigation.spec.ts -g "sidebar filter keeps focus|documents supported syntax"`
	- `npm run test:e2e:release -- keyboard-navigation.spec.ts -g "documents supported syntax|surfaces at least nine results"`
	- `npm run test:e2e:release -- keyboard-navigation.spec.ts -g "matches partial-word substrings|documents supported syntax|surfaces at least nine results"`
	- `cd memory-viewers/ticket-viewer/frontend/dioxus && npx playwright test ./e2e-release/sidebar-query-state-filter.spec.ts -c playwright.release.config.ts`
	- `viewer-ctl prepare ticket-viewer && cargo build -p ticket-viewer --release && cd memory-viewers/ticket-viewer/frontend/dioxus && npm run test:e2e:release -- mixed-workspace-root-route.spec.ts -g "swaps content panel body when clicking different ticket rows"`
	- Headed Edge visual verification at `1440x900` against the managed `ticket-viewer`: clicked two sidebar ticket rows and confirmed the content panel rendered different description bodies for each selection.
- Viewer validation blocked by an existing unrelated failure in `npm run test:e2e:release -- keyboard-navigation.spec.ts`: `quick-search keeps input focus while arrows move the active result and Enter opens it` still fails because the selected sidebar ticket is not found after opening a search result.
