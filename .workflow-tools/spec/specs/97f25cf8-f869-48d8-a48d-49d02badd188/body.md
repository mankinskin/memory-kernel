# Summary

Capture per-tool-call output size (`output_char_sizes`) at capture time in `session-api`, with an explicit per-call source attribution (`output_source`), so downstream cost-gate classification and rollups (see parent spec [29ae5f6e Empirical tool-metrics driven cost-gate classification](../29ae5f6e-c202-41f1-ba88-a446aa872993/spec.toml)) consume real observed sizes instead of guesses. This capture path must not add blocking latency to the capture hook's synchronous critical path, and a later-arriving fuller copy of a tool response must not be silently dropped in favor of an earlier, smaller one.

# Problem

`SessionStoreConfig::capture_copilot_transcript_with_tool_response` currently performs a synchronous blocking retry (`MAX_ATTEMPTS=12` × `RETRY_DELAY=200ms` via `std::thread::sleep`) on the hook's critical path while waiting for tool output to become available, adding up to ~2.4s worst-case latency to every hook invocation. Separately, `record_event_tool_call`'s first-wins dedup on `tool_call_id` means a later, fuller override of a tool response for an already-seen call is discarded instead of replacing the earlier (possibly empty/partial) one. Real-session validation of the current implementation showed only ~24% (7/29) of real tool calls end up with non-empty `output_char_sizes`, which is not sufficient signal for cost-gate classification, and `output_source` is not yet threaded into the cross-session `ToolAggregation` rollup.

# Scope

- Per-tool-call `output_char_sizes` capture at capture time in `memory-api/crates/session-api/src/tool_metrics.rs`.
- `output_source` attribution enum (`hook_payload` | `spill_file` | `transcript_turn` | `unspecified`) on `ToolCallSummary`, propagated through `ToolAggregation` into the cross-session rollup (`ToolTokenStats` / `ToolMetricsReport`).
- Moving retry/backfill of late-arriving tool output off the hook's synchronous critical path (async/non-blocking), replacing the current blocking sleep loop in `capture_copilot_transcript_with_tool_response`.
- Merge semantics for `record_event_tool_call` so a later, richer response for an already-seen `tool_call_id` overrides an earlier partial one instead of being dropped.
- Live-session coverage evidence for `output_char_sizes` against a concrete, confirmed threshold (see R5).

# Non-Goals

- Changing the `ToolTokenStats` percentile math or the cost-gate classification policy itself (owned by parent spec 29ae5f6e).
- Introducing a new tokenizer or token-estimation algorithm.
- Redesigning the hook registration/trigger model in `tools/agent-hooks/session-capture-stop.sh` beyond removing the blocking retry.
- Closing follow-up ticket 74b56d66 directly from this spec (it closes when 44119807 reaches `done`, per ticket workflow).

# Requirements (Acceptance Criteria)

1. **R1 — Capture-time size + attribution.** `output_char_sizes` is captured at capture time for tool calls, with each entry carrying a per-call `output_source` attribution (`memory-api/crates/session-api/src/tool_metrics.rs` `record_event_tool_call`, `ToolCallSummary.output_source`).
2. **R2 — Attribution propagates to rollup.** The `output_source` variants (`hook_payload`, `spill_file`, `transcript_turn`, `unspecified` fallback) propagate from capture through `ToolAggregation` (merge logic) into the cross-session rollup output (`ToolTokenStats` / `ToolMetricsReport`), not deferred to a later ticket.
3. **R3 — No blocking retry on the synchronous critical path.** The hook's synchronous critical path (`capture_copilot_transcript_with_tool_response`) performs no blocking retry/`sleep`. Any backfill of late-arriving tool output for a `tool_call_id` happens asynchronously/non-blocking relative to the hook's return path. Measurable budget: the synchronous capture path must complete in **≤50ms p95** added latency attributable to tool-output-size capture (excluding any pre-existing transcript I/O it already performed before this change); async backfill has no hard latency bound but must complete before the next aggregation read in the common case.
4. **R4 — Richer late copy wins.** When a later-arriving, fuller copy of a tool response arrives for an already-seen `tool_call_id`, it replaces the earlier partial/empty copy in `record_event_tool_call` rather than being discarded by first-wins dedup.
5. **R5 — Real-session coverage threshold.** Coverage, measured as (tool calls with non-empty `output_char_sizes`) / (total real tool calls in a live captured session), must be materially above the previously observed ~24% (7/29). **Proposed threshold: ≥90%**, evidenced by a fresh live-session capture (not the same 29-call sample used in the failed review pass). **This threshold needs explicit user confirmation before it is treated as a hard gate.**

# Traceability

- Ticket: [44119807 [tool-metrics][T2] Capture tool output size at capture time with per-call source attribution](../../../.ticket/tickets/44119807-53af-41b0-920a-ffbc985d425d/ticket.toml)
- Ticket (follow-up, closes when 44119807 reaches `done`): [74b56d66 [tool-metrics][T2 follow-up] Confirm live-session AC1 evidence for output_char_sizes via hook_payload](../../../.ticket/tickets/74b56d66-d94f-4422-bda6-5f583d8f7ec4/ticket.toml)
- Ticket (T1 probe, done): [76941e78 Probe: establish which source actually carries tool output size](../../../.ticket/tickets/76941e78-f812-440c-9fbc-04d3bb88f11a/ticket.toml)
- Parent spec: [29ae5f6e Empirical tool-metrics driven cost-gate classification](../29ae5f6e-c202-41f1-ba88-a446aa872993/spec.toml)
- E2E test: `memory-api/crates/session-api/tests/copilot_capture_hook_e2e.rs` (8 tests)
- Commit (memory-api submodule): `88960df8a53ea49e79301d01421bf738c29eb16f`
- Commit (outer repo): `2877e70b5bbd91569d018a44747bf9f3da33b720`

# Code References

- `memory-api/crates/session-api/src/store/config/capture_query.rs` — `capture_copilot_transcript_with_tool_response` (L61, body L67-L101, blocking retry L70-L97, race documented L47-L60).
- `memory-api/crates/session-api/src/tool_metrics.rs` — `record_event_tool_call` (L334-L362, first-wins dedup L359-L362); `ToolCallSummary.output_source` (L161-L164); `ToolAggregation` (L588-L598); aggregation merge (L456-L481); `ToolTokenStats` (L536-L555); `ToolMetricsReport` (L566-L585); `output_source` fallback (L387-L395).
- `memory-api/crates/session-api/src/hook/transcript.rs` L30-L40 — `ToolResponseOverride.output_source`.
- `memory-api/crates/session-api/src/bin/copilot-capture-hook.rs` L79 (call site), L116-L141 (sets `hook_payload`/`spill_file`).
- `memory-api/crates/session-api/src/hook/tool_execution.rs` L220-L233.
- Hook entry: `tools/agent-hooks/session-capture-stop.sh` L24-L27 (invokes binary inline, `Stop` trigger).

# Review Status

Third review pass (2026-07-30): AC1 conditional pass (real evidence confirmed, ~24% coverage = 7/29 real tool calls), AC2 pass, AC3 pass, AC4 pass, **overall FAIL** pending R3 (async retry redesign) and R5 (coverage threshold) closure.
