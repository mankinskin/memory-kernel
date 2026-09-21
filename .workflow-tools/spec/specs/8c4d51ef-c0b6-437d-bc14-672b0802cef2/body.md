<!-- aligned-structure:v1 -->

# Summary

The ticket-viewer main layout still splits ticket reading across separate content and detail panels, while graph mode fetches a fixed-depth root-local subgraph and treats selection changes like graph replacements instead of focus changes. The result is a fragmented detail experience and a graph surface that does not scale to full-workspace navigation.

## Behavior Story

The ticket-viewer main layout still splits ticket reading across separate content and detail panels, while graph mode fetches a fixed-depth root-local subgraph and treats selection changes like graph replacements instead of focus changes. The result is a fragmented detail experience and a graph surface that does not scale to full-workspace navigation.

## Provided Surface Contracts

- The ticket-viewer main reading surface is a single content panel that contains the selected ticket title, editable fields/actions, summary metadata, and contextual content in one document flow.
- The separate right-side detail panel is removed from the default experience; features previously only available there are preserved inside the content panel.
- The content-panel header remains compact and avoids full-width preface blocks before the primary body.
- Non-essential metadata is rendered in a footer section so the core document remains scannable.
- The content panel exposes immediate parent and child neighbours for the selected ticket.
- Parent/child neighbour items are clickable and retarget the active selection.
- Neighbour retargeting updates graph focus from client-side cached graph data without per-click graph refetch.
- Graph selection and neighbour-driven selection stay behaviorally consistent (same selected ticket, same focus retarget semantics).

## Required Validation

- Playwright release coverage validates that default ticket reading uses one consolidated content panel and no standalone right detail panel is required for core read/edit flows.
- Playwright release coverage validates compact header behavior and metadata footer placement for selected tickets with representative field density.
- Playwright release coverage validates parity of actions/edit capabilities between the pre-change detail experience and the consolidated panel (no functional regressions).
- Playwright release coverage validates parent/child neighbour lists in the content panel and confirms each neighbour click retargets selection to the expected ticket.
- Playwright release coverage validates neighbour-click retargeting reuses cached graph data and does not issue a network fetch for per-click graph expansion.
- Focused browser validation in external Chromium confirms navigation clarity, readability, and retarget consistency in both split and content-only layouts.
- Ticket traceability for this spec is maintained through W1 `a34d9891-5cd3-403c-8576-f3a55a50047e` and W4 `1b0dbe8a-1828-41e7-b33f-4a066c3622bf`.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Problem

The ticket-viewer main layout still splits ticket reading across separate content and detail panels, while graph mode fetches a fixed-depth root-local subgraph and treats selection changes like graph replacements instead of focus changes. The result is a fragmented detail experience and a graph surface that does not scale to full-workspace navigation.

# Goals

- make the selected ticket read like one compact integrated document in the main layout
- keep the workspace graph visible while selection changes move focus through the existing graph
- make dependency hierarchy read cleanly with mostly planar isometric defaults
- add a kanban-style table layout that groups tickets by workflow state and dependency-related subcomponents
- add node detail tiers so dense graphs stay legible when zoomed out

# Requirements

## Integrated ticket document

- the primary reading surface combines title, key metadata, description, and contextual asset content
- edit and history controls remain available without fragmenting the default reading experience into separate panels
- the integrated document remains usable beside the graph in split mode and in content-only mode

## Focused full-graph navigation

- graph mode can render a workspace-scoped graph payload instead of only a fixed-depth root subgraph
- switching selection changes focus, pan, and emphasis within the existing dataset rather than replacing the graph with a new local fetch
- reapplying the same graph payload preserves user-adjusted node positions instead of snapping back to the backend layout
- switching focused nodes retargets the camera without resetting the user-adjusted zoom level
- related nodes and edges are emphasized while distant unrelated regions can fade or cull enough to preserve readability

## Layout defaults and settings

