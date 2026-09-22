# Goal

Publish a canonical profiling/tracing phase taxonomy so phase-level timings and completion events are queryable and comparable across observability surfaces.

## Scope

- Canonical phase-key naming contract.
- Canonical phase-key sets for core operation families.
- Required phase-emission fields and compatibility/validation rules.

## Implementation evidence

Implemented in owning observability architecture spec:

- `.spec/specs/aa769a27-2721-4b9d-880c-5c4e2f8136a7/body.md`
  - Section: `Canonical profiling and tracing phase taxonomy`
  - Defines stable snake_case phase-key naming rules
  - Defines canonical phase keys for `open_or_init`, `scan`, integration path, workflow recompute, move apply/resume/rollback, and graph replay
  - Defines required fields per phase emission (`phase_key`, ids, component, operation kind, elapsed and deterministic counts)
  - Defines compatibility/deprecation and validation rules

Traceability link added in the same spec to this ticket.

## Validation linkage to ff6637f5 checklist outputs

This ticket provides direct contract support for `ff6637f5` checklist evidence:

- Bench/e2e phase timings can now align to canonical `phase_key` names for integration and workflow recompute families.
- Standardized metadata linkage (`operation_id`, `run_id`, optional `journal_id`) is required on phase emissions.
- Distribution reporting (`p50/p95/p99`) is explicitly modeled as profile artifacts by `phase_key`, not replay payload data.

## Depends-on context

Parent tracker: `84673399`.
Cross-tracker evidence target: `ff6637f5` validation checklist.