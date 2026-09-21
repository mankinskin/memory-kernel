# Validation and Traceability

## Review Status

**Specification reviewed on 2026-07-27.**

Related implementation ticket: [9185d8f2 Fail-open empirical tool-metrics gating](../../../memory-api/tickets/9185d8f2-1080-46b1-84da-485f9ad839f6/ticket.toml)

### Acceptance Criteria Summary

- **AC1–AC6**: ✅ All passed. Hardcoded constants removed, fail-open behavior verified, unit tests passing (30/30).
- **AC7** (live expensive-model MPC verification): ✅ **Satisfied** via ticket [56be2eaa](../../../memory-api/tickets/56be2eaa-6b32-4291-857a-b2c1aa24f273/ticket.toml) on 2026-07-27.
  - Added `verdict` subcommand for manual verification of gate decisions.
  - Added 14 integration tests:
    - 8 in-process proxy tests covering all 5 end-to-end gating scenarios (existing).
    - 2 stdio round-trip tests (`test_stdio_expensive_model_refused`, `test_stdio_cheap_model_allowed`) verifying binary startup, JSON-RPC framing, and MCP protocol layer with real child-process stdio communication.
    - 4 `verdict` subcommand tests verifying CLI interface (allow, delegate, reject, error cases).
  - Test suite: 44/44 passed (30 unit + 14 integration).

### Gating Logic Coverage

Empirical fail-open and graded-cost behavior is covered by:
- Unit tests: `evaluate_with_rollup`, `graded_budget_boundary_with_rollup`, `expensive_measured_tool_is_refused`
- Proxy integration tests: `unmeasured_tool_fail_open`
- Integration tests (tests/integration_gate.rs): 14 tests covering all gating scenarios plus binary/protocol layer verification

### Known Findings

**Substring over-match in `tool_cost()`**: Bidirectional substring matching between requested tool name and rollup keys (pre-existing, not introduced by this change) is tracked in [9c29d697 Reduce false-positive tool name matches in cost gate](../../../memory-api/tickets/9c29d697-f782-4737-aea1-645abf75cfb9/ticket.toml).

## Related Tickets

- **Deferred work completed**: [56be2eaa Integration test harness for live expensive-model gating verification](../../../memory-api/tickets/56be2eaa-6b32-4291-857a-b2c1aa24f273/ticket.toml)
- **Related finding**: [9c29d697 Reduce false-positive tool name matches in cost gate](../../../memory-api/tickets/9c29d697-f782-4737-aea1-645abf75cfb9/ticket.toml)
