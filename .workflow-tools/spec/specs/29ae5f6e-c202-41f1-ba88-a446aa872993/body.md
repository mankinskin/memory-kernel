# Empirical Tool-Metrics Driven Cost-Gate Classification

## Motivation

Replace the hardcoded, guess-based `TOKEN_HEAVY_TOOL_SUBSTRINGS` classification in `mcp-cost-gate` and `tools/model-prices/cost_gate.py` with an **empirically-derived** classification computed from our own session transcripts. This ensures the cost-gate reflects actual tool output behavior observed in production rather than static assumptions.

## Data Model

### Core Types

- **`ToolTokenStats`**: Per-tool statistics
  - `call_count`: Total invocations
  - `success_count`: Successful invocations (failed calls excluded from size percentiles)
  - Size quartiles: p25, p50, p75, p90 of output token estimates

- **`ToolMetricsReport`**: Global rollup across all sessions
  - Map of tool_name → `ToolTokenStats`
  - Window metadata: session count, date range
  - Schema version for forward compatibility

- **Tokenizer Trait**: Pluggable token estimation
  - Default implementation: `chars ÷ 4`
  - Designed for swap to real tokenizer later

### Per-Session Summary

Each session stores a **`tool-metrics.json`** file beside `session.json`, `transcript.json`, and `events.json`:
- Contains **sizes and counts ONLY** (no transcript content — privacy)
- Computed at capture time from `SessionTurn` entries where `role=Tool`
- Lazy backfill: sessions missing tool-metrics.json are processed on first aggregation pass

## Efficiency Design

### Aggregation Strategy

1. **Per-session summaries**: Computed once at capture time, persisted as small JSON files
2. **Global aggregation**: Reads the small per-session summaries (O(sessions)), not full transcripts
3. **Window scoping**: Last 30 days AND ≤100 most recent sessions (whichever is fewer)
4. **Scope**: Aggregates across ALL discoverable `.session` stores (global)

### Failure Handling

- Failed tool calls (where `event_meta.tool_success = false`) are **excluded from size percentiles** but counted separately
- This ensures heavy/light classification reflects typical successful output size, not error messages

### Rollup Refresh

- **Owner**: Copilot capture hook
- **When**: After each session persist
- **Action**: Invokes aggregation + export to `<store_root>/tool-metrics-rollup.json`

## Related Tickets

- [77eb143b Measure substitutable shell commands (classifier)](.ticket/tickets/77eb143b-0322-4c91-b3c4-deccc2b2927c/ticket.toml)
- [b7c61f0e Promote the sub-agent cost analyzer into session-api](.ticket/tickets/b7c61f0e-ed42-4eef-8d3b-da934d7c0628/ticket.toml)
- [9185d8f2 (existing related ticket)]
- **Location**: Session store root, read by cost-gate via `COST_GATE_TOOL_METRICS` env var

## Graded Cost Model

The cost-gate has evolved from binary heavy/light classification to a **graded numeric model** (1–100 scale). See dedicated sections:

- **Graded Cost Scale**: Numeric scale, named tiers, tool cost assignment
- **Model Budget & Offset**: Base budget from model pricing, grant-based offsets
- **Grant Records**: Durable grant storage and resolution
- **Escalation Workflow**: Upward escalation when blocked
- **Graded Cost Policy**: Gate decision flow and calibration
- **Argument-Based Cost Estimation**: Dynamic per-call cost estimation using tool argument properties

The empirical tool-metrics foundation (T1/T2) remains unchanged; the graded model extends how those metrics drive gate decisions.

## Downstream Contract

### Rust Gate (mcp-cost-gate)

- Reads rollup via `COST_GATE_TOOL_METRICS` env var (points to `tool-metrics-rollup.json` path)
- Reads grants via `COST_GATE_GRANTS_DIR` env var (points to session store `.session/<workspace>/grants/` directory)
- Tools without sufficient empirical data (< 5 calls) receive a **single default cost for unknown tools**
- Default cost calibrated to gate expensive/orchestrator-tier models while remaining below cheaper-agent budgets (bootstrap requirement)
- **Fails open**: Missing/unreadable rollup → fall back to single default cost
- Gate only **reads** the rollup (never writes)

### Python Mirror (tools/model-prices/cost_gate.py)

