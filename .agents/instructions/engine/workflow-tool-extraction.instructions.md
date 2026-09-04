---
description: "Use when planning a workflow-tool crate extraction. Covers repository-level dependency-cycle checks and remediation decisions."
applyTo: "memory-api/tools/**"
---

## Pre-Extraction Check

Before extracting a crate, map dependencies across the source repository and every consuming or providing repository. Record cycle nodes and their resolved or blocking status; one workspace's Cargo graph is insufficient.

- The resolved `ticket-api`/legacy-base cycle arose because `ticket-api` depended on legacy base while memory-api leaf crates depended on `ticket-api`.
- The `test-cli` -> `log-api` -> `test-api` cycle blocks test and log tool extraction.

Do not extract the test or log tool until the relevant ticket or spec records a remediation decision. See [69eb4118](../../../.ticket/tickets/69eb4118-19ec-4b5b-bb12-30e314029cc5/ticket.toml) and [858c5286](../../../.ticket/tickets/858c5286-6c2b-4a05-a0f3-4e8f6b90b75e/ticket.toml).

This file owns extraction architecture: [worktree-workflow.instructions.md](../commit/worktree-workflow.instructions.md) covers branch mechanics, and [submodule.instructions.md](../commit/submodule.instructions.md) would incorrectly restrict the check to submodules.