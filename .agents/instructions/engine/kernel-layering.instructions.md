---
description: "Use when changing memory-kernel, transport-harness, or domain extensions of their generic contracts."
applyTo: "{workflow-tools/memory-kernel/**,memory-api/**}"
---

## Neutral Kernel Boundary

`memory-kernel` owns generic filesystem-backed storage, indexing, search,
workspace, board, and cross-store move primitives. `transport-harness` may own
transport-generic startup mechanics. Neither crate may own a workflow domain's
schema, entity semantics, command dispatch, MCP server handler, HTTP router
registration, or domain-specific interoperability trait.

## Domain Extensions

Express domain-specific specialization of a generic kernel, harness, or test
fixture as an extension trait owned by the domain. An extension trait may
implement behavior for a generic receiver, but its methods and return values
remain domain-specific. `TicketManifestExt` in `ticket-api` is the reference:
ticket behavior layers over neutral manifest extra-map keys without making
ticket semantics part of the kernel.

When a contract appears in both the kernel and a domain crate, the crate that
owns the contract's semantic scope defines it. `memory-kernel` owns the neutral
`InteroperableArtifact` contract; `test-api` re-exports the contract and owns
test-specific extensions such as `TraceableArtifact`.

This boundary belongs here, rather than in
[core-crates.instructions.md](core-crates.instructions.md), because the
core-crates guidance applies only to the `context-stack` layers. It derives
from R3 and R4 of
[182940eb repository architecture dependency policies](../../../.spec/specs/182940eb-0df3-4fa0-8aff-2abce6095708/body.md).