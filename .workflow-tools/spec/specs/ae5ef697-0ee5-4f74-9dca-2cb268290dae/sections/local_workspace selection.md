A `local_workspace` is the caller-selected root the current operation is bound to (explicit `--workspace`/`workspace_root` argument, or the resolved cwd when discovery is permitted). All entity-store reads and writes for the current call are relative to this root; discovery of other roots (see below) never silently substitutes for it.

Observable criteria:
- `resolve_explicit_store_root_from` and `resolve_store_root_at_fixed_workspace` resolve strictly within the given `local_workspace` and never walk into a sibling or ancestor workspace.
- `validate_explicit_workspace_selector` rejects an omitted, empty, `default`, or `..` selector for any entity-creation call; `.` is accepted explicitly as "the current process working directory".