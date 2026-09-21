## Graded Cost Scale

The cost-gate moves from a binary light/heavy classification to a **graded numeric model** (1–100 scale):

- **Scale**: 1–100, configurable, with named tiers layered on top
- **Gate decision**: `effective_budget = base_budget(model) + offset(grant)`; **allow if `tool.cost ≤ effective_budget`**

### Tool Cost Assignment

- **tool.cost**: LINEAR map of empirical `est-output-tokens` (from tool_metrics T1/T2) → 1–100, clamped
- Tools with **insufficient data** (< 5 calls) receive a **single default cost for unknown tools**
- Default cost calibrated to:
  - Gate expensive/orchestrator-tier models (above their budget)
  - Remain below cheaper-agent budgets (bootstrap requirement)
  - Allow cheaper agents to call unknown tools and gather metrics

### Named Tiers

While the gate operates on the numeric scale, named tiers provide semantic labels:

- **Ultra-heavy** (90–100): Bulk file operations, mass search results
- **Heavy** (70–89): Most MCP tools
- **Medium** (40–69): Bounded reads, targeted queries
- **Light** (1–39): Simple lookups, metadata queries

Tier boundaries are **tunable** and calibrated so today's threshold X=15 boundary + current heavy list behavior is reproduced as the default.
