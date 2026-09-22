## Objective
Apply the worktree isolation contract to feedback, spec, and test stores.

## Required behavior
- Direct CLI/store operations use only the caller's containing checkout or explicit selector.
- MCP operations are resolved to the session-assigned worktree only.
- Ancestor discovery cannot silently write a worktree's feedback, spec, or test artifacts into the main checkout.
- Nested stores inside the same checkout remain supported; sibling `.worktrees/**` stores are never candidates.
- Legacy cross-worktree paths are pruned or inert after migration.

## Owning paths
- `memory-api/crates/memory-api/src/workspace.rs`
- `memory-api/crates/spec-api/src/store.rs`
- `memory-api/tools/mcp/feedback-mcp/src/server.rs`
- `memory-api/tools/mcp/test-mcp/src/server.rs`

## Validation
Add isolation tests for explicit and CWD-based resolution in two checkouts, including nested same-checkout stores and excluded sibling worktrees.

## Parent
Implements one slice of ticket `3a624bf6`; build on the shared resolver semantics from ticket `8130027d`.