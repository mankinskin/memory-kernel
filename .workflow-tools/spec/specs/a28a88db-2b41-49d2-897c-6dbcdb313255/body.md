<!-- aligned-structure:v2 -->

# Summary

Replace the always-on generated instruction load with one minimal bootstrapper plus discoverable, pinnable rule entries that are rendered into a focused per-session instruction set.

## Motivation ("why")

Universal `applyTo: "**"` instruction files inject unrelated ticket, spec, commit, and token-efficiency guidance into every task. The canonical rule store already makes those bodies searchable; sessions need a small discovery contract and an explicit render path instead of unconditional expansion.

## Dependent expectation

If this spec is implemented, dependents can rely on only the minimal bootstrapper being universal, the large instruction bodies remaining searchable and available on demand, and a session rendering exactly its pinned rule entries without expanding unrelated rules.

## Guards

- `val-session-selective-rule-render`.
- `val-rule-selective-instruction-generation`.
- `val-rule-sync-targets`.

## Positions

- Canonical searchable rule entries: `implemented` at `.rule/rules/` and `memory-api/crates/rule-api/src/store/store_query.rs`.
- Generated instruction target selection: `implemented` at `rule-targets/50-agents-instructions.yaml`; ticket/spec/commit are description-only generated artifacts and token-efficiency retains narrow path globs.
- Minimal universal bootstrapper: `implemented` at `.agents/instructions/session/session-bootstrap.instructions.md` from canonical rule `89330b3b-4d28-4c48-80dd-203311dbe855`.
- Session-side pinned-rule rendering: `implemented` at `memory-api/crates/session-api/src/store.rs`.
- CLI render surface: `implemented` at `memory-api/tools/cli/session-cli/src/lib.rs`.
- MCP render surface: `implemented` at `memory-api/tools/mcp/session-mcp/src/server.rs`.

## Governing-rule requirement

This contract is governed by `.agents/instructions/spec/spec-system.instructions.md` and its aligned-structure v2 requirement.

# Contract

- Exactly one generated bootstrapper instruction uses `applyTo: "**"`.
- The bootstrapper stays below 500 tokens and directs an agent to initialize/resume session context, search rules, pin relevant rule URNs, and render the pinned instruction set.
- `ticket-system.instructions.md`, `commit.instructions.md`, and `spec-system.instructions.md` remain deterministic generated artifacts with description-only discovery and no universal `applyTo`; `token-efficiency.instructions.md` retains only its narrow file globs.
- Their canonical `.rule` entries remain indexed and discoverable through representative rule searches.
- Session rendering accepts the current workspace session ID, resolves only pinned rule URNs from the sibling rule store, preserves canonical deterministic ordering, and returns generated markdown containing only those rule bodies.
- Non-rule pins never enter the rendered instruction set. Missing, malformed, cross-workspace, or non-rule references fail explicitly rather than broadening the render.
- Rule target generation remains deterministic and `rule sync-targets --check` passes.

# Non-goals

- Changing the canonical rule manifest or search index formats.
- Automatically selecting rules from vague semantic matches.
- Rendering ticket or spec bodies into the session instruction set.
- Removing narrowly scoped instruction files for concrete code paths.

# Acceptance Criteria

1. The generated always-on instruction set contains only the sub-500-token bootstrapper; the four large bodies are absent from that set.
2. Representative searches find canonical ticket-state, commit-hook, spec-authoring, and token-efficiency rule entries.
3. Spec-system guidance appears exactly once in its generated on-demand artifact.
4. `rule sync-targets --check` is deterministic on the reduced universal set.
5. Given pinned rule entries plus unrelated pins, session rendering emits exactly the pinned rule bodies in deterministic order.
6. Missing or invalid pinned rule references fail explicitly without rendering unrelated content.
7. Session CLI and MCP expose the focused render operation consistently.

# Traceability

- Parent spec: `8c880efc-7083-4e1d-bf06-96b8254be913`.
- Implementation ticket: [b4a8dc5e Minimal bootstrapper + selective instruction loading](memory-api/.ticket/tickets/b4a8dc5e-9d80-4fea-bb42-0c30aba0ecd6/ticket.toml).
- Pin/view dependency: `6b2dc497-188c-44f5-9106-bf35deecb7a1`.
- Design dependency: `afa00b5c-c736-4d75-b157-d3e9ce90d819`.
- Selective render evidence: [exec-val-session-selective-rule-render-20260714](.test/default/executions/exec-val-session-selective-rule-render-20260714.json).
- Generation/discovery evidence: [exec-val-rule-selective-instruction-generation-20260714](.test/default/executions/exec-val-rule-selective-instruction-generation-20260714.json).
- Deterministic sync evidence: [exec-val-rule-sync-targets-20260714](.test/default/executions/exec-val-rule-sync-targets-20260714.json).
