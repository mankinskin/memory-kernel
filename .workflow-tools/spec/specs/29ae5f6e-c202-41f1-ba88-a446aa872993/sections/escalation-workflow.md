## Escalation Workflow

When a sub-agent is blocked by an insufficient budget, it can **escalate upward** rather than failing silently or guessing.

### Escalation Record

- **escalation_id**: UUID
- **blocking_decision**: What the gate rejected
- **context**: Why the tool was needed
- **requested_capability/offset**: What would unblock the agent
- **options_considered**: Alternative approaches the agent evaluated
- **resolution_outcome**: How the escalation was resolved
- **created_at**, **resolved_at**: Timestamps

### Sub-Agent Convention

When blocked, return the marker:
```
ESCALATION:<record-id>
```

The orchestrator detects this marker and resolves the escalation.

### Resolution Actions

1. **handle it**: Orchestrator performs the work directly
2. **grant offset**: Issue a grant (T5) and retry the sub-agent with `grant_id`
3. **escalate to user**: Ask the user for guidance
4. **spawn session**: Delegate to an interactive session

The resolution outcome is recorded in the escalation record.

### Async Queue

Escalations support **async pickup** — if an orchestrator doesn't resolve in-turn, the escalation remains in the queue and can be resolved later by:
- The same orchestrator on the next turn
- A different orchestrator or session
- A user via CLI/MCP

### Storage & Surface

- **Storage**: `session-api` (memory-api/crates/session-api)
- **CLI**: `session-cli escalation create|list|resolve`
- **MCP**: `session-mcp` escalation tools

See **T6** [c81f3938](../../../.ticket/tickets/c81f3938-0b4b-42a0-bbf1-888ddd9d2262/ticket.toml) for implementation details.
