## Unmeasured-Tool Policy

The cost gate is **fail-open** for unmeasured tools: a tool with no entry in the tool-metrics rollup is assigned cost **0** and is always allowed. Unproven cost is never treated as high cost.

All hardcoded tool classification is removed. There are no `TOKEN_HEAVY_TOOL_SUBSTRINGS`, no `ALWAYS_ALLOWED_TOOL_SUBSTRINGS`, no `ToolClass`, and no `heavy_fallback_cost()`. The empirical rollup is the sole source of tool cost.

A tool becomes subject to gating after **one** recorded call (`MIN_CALLS = 1`); its cost is the average over the existing rollup aggregation window. No separate cost-averaging window is introduced.

The behavior is unconditional — no env flag or opt-out.

### Rationale

Static gating blocked read/inspection tools for large models while the measurement pipeline produced no data, which prevented the measurements needed to make gating accurate. Gating must be earned by evidence.

### Unchanged by This Policy

- Rejection of unknown `caller_model`
- The grant/offset mechanism
- Existing fail-open when the price table cannot be loaded
