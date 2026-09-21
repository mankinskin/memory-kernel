## Capture hook self-heal (`memory-api/crates/session-capture-hook/src/main.rs`)

The hook previously inferred a worktree from `std::env::current_dir()`, which is the main checkout when the editor launches the hook, so it wrote `path: <main>, branch: main` into the main store — the exact record the resolver reads. Two call sites were corrected:

- Capture-time inference now derives the worktree root from the store root's parent rather than the process working directory.
- `initialize_session_routing` now resolves through `SessionWorkspaceResolver` instead of the working directory. It writes **nothing** when the only candidate is the main checkout or when nothing resolves, and calls `SessionStoreConfig::replace_main_worktree_inference` to overwrite an existing main-pointing assignment once a genuine worktree resolves. That makes a previously-poisoned record self-heal on the next `UserPromptSubmit`.
- A session with no discoverable worktree is skipped entirely (no capture) by deliberate decision — there is no main-checkout fallback.

## Check-in ownership fix (`memory-api/crates/session-api/src/store/config/worktree_runtime.rs`)

`check_in_worktree` previously required both `agent_id` and `ticket_id` on an existing record to match the caller, so a record written by the capture hook (which uses a generic placeholder `agent_id`) could never be claimed. Now:

- A record whose `ticket_id` is absent or whitespace is **unclaimed** and any owner may claim it, taking over both fields.
- A record with a real `ticket_id` keeps the strict both-must-match rule and still returns `SessionOwnershipMismatch`.

## Terminal subprocess isolation (`memory-api/crates/compact-terminal-api/src/execute.rs`)

The spawned shell inherited the MCP server's stdin and wedged the stdio transport, and the timeout path left the child running. Now:

- `stdin` is set to `Stdio::null()` on spawn — the child never inherits the server's MCP-protocol stdin stream.
- On timeout, the child is killed and reaped; on Windows the Git Bash PID is recorded and MSYS descendants are killed recursively.
- Buffered stdout captured before the kill is returned as `stdout_partial`, so a timed-out command is not silently empty.
