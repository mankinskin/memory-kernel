<!-- aligned-structure:v1 -->

# Summary

The ticket graph is queryable, but operators still have to mentally reconstruct dependency shape, bridge nodes, and parallel tracks from list output. Existing graph-aware next planning improves ranking, but there is no canonical rendering contract for graph display in CLI output, generated docs, or embedded planning artifacts.

## Behavior Story

The ticket graph is queryable, but operators still have to mentally reconstruct dependency shape, bridge nodes, and parallel tracks from list output. Existing graph-aware next planning improves ranking, but there is no canonical rendering contract for graph display in CLI output, generated docs, or embedded planning artifacts.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Problem

The ticket graph is queryable, but operators still have to mentally reconstruct dependency shape, bridge nodes, and parallel tracks from list output. Existing graph-aware next planning improves ranking, but there is no canonical rendering contract for graph display in CLI output, generated docs, or embedded planning artifacts.

## Goals

- Define one reusable graph-rendering primitive that can draw directed graphs for ticket and related stores.
- Define closure-aware expansion for a ticket graph command so selected nodes render with their transitive bridge nodes instead of misleading partial edges.
- Support both compact human output and durable export formats that can be embedded into tickets, specs, and generated docs.
- Leave room for a later flow-graph mode that highlights sequencing pressure and parallel-ready tracks.

## Required behavior

### Reusable renderer primitive
- The rendering primitive accepts a node set, directed edges, optional labels, and optional status metadata.
- The primitive is store-agnostic: ticket-specific commands prepare data, while the renderer owns layout, ordering, and formatting.
- Rendering must be deterministic for stable diffs and embeddable generated output.

### Closure-aware ticket graph command
- A ticket graph command accepts one or more seed tickets or a predefined ticket set.
- When two included tickets are transitively connected by `depends_on`, the rendered graph must include every intermediate node on at least one connecting path.
- The command may expose explicit modes such as `--closure minimal-paths` and `--closure full-subgraph`, but the default must never hide bridge nodes needed to explain an included edge relationship.
- The command should support root-oriented expansion over tracker subgraphs and reverse-dependency views.

### Output formats
- Mermaid is the canonical portable graph output because it survives embedding in specs, tickets, and generated documentation.
- ASCII remains supported for terminal-first inspection and quick human review.
- JSON and TOON graph output MUST include `id`, `label`, and `edge_kind`; `derived_status` MUST be either a documented enum value or null.

### Flow and parallel-track overlays
- A later flow-graph mode may decorate the same graph with derived metadata from `ticket-api`, including dependency-convergence pressure, blocked-vs-ready state, and parallel-ready track grouping.
- The base rendering contract must therefore allow optional annotations without hard-coding ticket workflow semantics into the primitive.

### Embedded generated artifacts
- Specs and tickets may embed generated Mermaid or ASCII graphs as derived artifacts, but those embeds must be generated from canonical graph data rather than hand-maintained diagrams.
- Rule-generated docs may consume the same rendering contract for graph snapshots.

## Acceptance criteria

- A spec-owned renderer contract exists with deterministic node and edge ordering requirements.
- Ticket graph display defines closure semantics that include transitive bridge nodes by default.
- Mermaid and ASCII outputs are both accounted for, with Mermaid treated as the durable embedding format.
- The contract leaves a clean extension point for flow/parallel overlays without coupling the renderer to one domain model.

## Related specs

- `ticket-api/workflow/graph-aware-best-next`
- `audit-api/ticket-dependency-topology-validation`
- `architecture/cross-store-workspace-interaction`

## Traceability

- [f3305925 [ticket-cli] Graph rendering and closure-aware dependency display](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/f3305925-7217-4ff3-8c4e-820ebc1e6de5/ticket.toml)
- [43fc22b3 [ticket-graph] Tracker: validation-aware graph tooling and audit enforcement](C:/Users/linus/git/graph_app/.workflow-tools/ticket/tickets/43fc22b3-9b36-4a54-b520-f51000330a46/ticket.toml)

## Validation

- Focused ticket-api graph fixture tests proving closure expansion includes intermediate path nodes.
- Focused ticket-cli tests proving deterministic ASCII and Mermaid output for the same fixture graph.
- Golden-output tests proving embedded Mermaid output remains stable across repeated runs.