- dependency hierarchy should read from parent to child consistently
- default placement should prefer a mostly planar arrangement suitable for isometric viewing instead of deep overlapping stacks
- graph settings should also expose a kanban or table layout mode that maps ticket states to stable columns
- the kanban columns should leave enough horizontal spacing that adjacent workflow lanes, cards, and headers do not crowd each other
- the kanban layout should organize nodes into vertically ordered dependency-group rows so related subcomponents stay clustered within each state column
- each kanban cell may contain multiple nodes, but those nodes should remain separately readable through soft clustering rather than exact overlap
- the kanban layout should render visible state-column headers and separators so workflow lanes remain readable without inferring the table structure from node placement alone
- dependency-group rows should render visible row labels so viewers can scan clustered branches without opening each node first
- kanban headers and row labels should scale from camera distance and visible node sizing so they stay readable when zoomed in, avoid overlapping nearby nodes or each other, and shrink to compact sizes and line heights when the graph is zoomed far out
- graph settings should expose the main hierarchy and spacing parameters required to tune this view

## Node level of detail

- node rendering supports several detail tiers, from minimal particles or dots through compact and rich ticket presentations
- zoom level, focus, and visibility budget can change which tier is used
- focused nodes may remain richer than distant nodes at the same zoom level

# Planned work

Tracker: [05dae5fd [ticket-viewer][ticket-http][viewer-api] Improve main layout ticket documents and focused full-graph navigation](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/05dae5fd-1a1d-4a64-be62-f29ca0771a4d/ticket.toml)

1. [8f5d611f [ticket-viewer] Build integrated ticket document panel](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/8f5d611f-0033-423e-b2f6-17683feb8e34/ticket.toml)
2. [60092819 [ticket-viewer] Fix graph layout defaults and isometric settings](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/60092819-f725-48ec-93f0-aba195ef81eb/ticket.toml)
3. [397fa45b [ticket-http][ticket-viewer] Expose workspace graph payload for focused full-graph navigation](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/397fa45b-a0bd-43d2-b430-2dfa44d80c5c/ticket.toml)
4. [6e7a15c9 [ticket-viewer] Keep full workspace graph visible with focused navigation](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/6e7a15c9-d8e6-4bbe-bb34-b83bd651896b/ticket.toml)
5. [322ba030 [viewer-api][ticket-viewer] Add multi-level graph node detail rendering](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/viewer-api/.ticket/tickets/322ba030-160c-41d3-8a12-42936ae92858/ticket.toml)
6. [1f39ba8f [ticket-viewer] Add graph review E2E coverage](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/1f39ba8f-650b-417d-b664-1878f08af669/ticket.toml)
7. [800f09ed [ticket-viewer][viewer-api] Tighten graph layout and enlarge rich nodes](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/800f09ed-beb0-4a12-be93-1392e45eadb8/ticket.toml)
8. [d1d38010 [ticket-viewer][viewer-api] Preserve graph layout and camera across same-graph refreshes](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.ticket/tickets/d1d38010-08b8-4a06-ad2b-0bbed453c941/ticket.toml)
9. [eeda4039 [ticket-viewer][viewer-api] Add kanban table graph layout mode](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.ticket/tickets/eeda4039-d82d-4573-9d79-0bc89e152a76/ticket.toml)

# Implementation traceability

Completed first slice: [8f5d611f [ticket-viewer] Build integrated ticket document panel](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/8f5d611f-0033-423e-b2f6-17683feb8e34/ticket.toml)

- content mode now renders a single document-focused surface while the separate detail inspector remains available only in split mode
- the integrated document header, metadata grid, generic extra-field section, and inline asset context live in [panels.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/routes/list/panels.rs), [page.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/components/ticket_content/page.rs), and [render.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/components/ticket_content/render.rs)
- the integrated document now resolves selected-ticket summaries by id, prefers the resolved owning ticket reference for mixed-workspace document fetches, and rehydrates both the document surface and the TOML tab from fetched ticket detail fields in [panels.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/routes/list/panels.rs), [page.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/components/ticket_content/page.rs), and [render.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/components/ticket_content/render.rs), so mixed-workspace tickets keep their title, key metadata, and remaining top-level ticket.toml fields inline instead of requiring the raw TOML tab
- focused browser coverage for the document surface, extra seeded field rendering, and asset-context path lives in [mixed-workspace-root-route.spec.ts](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/e2e-release/mixed-workspace-root-route.spec.ts)