- Implements identical single-default-for-unknown-tools + fail-open logic
- Maintains parity with Rust behavior

### Grant JSON Contract

Grants are stored as individual JSON files with schema:
```json
{
  "grant_id": "<uuid>",
  "scope": "session-wide | sub-agent-spawn",
  "offset": <1-100>,
  "created_at": "<RFC3339>",
  "revoked_at": "<RFC3339|null>",
  "metadata": { "reason": "...", "issuer": "..." }
}
```

### Integration Documentation

- **AGENTS.md** updated with graded budget model, cost routing, and delegation guidance (section: "Model cost awareness & routing")

## Traceability

### Implementation Tickets

- **T1** [b64cc71d](../../../.ticket/tickets/b64cc71d-8594-4617-b3fb-3057fca0b56b/ticket.toml): session-api tool_metrics core — **implemented+tested**
- **T2** [66c85d65](../../../.ticket/tickets/66c85d65-98bc-4432-b6f6-6c41664645f8/ticket.toml): tool_metrics surfaces + rollup writer — **implemented+tested**
- **T3** [4e7e53f5](../../../.ticket/tickets/4e7e53f5-b3de-477f-8cbd-f88b6c103bb5/ticket.toml): graded cost + model budget + offset resolution — **implemented+tested**
- **T4** [a0b59873](../../../.ticket/tickets/a0b59873-abe9-4e62-84a3-c233635b4cd6/ticket.toml): spec + validation — **implemented+tested**
- **T5** [6737a239](../../../.ticket/tickets/6737a239-60fa-44af-8bf3-a60f8eb1e8a8/ticket.toml): budget-offset grants — **implemented+tested**
- **T6** [c81f3938](../../../.ticket/tickets/c81f3938-0b4b-42a0-bbf1-888ddd9d2262/ticket.toml): upward escalation workflow — **implemented+tested**
- **T7** [9c9e2edc](../../../.ticket/tickets/9c9e2edc-81fc-489e-9153-bf4ac0bf1a13/ticket.toml): dynamic argument-based cost estimation — **planned**
- **T8** [9185d8f2](../../../.ticket/tickets/9185d8f2-1080-46b1-84da-485f9ad839f6/ticket.toml): remove hardcoded token-heavy tool categorization; single default cost + empirical bootstrap — **planned**

**Related/Context Tickets**:
- [8c4d1d9c](../../../.ticket/tickets/8c4d1d9c-1004-4539-9880-0a0e8aa03dd3/ticket.toml): token-optimized default agent tools (peek suite)
- [445a2d76](../../../.ticket/tickets/445a2d76-5795-4d7a-aec8-d1536ec61416/ticket.toml): parent epic

### Dependencies

- T2 depends on T1
- T3 depends on T1
- T5 depends on T1, T3
- T6 depends on T5
- T4 depends on T1, T2, T3, T5, T6
- T7 depends on T1, T2, T3

### Validation Evidence

**Date**: 2026-07-26  
**Test Suites Validated** (all passed):

1. **session-api**: 139 tests passed  
   Command: `cargo test -p session-api`
   
2. **session-cli**: 18 tests passed  
   Command: `cargo test -p session-cli`
   
3. **session-mcp**: 11 tests passed  
   Command: `cargo test -p session-mcp`
   
4. **mcp-cost-gate**: 24 tests passed  
   Command: `cargo test -p mcp-cost-gate`
   
5. **python-cost_gate**: 36 tests passed  
   Command: `python tools/model-prices/test_cost_gate.py`

**Total**: 228 tests passed, 0 failed

**Evidence Records**: Validation specs and execution results recorded in `.test/default/` store and linked to this spec via test-api (validation execution IDs: `exec-session-api-2026-07-26`, `exec-session-cli-2026-07-26`, `exec-session-mcp-2026-07-26`, `exec-mcp-cost-gate-2026-07-26`, `exec-python-cost-gate-2026-07-26`).

## References

- Transcript data model: `memory-api/crates/session-api/src/model.rs` (~L205)
- Existing audit pattern: `memory-api/crates/session-api/src/audit.rs`
- Gate consumers: `memory-api/tools/mcp/mcp-cost-gate/src/gate.rs`, `tools/model-prices/cost_gate.py`
