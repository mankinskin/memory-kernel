## Grant Records

Budget-offset grants provide a **durable, auditable** mechanism to increase a caller's effective budget.

### Grant Schema

- **grant_id**: UUID reference
- **scope**: session-wide | sub-agent-spawn
- **offset**: Numeric budget increase (1–100 scale)
- **created_at**, **revoked_at**: Audit timestamps
- **metadata**: Optional context (e.g., reason, issuer)

### Operations

- **create**: Issue a new grant with scope and offset
- **list**: Query active grants by scope/session/agent
- **revoke**: Mark a grant as revoked (sets `revoked_at`, excludes from lookups)
- **get_by_id**: Resolve a grant reference to its offset value

### Storage & Surface

- **Storage**: `session-api` (memory-api/crates/session-api)
- **CLI**: `session-cli grant create|list|revoke|get`
- **MCP**: `session-mcp` grant tools

### Cost-Gate Integration

The cost-gate (T3) resolves the offset by:
1. Extracting `grant_id` from the caller's request metadata
2. Calling `get_grant(grant_id)` to resolve the offset
3. Computing `effective_budget = base_budget + offset`
4. Applying the gate decision

This ensures **offset values are never self-declared** and remain auditable.

See **T5** [6737a239](../../../.ticket/tickets/6737a239-60fa-44af-8bf3-a60f8eb1e8a8/ticket.toml) for implementation details.
