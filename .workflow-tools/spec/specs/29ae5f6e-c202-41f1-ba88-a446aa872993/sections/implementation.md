## Implementation

Implemented in ticket [9185d8f2 Remove hardcoded token-heavy tool categorization](http://localhost:3002/workspace/default/ticket/9185d8f2-1080-46b1-84da-485f9ad839f6).

### Files Changed

- [memory-api/tools/mcp/mcp-cost-gate/src/gate.rs](memory-api/tools/mcp/mcp-cost-gate/src/gate.rs)
- [memory-api/tools/mcp/mcp-cost-gate/src/proxy.rs](memory-api/tools/mcp/mcp-cost-gate/src/proxy.rs)

### Summary

Removed all hardcoded tool classification constants and functions (`TOKEN_HEAVY_TOOL_SUBSTRINGS`, `ALWAYS_ALLOWED_TOOL_SUBSTRINGS`, `ToolClass`, `classify_tool()`, `heavy_fallback_cost()`).

`Gate::tool_cost()` now returns the max measured cost from the empirical rollup when `call_count >= MIN_CALLS`, else 0 (fail-open). `MIN_CALLS` lowered from 5 to 1.

`evaluate_legacy()` now delegates to `evaluate()`.

### Test Evidence

- **Unit tests**: `cargo test -p mcp-cost-gate` → 26 passed, 0 failed
- **Release build**: `cargo build --release -p mcp-cost-gate` → succeeded
- **Binary installation**: `./install-tools.sh --tool mcp-cost-gate` → reinstalled successfully

Tests removed: `classify`, `tool_cost_static_fallback`, `heavy_fallback_boundary_tests`, `always_allowed_bypass`, `expensive_token_heavy_is_refused`.

Tests added/updated: `tool_cost_fail_open_no_rollup`, `tool_cost_from_rollup`, `evaluate_with_rollup`, `fail_open_unmeasured`, `evaluate_legacy_delegates_to_evaluate`, `graded_budget_boundary_with_rollup`, `expensive_measured_tool_is_refused`, `unmeasured_tool_fail_open`.

### Known Limitations

**No standalone CLI verdict command:** mcp-cost-gate is an MCP stdio proxy only. Live expensive-model verification could not be executed in-session; AC7 manual smoke check is covered by unit and proxy integration tests only.

**Pre-existing behavior preserved:** Bidirectional substring matching between requested tool name and rollup tool names in `tool_cost()` was present before this change.
