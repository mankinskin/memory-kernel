---
description: "Use when editing context-engine core crates (trace/search/insert/read/api). Covers layering, common API gotchas, and edit-time rules."
---


## Architecture Order

The workspace layers build in this order:
1. `context-trace`
2. `context-search`
3. `context-insert`
4. `context-read`

When changing upper layers, check assumptions in lower layers first.

## Edit Rules

- Keep public APIs stable unless the task explicitly requires changes.
- Preserve existing type and naming conventions within the crate.
- Keep scope disciplined: avoid unrequested broad refactors, but proactively fix a discovered quality finding in code the current unit touches or directly depends on; that related fix is in scope.

## Discovery Checklist

Before editing, per [AGENTS.md](../../../AGENTS.md#operating-principles)'s context-gathering and existing-tests rules:
1. Read crate-level `README.md` and `HIGH_LEVEL_GUIDE.md` when available.
2. Read existing tests for expected behavior.

## Testing and Validation

- Run targeted crate tests first: `cargo test -p <crate> <test_name> -- --nocapture`
- For trace-driven tests, initialize tracing with:

```rust
let _tracing = init_test_tracing!(&graph);
```

- Use `target/test-logs/` for full debug output when tests fail, per [AGENTS.md](../../../AGENTS.md#operating-principles)'s test-log rule.
