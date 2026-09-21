## Model Budget & Offset

### Base Budget (Model-Derived)

- **base_budget(model)**: LINEAR inverse of `output_mtok` from `tools/model-prices/model_prices.json`
- **Mapping**: Cheap model (low output_mtok) → high budget; expensive model (high output_mtok) → low budget
- **Calibration**: Tunable anchor constants ensure today's threshold X=15 boundary reproduces the current orchestrator/executor split
- **Example tiers** (at X=15):
  - Opus-tier (25–75 output_mtok) → budget ~10–30
  - Sonnet (15 output_mtok) → budget ~50 (threshold)
  - GPT-5, Gemini Pro (10 output_mtok) → budget ~70
  - Haiku, Flash, mini (<10 output_mtok) → budget ~90–100

### Offset (Grant-Based)

- **Offset resolution**: looked up from a DURABLE, auditable grant record via `grant_id`/`session_id` the caller passes per call
- **Offset value is NEVER self-declared** — callers pass a grant reference, the gate resolves it
- **Scopes** (v1): **session-wide** and **sub-agent-spawn**
- Grant records are managed via T5 (budget-offset grants)

### Gate Decision

```
effective_budget = base_budget(model) + offset(grant)
allow if tool.cost ≤ effective_budget
```

When the budget is insufficient, the gate returns `delegate` with guidance.
