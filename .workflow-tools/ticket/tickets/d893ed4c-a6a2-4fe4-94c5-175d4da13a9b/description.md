W7. Descendant .ticket stores are already discovered and folded into the aggregated default index (verified: memory-viewers/viewer-api/memory-api tickets appear in .ticket/index.toon), but list_workspaces only reports the aggregated root and does not enumerate descendant stores as separately-selectable workspaces. This blocks a viewer domain/workspace selector.

Requirements:
- Extend list_workspaces to enumerate policy-discovered descendant store roots as labelled, selectable workspaces.
- Reuse discover_workspace_scan_roots_with_policy (memory-api/crates/memory-api/src/workspace.rs) so enumeration and aggregation agree.
- Surface labels usable by the viewer domain selector (W5b).

Implementation lives in memory-api / ticket-api / ticket-mcp. Enables root-level domain selection in the demo-viewer (W5b).