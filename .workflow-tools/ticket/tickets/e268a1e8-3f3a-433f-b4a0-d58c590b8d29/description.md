# Goal
Implement one concrete core-profile fixture and template smoke path for newly bootstrapped durable stores so the foundational memory-store contract is exercised by code, not only by policy text.

## Why this work exists
The hardened bootstrap spec now defines a core profile plus extension profiles, but the repository still lacks a proof point that a freshly bootstrapped store can perform the claimed minimum behavior without bespoke follow-up edits. This ticket turns the policy into an executable fixture or template smoke path.

## Scope
- Create one reusable minimal-store fixture or template path on shared `memory-api` primitives.
- Prove the core profile out of the box: create, get, update, delete, schema validation, nested-workspace resolution, and claimed baseline transport wiring.
- Leave explicit extension hooks for richer workflow semantics, attached artifacts, and domain-specific queries without requiring them for the core proof.
- Reuse existing workspace-resolution and schema-validation anchors instead of inventing a second bootstrap stack.

## Primary anchors
- spec `9ee9387f-5384-42a9-95c4-ecbad1713030`
- tracker `79dd2d35-267b-4395-8316-0761df45f3c5`
- `memory-api/crates/memory-api/src/workspace.rs`
- `memory-api/crates/spec-api/src/default_schema.rs`
- `memory-api/crates/rule-api/src/default_schema.rs`
- `memory-api/tools/cli/ticket-cli/tests/contracts_schema_validation.rs`

## Acceptance criteria
- One concrete fixture or template smoke path exists for the foundational store profile.
- The proof covers CRUD, schema enforcement, nested-workspace discovery, and the baseline transport surfaces claimed by the selected profile.
- Extension-specific behavior stays optional and layered, with explicit hooks rather than hidden requirements in the core path.
- Validation results are captured through focused commands or tests that future bootstrap work can reuse.

## Validation expectations
- Run the smallest focused test set that proves the chosen fixture or template path.
- Re-run any directly affected schema-validation or workspace-resolution checks.
- Record any remaining blocker if transport parity or extension hooks still need follow-up tickets.