## Validation evidence

All of the following passed on branch `agent/70abae1b-session-worktree-discovery`:

| Crate | Tests | Notable cases |
|---|---|---|
| `session-workspace-resolver` | 24 tests | `a_main_pointing_record_does_not_defeat_discovery`, `a_main_pointing_record_is_honored_when_nothing_is_discoverable` |
| `session-api` | 6 unit + 4 integration | — |
| `session-capture-hook` | 8 unit + 4 integration | `user_prompt_submit_without_discoverable_worktree_does_not_assign_main` |
| `compact-terminal-api` | 10 tests | — |
| `compact-terminal-mcp` | 6 integration tests | — |

`cargo fmt --check` is clean for all changed files; `clippy` shows only pre-existing warnings.

### Live CLI verification of the terminal fix

Via `memory-api/target/debug/compact-terminal.exe run`:

- `echo alive-first` → exit 0, 24ms
- `echo alive-second` immediately after → exit 0, 24ms (this back-to-back case previously hung)
- `cat` with no input → exit 0, 33ms (previously hung forever on inherited stdin)
- `sleep 30` with `--timeout 3` → `timed_out` returned in 3.125s, no orphaned child in `ps`