Completed second slice: [60092819 [ticket-viewer] Fix graph layout defaults and isometric settings](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/60092819-f725-48ec-93f0-aba195ef81eb/ticket.toml)

- the local graph layout in [layout.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/layout.rs) now places hierarchy layers as parent-anchored horizontal bands with fixed layer spacing and bounded z staggering instead of circular xz rings, so parent-to-child depth reads top-to-bottom with less overlap
- graph defaults in [page.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/routes/list/page.rs) and [graph3d.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/graph3d.rs) now start in hierarchical 3D with orthographic projection and a mostly planar isometric reset angle
- focused graph browser coverage in [graph-detail-sidebar.spec.ts](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/e2e-release/graph-detail-sidebar.spec.ts) now validates both detail-sidebar selection and the default hierarchical isometric layout using real sidebar ticket selection in mixed-workspace graphs
- the remaining tickets stay planned for workspace graph payloads, focus navigation, and node LOD

Completed third slice: [397fa45b [ticket-http][ticket-viewer] Expose workspace graph payload for focused full-graph navigation](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/397fa45b-a0bd-43d2-b430-2dfa44d80c5c/ticket.toml)

- [graph.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/tools/http/ticket-http/src/serve/handlers/graph.rs), [traversal.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/tools/http/ticket-http/src/serve/handlers/graph/traversal.rs), and [routes.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/tools/http/ticket-http/src/serve/routes.rs) now expose `/api/graph/workspace` and build a workspace-scoped graph payload that includes isolated local tickets, preserved edge metadata, and related cross-workspace nodes without truncating to a root-local depth window
- [backend.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/api/backend.rs), [api.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/api.rs), [graph_fetch.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/graph_fetch.rs), [graph3d.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/graph3d.rs), and [page.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/components/dep_graph/page.rs) now fetch and cache graph data per workspace so focus changes can reuse one dataset instead of reloading a new root subgraph on every selection
- focused browser coverage in [graph3d-fetch-error.spec.ts](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/e2e-release/graph3d-fetch-error.spec.ts) and [graph-detail-sidebar.spec.ts](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/e2e-release/graph-detail-sidebar.spec.ts) now validates the workspace route in both failure and success paths
- the remaining tickets stay planned for focus-navigation behavior on top of the workspace payload and for multi-level node detail rendering

Completed fourth slice: [6e7a15c9 [ticket-viewer] Keep full workspace graph visible with focused navigation](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/6e7a15c9-d8e6-4bbe-bb34-b83bd651896b/ticket.toml)

- [page.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/routes/list/page.rs) and [panels.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/routes/list/panels.rs) now keep a stable graph root per workspace while focused selection changes only the active node inside the mounted graph
- [mod.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/viewer-api/frontend/dioxus/src/graph3d/mod.rs) now applies shared `selection_auto_focus` camera retargeting so selection changes pan toward the active node without remounting or refetching the dataset
- [graph3d.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/graph3d.rs) now dims unrelated nodes around the focused context and exposes stable `data-node-id` selectors so focus changes can be asserted against the still-mounted workspace graph
- focused browser coverage in [graph-detail-sidebar.spec.ts](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/e2e-release/graph-detail-sidebar.spec.ts) now validates that graph selection recenters the active node, dims unrelated visible nodes, and keeps the main ticket route stable

Completed fifth slice: [322ba030 [viewer-api][ticket-viewer] Add multi-level graph node detail rendering](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/viewer-api/.ticket/tickets/322ba030-160c-41d3-8a12-42936ae92858/ticket.toml)

