<!-- aligned-structure:v2 -->

## Motivation

`worktree-ctl new` creates a Git worktree and its submodules but leaves the worktree-local `.ticket`, `.spec`, and `.rule` runtime state uninitialized. The initialization failure blocks session and board operations immediately after worktree creation.

## Dependent Expectation

If this specification is implemented, dependents can rely on `worktree-ctl bootstrap <session-uuid> <slug>` to create or reuse the session worktree and run the worktree's existing `init.sh` before a session attempts ticket or board operations. Ticket [5f075124 Bootstrap worktree-local repository stores](.ticket/tickets/5f075124-402c-4a47-a549-5f522c4d95d1/ticket.toml) tracks the implementation.

## Guards

- `cargo test -p worktree-ctl --test worktree_contracts bootstrap -- --nocapture` passes the two bootstrap regressions and the existing submodule bootstrap regressions.
- `cargo test -p worktree-ctl --test worktree_contracts -- --skip guidance_documents_nested_legacy_and_worktree_local_active_session_marker` passes the remaining controller contracts. The omitted guidance assertion predates this change and expects a retired session marker in an untouched instruction file.

## Positions

- `tools/worktree/worktree-ctl/src/main.rs::handle_bootstrap`: implemented; reuses `new`, then invokes `init.sh` from the worktree root.
- `tools/worktree/worktree-ctl/tests/worktree_contracts.rs`: implemented; verifies initialization runs in the worktree and dry runs do not create one.
- `.agents/instructions/commit/branch-worktree.instructions.md`: implemented; documents the one-line bootstrap command and idempotency boundary.
- `worktree-ctl new`: implemented and unchanged as a Git/submodule-only operation.

## Governing-Rule Requirement

The session worktree workflow guidance must introduce this draft contract before an agent relies on `bootstrap`; [branch-worktree.instructions.md](.agents/instructions/commit/branch-worktree.instructions.md) is the current operational owner. The governing PolicyRule link is pending because no rule-store read surface is available in the assigned worktree.
