# Entity Kernel Schema Versioning and Activation

## Objective
Define versioned domain manifests and separate entity-type schemas without changing current business schemas.

## Contract
Each domain has a manifest containing `domain_id`, an independent `domain_schema_version`, and entity-type memberships. Each membership has an explicit active/inactive flag. Each entity-type schema has its own independent `entity_type_schema_version`, fields, states, transitions, edge rules, required/terminal states, and version migration handlers.

Activation is explicit in the domain manifest. Inactive types may retain records for documentation or operational archiving; inactive records remain addressable but do not receive maintained migration, validation, or compatibility guarantees and do not block active-domain migration.

Domain-version changes track domain membership or policy. Entity-type-version changes track that entity type. Infrastructure schema evolution is separate from business field/state/transition evolution.

## Acceptance Criteria
- A domain manifest can declare multiple entity types with independent versions.
- Active and inactive memberships are persisted explicitly.
- Inactive retained records are reported and untouched by active migration.
- Version compatibility and migration-handler requirements are deterministic.
- Existing ticket/spec/rule/session/test/feedback business schemas are unchanged.

## Non-goals
No conversion of current domain schema files or business-schema changes in this specification pass.

## Evidence
- `transcripts/15-09-2026_entity-kernel-all-domains/02-schema-versioning.md`
- `workflow-tools/memory-kernel/src/model/schema.rs`
- `workflow-tools/memory-kernel/src/model/schema_registry.rs`
- `workflow-tools/rule/crates/rule-api/src/default_schema.rs`