- [data.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/viewer-api/frontend/dioxus/src/graph3d/data.rs) now defines rich, compact, and minimal node detail tiers plus tier-specific dimensions chosen from projected pixel scale and focus state
- [render.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/viewer-api/frontend/dioxus/src/graph3d/render.rs) now applies the selected tier per frame, resizes the card footprint, and toggles the matching pre-rendered node detail DOM blocks
- [graph3d.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/graph3d.rs) now exposes stable `data-node-id` selectors and rich, compact, and minimal ticket card variants so the shared renderer can degrade dense graphs without unreadable text walls while keeping the active node informative
- [Cargo.toml](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/viewer-api/frontend/dioxus/Cargo.toml) now enables the minimal host-side `web-sys` features required for `dioxus-web` history compilation during focused `cargo test` runs of the shared viewer-api Dioxus crate
- focused browser coverage in [graph-detail-sidebar.spec.ts](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/e2e-release/graph-detail-sidebar.spec.ts) now validates both focused recentering/dimming and the new LOD behavior that keeps the active node rich while collapsing other visible nodes

Completed sixth slice: [1f39ba8f [ticket-viewer] Add graph review E2E coverage](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/1f39ba8f-650b-417d-b664-1878f08af669/ticket.toml)

- [graph-detail-sidebar.spec.ts](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/e2e-release/graph-detail-sidebar.spec.ts) now validates switching away from the default Hierarchical 3D plus Orthographic graph view and restoring it without losing the mounted graph or the top-to-bottom hierarchy ordering
- the same release suite now zooms the Graph3D surface out before selecting a compact or minimal child node, proving smaller LOD tiers stay clickable and the selected node remains rich after focus changes and zooming back in

Completed seventh slice: [800f09ed [ticket-viewer][viewer-api] Tighten graph layout and enlarge rich nodes](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/800f09ed-beb0-4a12-be93-1392e45eadb8/ticket.toml)

- [layout.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/layout.rs) now reduces the default hierarchy spacing and z stagger so the initial graph footprint stays tighter and the framed graph reads larger at the default camera
- [data.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/viewer-api/frontend/dioxus/src/graph3d/data.rs) now increases the ticket minimal, compact, and rich LOD dimensions so ticket-viewer nodes render larger on the shared Graph3D surface
- [graph3d.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/graph3d.rs) now renders a taller, more cubic rich card with multi-line title content and denser compact metadata so high-LOD nodes reveal more ticket information

Completed eighth slice: [d1d38010 [ticket-viewer][viewer-api] Preserve graph layout and camera across same-graph refreshes](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.ticket/tickets/d1d38010-08b8-4a06-ad2b-0bbed453c941/ticket.toml)

- [mod.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/viewer-api/frontend/dioxus/src/graph3d/mod.rs) now preserves dragged node coordinates when the mounted graph receives a same-topology layout refresh without a layout-mode change, so workspace graph re-renders and focus changes keep the local arrangement instead of animating back to backend coordinates
- the same shared sync path now retargets selection autofocus with the current camera distance instead of reframing the graph, so focus changes preserve the user's zoom level while still panning toward the active node
- [render.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/viewer-api/frontend/dioxus/src/graph3d/render.rs) now mirrors the live camera distance onto the graph container as a data attribute, giving the browser regression a stable zoom probe
- [graph-detail-sidebar.spec.ts](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/e2e-release/graph-detail-sidebar.spec.ts) now drags a node, zooms the shared graph, changes focus inside the mounted graph, and asserts both the relative node offsets and camera distance stay stable

Completed ninth slice: [eeda4039 [ticket-viewer][viewer-api] Add kanban table graph layout mode](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.ticket/tickets/eeda4039-d82d-4573-9d79-0bc89e152a76/ticket.toml)

