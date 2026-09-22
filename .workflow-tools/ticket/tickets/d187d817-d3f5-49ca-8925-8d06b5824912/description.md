Implemented TOON machine-readable output across the memory-api CLI suite and extended spec-cli structured field decoding to accept TOON next to JSON.

Completed:
- added `--toon` output handling to ticket-cli, spec-cli, rule-cli, and audit-cli
- preserved existing JSON payload shapes while rendering TOON for success and error outputs
- extended spec-cli `--fields-file` decoding to accept JSON or TOON object payloads
- updated CLI READMEs and repo RTK guidance to prefer `rtk ... --toon` plus `toon-format` / `toon-rust`

Validation:
- passed `rtk cargo test -p ticket-cli --test contracts_command_schema command_schema_toon_is_machine_readable -- --nocapture`
- passed `rtk cargo test -p spec-cli parse_list_accepts_toon_flag -- --nocapture`
- passed `rtk cargo test -p spec-cli dispatch_structured_contract_fields_accept_toon_files -- --nocapture`
- passed `rtk cargo test -p spec-cli --test toon_output init_supports_toon_output -- --nocapture`
- passed `rtk cargo test -p audit-cli cli_supports_toon_output -- --nocapture`
- blocked `rtk cargo test -p rule-cli --test toon_output init_supports_toon_output -- --nocapture` due existing upstream `rule-api` compile errors in `memory-api/crates/rule-api/src/targets.rs` (`expected String, found Option<String>`).