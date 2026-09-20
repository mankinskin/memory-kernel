<!-- aligned-structure:v1 -->

# Summary

Add a compact TOON machine-readable format alongside existing JSON output across the memory-api CLI suite.

## Behavior Story

Add a compact TOON machine-readable format alongside existing JSON output across the memory-api CLI suite.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Goal
Add a compact TOON machine-readable format alongside existing JSON output across the memory-api CLI suite.

# Scope
- Add a global `--toon` output flag next to `--json` for `ticket`, `spec`, `rule`, and `audit`.
- Keep existing human-readable output unchanged.
- Ensure machine-readable success and error payloads render as TOON when requested.
- Extend structured file-reading inputs that are currently JSON-only so they accept TOON payloads too.
- Document TOON usage together with `rtk` as the preferred compact machine-readable CLI workflow.

# Acceptance Criteria
1. Each CLI supports `--toon` output for successful command payloads and formatted errors.
2. `--json` behavior remains backward compatible.
3. The existing structured file-based input path for spec field maps accepts either JSON or TOON.
4. Focused tests cover TOON output and TOON input decoding on the touched CLI surfaces.
5. CLI docs describe when to use `--toon` with `rtk`.

# Traceability
- Implementation ticket: [d187d817 Add TOON input and output support](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/workflow-tools/.workflow-tools/ticket/tickets/d187d817-d3f5-49ca-8925-8d06b5824912/ticket.toml)
- Updated docs:
  - [ticket-cli README](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/tools/cli/ticket-cli/README.md)
  - [spec-cli README](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/tools/cli/spec-cli/README.md)
  - [rule-cli README](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/tools/cli/rule-cli/README.md)
  - [audit-cli README](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-api/tools/cli/audit-cli/README.md)
  - [repo RTK guidance](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/.github/copilot-instructions.md)

# Validation
- ValidationSpec: ticket-cli TOON output contract for `export-command-schema`.
- ValidationExecution: passed `cargo test -p ticket-cli --test contracts_command_schema command_schema_toon_is_machine_readable -- --nocapture`.
- ValidationSpec: spec-cli parser accepts `--toon`.
- ValidationExecution: passed `cargo test -p spec-cli parse_list_accepts_toon_flag -- --nocapture`.
- ValidationSpec: spec-cli structured `--fields-file` decoder accepts TOON.
- ValidationExecution: passed `cargo test -p spec-cli dispatch_structured_contract_fields_accept_toon_files -- --nocapture`.
- ValidationSpec: spec-cli TOON success output for `init`.
- ValidationExecution: passed `cargo test -p spec-cli --test toon_output init_supports_toon_output -- --nocapture`.
- ValidationSpec: audit-cli TOON success output for `run`.
- ValidationExecution: passed `cargo test -p audit-cli cli_supports_toon_output -- --nocapture`.
- ValidationSpec: rule-cli TOON success output for `init`.
- ValidationExecution: blocked `cargo test -p rule-cli --test toon_output init_supports_toon_output -- --nocapture` by pre-existing compile errors in `memory-api/crates/rule-api/src/targets.rs` (`Option<String>` vs `String` mismatches at lines 409, 410, 422, 472, and 473).