- [camera.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/viewer-api/frontend/dioxus/src/graph3d/camera.rs) and [settings_overlay.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/viewer-api/frontend/dioxus/src/graph3d/settings_overlay.rs) now add a shared `KanbanTable` layout mode to the built-in graph settings surface so viewers can switch into a workflow-table presentation without introducing a tool-local mode fork
- [layout.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/layout.rs) now computes kanban placements with even wider workflow-state columns, dependency-group row seeds, and soft intra-cell clustering so related ticket branches stay grouped while multiple nodes remain visible inside one table cell
- [graph3d.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/graph3d.rs) now reflows cached graph layouts with the active layout mode at render time and exposes deterministic per-node layout coordinates for browser assertions, so switching between hierarchical, flat, and kanban views does not require a refetch
- [layout.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/layout.rs), [graph3d.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/src/graph3d.rs), [mod.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/viewer-api/frontend/dioxus/src/graph3d/mod.rs), and [render.rs](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/viewer-api/frontend/dioxus/src/graph3d/render.rs) now project visible kanban lane guides from shared world-space anchors, keep layout-mode camera resets framed to the guide extents, pin column headers and row labels back into the viewport when their guide anchors drift just past the top or left edge, and scale those guide labels from camera zoom so they stay readable up close while still collapsing to compact line heights far out
- [graph-detail-sidebar.spec.ts](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/e2e-release/graph-detail-sidebar.spec.ts) now switches the mounted workspace graph into Kanban mode and asserts both the left-to-right workflow ordering and the visible state headers, separators, and row labels

# Related specs

- viewer-api Graph3D shared behavior: [viewer-api Graph3D](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/viewer-api/.spec/specs/4f14356f-c4bd-4554-be1e-35361de241da/body.md)
- ticket-viewer shell layout baseline: [ticket-viewer shell](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/.spec/specs/5650b307-27b0-48d0-b722-daa5e7fd30cf/body.md)

# Validation status

Completed for [8f5d611f [ticket-viewer] Build integrated ticket document panel](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/8f5d611f-0033-423e-b2f6-17683feb8e34/ticket.toml):

- passed `cargo check --manifest-path memory-viewers/ticket-viewer/Cargo.toml`
- passed `viewer-ctl prepare ticket-viewer`
- passed `cargo build --manifest-path memory-viewers/ticket-viewer/Cargo.toml --release`
- passed `npm run test:e2e:release -- mixed-workspace-root-route.spec.ts`
- passed `cargo check --manifest-path memory-viewers/ticket-viewer/frontend/dioxus/Cargo.toml`
- passed `viewer-ctl stop ticket-viewer && cd memory-viewers/ticket-viewer/frontend/dioxus && npm run test:e2e:release -- mixed-workspace-root-route.spec.ts -g "content mode renders an integrated ticket document and keeps asset context inline"`
- visual confirmation screenshot: [mixed-workspace-integrated-document](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/playwright-report-release/data/ed4faaf64512b0cb56518c5feb9d86f952942362.png)

Completed for [60092819 [ticket-viewer] Fix graph layout defaults and isometric settings](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/60092819-f725-48ec-93f0-aba195ef81eb/ticket.toml):

- passed `cargo check --manifest-path memory-viewers/ticket-viewer/Cargo.toml`
- passed `npm run test:e2e:release -- graph-detail-sidebar.spec.ts -g "graph mode defaults to hierarchical isometric controls and preserves top-to-bottom depth ordering"`
- passed `npm run test:e2e:release -- graph-detail-sidebar.spec.ts`
- visual confirmation screenshot: [graph-mode-isometric-layout](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/ticket-viewer/frontend/dioxus/playwright-report-release/data/61f8fdc41d04e91a5f0386676858d0c531e6d637.png)

Completed for [397fa45b [ticket-http][ticket-viewer] Expose workspace graph payload for focused full-graph navigation](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/397fa45b-a0bd-43d2-b430-2dfa44d80c5c/ticket.toml):

