<!-- aligned-structure:v1 -->

# Summary

The current ticket-viewer graph still centers its presentation around rich DOM ticket cards, limited focus falloff, and a mostly 3D/isometric camera model. That makes dense graphs expensive to render, visually noisy around the app panels, and awkward for presentation-oriented selection flows.

## Behavior Story

The current ticket-viewer graph still centers its presentation around rich DOM ticket cards, limited focus falloff, and a mostly 3D/isometric camera model. That makes dense graphs expensive to render, visually noisy around the app panels, and awkward for presentation-oriented selection flows.

## Provided Surface Contracts

- Ticket-viewer graph data flow supports loading a workspace-scale graph into a client-side cache while rendering only a bounded neighbourhood around the active selection.
- Bounded neighbourhood rendering is parameterized by configurable graph-distance/focus rules and avoids full-graph visual overload.
- Selection changes recompute the rendered neighbourhood from cache without network refetch.
- Graph focus behavior preserves predictable selection semantics: selected node remains emphasized, immediate neighbours retain higher emphasis, and low-relevance nodes degrade by explicit focus bands.
- Node rendering follows an ordered level-of-detail ladder (`point/sphere -> icon -> label -> compact -> full`) driven by projected size, focus, and visibility budget.
- The shared Graph3D renderer remains extensible and performant for focus, LOD tiers, and cache-bounded neighbourhood rendering, with reusable extension points for additional viewers.
- Optional fixed 2D presentation mode remains available with matching 2D framing behavior and consistent selection/focus expectations.

## Required Validation

- Playwright release coverage validates bounded neighbourhood rendering from cached full-graph data and confirms selection changes do not trigger graph refetches.
- Playwright release coverage validates focus-band behavior (selected node, immediate neighbours, wider context, low-focus falloff) and outside-click deselection behavior.
- Playwright release coverage validates ordered LOD transitions, including one-tier hover promotion and camera-mode-appropriate low-tier glyphs.
- Playwright release coverage validates optional fixed 2D mode switching and parity of core focus/selection interactions between 2D and 3D modes.
- Focused renderer performance validation documents graph-size baselines, frame stability, and regression thresholds for cache-bounded neighbourhood updates.
- Extension-point documentation validation confirms renderer integration points remain consumable for non-ticket viewers (rule/audit/demo).
- Ticket traceability for this spec is maintained through W2 `e978d833-bbaa-4cb8-8c1a-52a282a079d9` and W6 `e0d8ccef-49d8-4446-bd63-78626a4163dc`.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Problem

The current ticket-viewer graph still centers its presentation around rich DOM ticket cards, limited focus falloff, and a mostly 3D/isometric camera model. That makes dense graphs expensive to render, visually noisy around the app panels, and awkward for presentation-oriented selection flows.

# Goals

- make selection and de-selection behavior explicit and predictable
- drive node appearance from property-based level-of-detail rules instead of a fixed rich-card default
- keep graph content visually behind sidebar and in-viewport panels while still reacting to the visible graph area
- add an optional fixed 2D graph camera and matching 2D grid presentation
- support temporary presentation layouts that can move nodes into a more readable arrangement while a selection is active
- close the remaining design gaps around node tier ordering, hover behavior, selected-state effects, and end-to-end validation coverage

# Requirements

## Selection focus model

- graph nodes are transparent only when they are outside the active focus context
- selecting a node keeps the selected node fully emphasized and keeps at least the immediate linked neighborhood in higher focus
- the focus model may expand beyond direct neighbors through a selection-strength or path-length rule, but the default emphasis remains the immediate neighborhood
- focus effects decay stepwise by path distance, with explicit bands for selected node, direct neighbors, wider neighborhood, and low-focus nodes
- selected-state emphasis includes slight scale growth on the selected node, glow or halo emphasis, brighter or thicker linked edges, and a neighborhood tint shift for related nodes
- clicking or tapping outside graph nodes clears the active node selection

## Property-based node rendering

- node rendering supports the ordered detail ladder `point/sphere → icon → label → compact → full`
- the lowest detail tier switches by camera mode: fixed 2D mode uses flat points and 3D mode uses small shaded spheres
- detail selection is driven by projected size, focus state, visibility budget, and other render properties rather than by a single card-first template
- hover raises a node by exactly one detail tier rather than jumping directly to a rich preview
- tooltips are not used as a separate low-tier fallback when tier promotion is sufficient
- full HTML-rich node rendering is reserved for high-detail tiers only
- lower detail tiers may use color, shading, icons, and condensed labels to trade information density against performance and screen space

