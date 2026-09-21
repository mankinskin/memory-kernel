## Decision Record

The full binding decision register is in [Full Interview Decision Register](sections/Full%20Interview%20Decision%20Register.md). This map assigns implementation ownership:

- [7ef3f8db Implement directed inherited schema lifecycle engine](.ticket/tickets/7ef3f8db-d4a9-4135-99eb-3c006070a328/ticket.toml): inheritance, directed lifecycle semantics, category refinement, cancellation, relation-graph separation, and shared primitives/namespaces.
- [1f8e6e6d Add deterministic dual-format schema loading](.ticket/tickets/1f8e6e6d-c8ea-461d-83c9-c26daf0e3cd3/ticket.toml): permanent TOML/JSON input compatibility, deterministic combined load order, diagnostics, collisions, and atomic retention.
- [abd3f280 Generate resolved schema catalog and JSON built-ins](.ticket/tickets/abd3f280-9bd1-48cf-8503-17dd820afb30/ticket.toml): resolved manifest, shipped JSON built-ins, and concrete schema catalog.
- [9e7a5f1a Integrate schema catalog into CLI and VS Code](.ticket/tickets/9e7a5f1a-a2ce-43ce-9c8f-bdce7cf712d2/ticket.toml): catalog-driven CLI/VS Code behavior and transport/client parity.
- [d8bd4c53 Research deterministic legacy-ticket classifier](.ticket/tickets/d8bd4c53-898e-4984-97e5-6ef605569f91/ticket.toml): mandatory Track 5 classifier rules, evidence, thresholds, and tie/conflict behavior before live migration.
- [7df984eb Inventory and migrate legacy ticket schemas](.ticket/tickets/7df984eb-c96c-4501-98fa-4e88dd28ec4e/ticket.toml): preflight, cutover/exemptions, legacy conversion, and approved transactional migration after d8bd4c53 completes.
- [3bb41fb2 Validate schema modernization release and repair flow](.ticket/tickets/3bb41fb2-9907-4b54-83b1-c62a0ce96756/ticket.toml): Track 5 completion gate, interface matrix, release validation, corrective repair, remediation review, and all-pass evidence.

Execution order remains engine -> loader -> catalog -> clients -> classifier research -> migration -> release validation. The epic depends on each track; Track 5 migration additionally depends on classifier research.