- passed `cargo test -p ticket-http workspace_graph_includes_isolated_local_and_cross_workspace_nodes -- --nocapture`
- passed `cargo check --manifest-path memory-viewers/ticket-viewer/frontend/dioxus/Cargo.toml`
- passed `cargo build --manifest-path memory-viewers/ticket-viewer/Cargo.toml --release`
- passed `viewer-ctl install ticket-viewer`
- passed `npm run test:e2e:release -- graph3d-fetch-error.spec.ts`
- passed `npm run test:e2e:release -- graph-detail-sidebar.spec.ts`

Completed for [6e7a15c9 [ticket-viewer] Keep full workspace graph visible with focused navigation](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/6e7a15c9-d8e6-4bbe-bb34-b83bd651896b/ticket.toml):

- passed `cargo check --manifest-path memory-viewers/ticket-viewer/frontend/dioxus/Cargo.toml`
- passed `viewer-ctl stop ticket-viewer && cd memory-viewers/ticket-viewer/frontend/dioxus && npm run test:e2e:release -- graph-detail-sidebar.spec.ts`

Completed for [322ba030 [viewer-api][ticket-viewer] Add multi-level graph node detail rendering](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/viewer-api/.ticket/tickets/322ba030-160c-41d3-8a12-42936ae92858/ticket.toml):

- passed `cargo test --manifest-path viewer-api/viewer-api/frontend/dioxus/Cargo.toml node_detail_`
- passed `viewer-ctl stop ticket-viewer && cd memory-viewers/ticket-viewer/frontend/dioxus && npm run test:e2e:release -- graph-detail-sidebar.spec.ts`

Completed for [1f39ba8f [ticket-viewer] Add graph review E2E coverage](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/1f39ba8f-650b-417d-b664-1878f08af669/ticket.toml):

- passed `viewer-ctl stop ticket-viewer && cd memory-viewers/ticket-viewer/frontend/dioxus && npm run test:e2e:release -- graph-detail-sidebar.spec.ts`

Completed for [800f09ed [ticket-viewer][viewer-api] Tighten graph layout and enlarge rich nodes](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/800f09ed-beb0-4a12-be93-1392e45eadb8/ticket.toml):

- passed `viewer-ctl stop ticket-viewer && cd memory-viewers/ticket-viewer/frontend/dioxus && npm run test:e2e:release -- graph-detail-sidebar.spec.ts`

Completed for [d1d38010 [ticket-viewer][viewer-api] Preserve graph layout and camera across same-graph refreshes](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.ticket/tickets/d1d38010-08b8-4a06-ad2b-0bbed453c941/ticket.toml):

- passed `viewer-ctl stop ticket-viewer && cd memory-viewers/ticket-viewer/frontend/dioxus && npm run test:e2e:release -- graph-detail-sidebar.spec.ts -g "dragged graph layout and camera zoom persist when focus changes inside the same graph"`
- passed `cargo test --manifest-path viewer-api/viewer-api/frontend/dioxus/Cargo.toml preserve_same_topology_layout -- --nocapture`
- passed `cargo test --manifest-path viewer-api/viewer-api/frontend/dioxus/Cargo.toml selection_focus_goal_preserves_distance -- --nocapture`

Completed for [eeda4039 [ticket-viewer][viewer-api] Add kanban table graph layout mode](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/.ticket/tickets/eeda4039-d82d-4573-9d79-0bc89e152a76/ticket.toml):

- passed `cargo test --manifest-path memory-viewers/ticket-viewer/frontend/dioxus/Cargo.toml kanban_layout_ -- --nocapture`
- passed `viewer-ctl stop ticket-viewer && cd memory-viewers/ticket-viewer/frontend/dioxus && npm run test:e2e:release -- graph-detail-sidebar.spec.ts -g "graph settings can switch to kanban state columns"`

# Ongoing validation plan

- `cargo check --manifest-path memory-viewers/ticket-viewer/Cargo.toml`
- `cargo check --manifest-path viewer-api/viewer-api/Cargo.toml`
- targeted ticket-viewer browser validation in an external Chromium-family browser for document, graph focus, and layout behavior
- release Playwright coverage for each additional slice as layout, full-graph navigation, and LOD work lands
