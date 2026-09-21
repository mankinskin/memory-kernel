## Argument-Based Dynamic Cost Estimation

### Problem

The graded cost model currently assigns each tool a **single static per-tool cost**. Tools without sufficient empirical data receive the **single default cost for unknown tools**. This ignores that a tool's real token cost depends on its **call arguments**.

Peek tools are designed for bounded, token-efficient reads. A single default cost over-restricts expensive models from cheap bounded reads.

### Design: Argument-Based Dynamic Cost Estimation

**Effective per-CALL cost** = `clamp(base_cost + Σ estimator_i(arg_value_i), 1, scale_max=100)`

#### Per-Tool Cost Model (Declarative)

Each tool can define:
- **base_cost**: Static baseline (used when no arg estimators apply)
- **arg_estimators**: Array of estimators over specific call parameters

Estimator shape:
```
{
  param_path: "arg_name",
  scaling: "linear" | "log",  // per unit
  unit_weight: <float>,       // scale factor
  cap: <optional max contribution>
}
```

Absent args fall back to `base_cost`.

#### Peek Tool Integration

Peek tools receive estimators over their windowing parameters:
- **peek_read**: window size / line count / byte limit
- **peek_grep**: max-results / context radius
- **peek_skeleton**: depth
- **peek_count**: scope

Small windows land in **light/medium** tier; only large windows approach **heavy**.

#### Calibration Interplay

- **Pre-calibration**: Declarative estimators replace the single default cost for arg-bounded tools
- **Post-calibration**: Once ≥5 empirical calls exist for a tool, arg values feed the size→cost mapping; declarative estimators remain the fallback when empirical data is sparse for a specific arg range

#### Implementation Location

Single source of truth (or mirrored) across:
- **Python gate**: `tools/model-prices/cost_gate.py`
- **Rust proxy**: `memory-api/tools/mcp/mcp-cost-gate/src/gate.rs`

### Manual Override Continuation

Operators can still override any tool's effective cost via:
1. **Grant records** in `.session/grants/` — offsets stack on top of `effective_budget`
2. **Direct editing** of the declarative per-tool cost model

### Traceability

**Implementing Ticket**: [9c9e2edc](../../../.ticket/tickets/9c9e2edc-81fc-489e-9153-bf4ac0bf1a13/ticket.toml) — Dynamic argument-based cost estimation in cost gate

**Related Tickets**:
- [8c4d1d9c](../../../.ticket/tickets/8c4d1d9c-1004-4539-9880-0a0e8aa03dd3/ticket.toml) — Related scope (token-optimized default agent tools)
- [445a2d76](../../../.ticket/tickets/445a2d76-5795-4d7a-aec8-d1536ec61416/ticket.toml) — Parent epic

**Dependencies**: Builds on T1–T3 (tool-metrics + graded cost scale)
