## Objective
Make direct ticket CLI execution checkout-contained: an explicit selector always wins, and discovery remains inside the caller's checkout.

## Required behavior
- `--index-root` and `--workspace` are authoritative and never redirected by discovery.
- Descendant discovery continues to support nested stores inside the same checkout, but MUST exclude `.worktrees/**`.
- A parent ticket store never persists scan-root paths belonging to a sibling worktree.
- Opening a pre-existing polluted store prunes discovered `.worktrees/**` scan roots without relying on the removed worktree's existence.

## Owning paths
- `memory-api/crates/memory-api/src/workspace.rs`
- `memory-api/tools/cli/ticket-cli/src/cli/dispatch.rs`
- `memory-api/crates/ticket-api/src/storage/store/scan.rs`
- `memory-api/tools/http/ticket-http/src/main.rs`

## Validation
Add regression coverage proving an explicit root is honored, nested same-checkout stores are discovered, `.worktrees/**` is excluded, and a legacy scan-root row for a deleted worktree cannot affect reads or writes.

## Parent
Implements one slice of ticket `3a624bf6`.
AC3/AC4/AC5 implemented in memory-api c51114c8: scan roots under .worktrees are refused and pruned on open, indexed paths into deleted worktrees are re-resolved to the main store, and `ticket workspace roots` / `prune-roots` were added. Regression tests added; ticket-api 213 passed, ticket-cli all passed. AC1/AC2 already held.