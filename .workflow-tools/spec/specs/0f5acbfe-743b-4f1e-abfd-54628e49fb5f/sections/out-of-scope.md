## Out of scope / future work

Ticket `5e6cf4f8-120c-4674-95de-d7b79c99f5b3` ("Rewrite worktree.sh as a Rust binary and add worktree lifecycle recycling") is **not implemented** and is explicitly out of scope for the behavior this spec describes. It depends on `a1b911ab-9394-4ba8-9134-1b2687e96ccd` (delivered) and covers:

- eager worktree creation from the `UserPromptSubmit` hook,
- a Rust rewrite of `tools/worktree/worktree.sh` driving git through a library instead of shelling out,
- locking a worktree while its session is active,
- clean/dirty-gated automatic reclamation (rename/move in place) on session completion,
- rebuilding machine-local entity-store indexes (`.ticket`, `.spec`, `.test` `*.db`/`search_index/`) on a newly created or recycled worktree.

None of this is delivered by the work this spec traces. Do not treat eager creation or lifecycle recycling as implemented behavior until `5e6cf4f8-120c-4674-95de-d7b79c99f5b3` closes with its own validation evidence.