## Panel-aware graph presentation

- graph nodes always project behind the UI screen plane and never render visually above sidebar panels or in-viewport panels
- graph layout and camera framing account for panel-occupied screen space by biasing focus toward the remaining visible graph area
- panel boundaries can repel or otherwise push node projections so dense clusters do not sit directly under high-importance UI chrome

## Optional fixed 2D mode

- the graph supports an optional fixed 2D canvas camera mode in addition to the current 3D-oriented presentation
- the 2D mode uses matching 2D grid patterns and interaction framing suited to a planar graph canvas
- switching between presentation modes preserves graph usability and predictable focus behavior

## Presentation keyframing

- selection can temporarily move nodes into alternate presentation positions without losing the underlying graph layout
- the system supports keyframed transitions into and out of these temporary layouts
- temporary presentation layouts can be used to open space for detailed views around the selected node and then restore the prior arrangement
- keyframing can be triggered both automatically on selection and explicitly by the user
- automatic keyframing is configurable in settings so users can enable it, disable it, or tune when selection should trigger it

# Planned work

Tracker: [10c94251 [ticket-viewer][viewer-api] Graph focus and 2D presentation follow-up](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/10c94251-1c0c-4542-a282-ea3d75a205b5/ticket.toml)

1. [923c866a [ticket-viewer][viewer-api] Refine graph selection focus and outside-click deselection](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/923c866a-fecd-4ddb-8be0-00ca4cb22af9/ticket.toml)
2. [f9e9aaae [viewer-api][ticket-viewer] Introduce property-based graph node rendering tiers](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/viewer-api/.ticket/tickets/f9e9aaae-b1ec-434c-a839-7ec990d1e6c7/ticket.toml)
3. [929bc26a [ticket-viewer][viewer-api] Make graph framing panel-aware and keep nodes behind UI panels](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/929bc26a-5296-4d64-b1b2-2ec580c0659c/ticket.toml)
4. [68eaae1f [viewer-api][ticket-viewer] Add optional 2D graph mode and presentation keyframing](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/viewer-api/.ticket/tickets/68eaae1f-b230-4aab-8572-cbf41d1d3b6d/ticket.toml)

# Related work

- linked layout/document graph tracker: [05dae5fd [ticket-viewer][ticket-http][viewer-api] Improve main layout ticket documents and focused full-graph navigation](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/.ticket/tickets/05dae5fd-1a1d-4a64-be62-f29ca0771a4d/ticket.toml)
- prior graph detail rendering ticket: [322ba030 [viewer-api][ticket-viewer] Add multi-level graph node detail rendering](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/.ticket/tickets/322ba030-160c-41d3-8a12-42936ae92858/ticket.toml)
- prior focused graph navigation ticket: [6e7a15c9 [ticket-viewer] Keep full workspace graph visible with focused navigation](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/.ticket/tickets/6e7a15c9-d8e6-4bbe-bb34-b83bd651896b/ticket.toml)
- prior layout-defaults ticket: [60092819 [ticket-viewer] Fix graph layout defaults and isometric settings](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/.ticket/tickets/60092819-f725-48ec-93f0-aba195ef81eb/ticket.toml)

# Validation plan

- add release Playwright coverage for outside-click deselection using empty graph-space pointer input and assertions on cleared selection state
- add release Playwright coverage for the ordered detail ladder, including camera-mode-specific lowest-tier glyphs (`flat point` in 2D and `small sphere` in 3D), one-tier hover promotion, and no-tooltip fallback assumptions
- add release Playwright coverage for selected-state effects: scale increase on the selected node, glow or halo state hooks, brighter or thicker linked edges, and neighborhood tint changes with stepwise path-distance falloff
- add release Playwright coverage for panel-aware framing by opening sidebar and in-viewport panels, asserting nodes remain visually behind panels, and verifying focus centering shifts into the remaining visible graph area
- add release Playwright coverage for optional 2D mode switching, 2D grid rendering, and parity of selection/focus behavior between 2D and 3D modes
- add release Playwright coverage for keyframed presentation activation in both explicit and auto-trigger paths, including settings-controlled auto-trigger enable/disable behavior and deterministic restore of the prior layout
- perform focused browser validation in an external Chromium-family browser for selection clearing, panel overlap behavior, 2D mode, selected-node effects, and keyframed presentation transitions
