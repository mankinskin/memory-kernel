Track root: [bbb4bce9 Structured Ticket Entities (track root)](http://localhost:3002/workspace/default/ticket/bbb4bce9)

| Ticket | Title | Owns spec criteria |
|---|---|---|
| [5a3d152c Ticket parts: parts/ storage, [[parts]] manifest index, and core kind vocabulary](http://localhost:3002/workspace/default/ticket/5a3d152c) | Ticket parts: parts/ storage, [[parts]] manifest index, and core kind vocabulary | 1 |
| [3d952036 Part-addressed writes and mandatory description_mode](http://localhost:3002/workspace/default/ticket/3d952036) | Part-addressed writes and mandatory description_mode | 5 |
| [f9e70385 Plan freezing at `planned`: hard reject, amendment parts, unfreeze by state transition](http://localhost:3002/workspace/default/ticket/f9e70385) | Plan freezing at `planned`: hard reject, amendment parts, unfreeze by state transition | 2, 3, 4 |
| [9d69e93d Typed [[refs]] manifest table for external entity references](http://localhost:3002/workspace/default/ticket/9d69e93d) | Typed [[refs]] manifest table for external entity references | 6 |
| [4c7b884e Projected ticket reads: summary/plan/review/full profiles and explicit part lists](http://localhost:3002/workspace/default/ticket/4c7b884e) | Projected ticket reads: summary/plan/review/full profiles and explicit part lists | 7 |
| [f65f2b32 Migrate existing descriptions into typed parts (dry-run then apply, lossless)](http://localhost:3002/workspace/default/ticket/f65f2b32) | Migrate existing descriptions into typed parts (dry-run then apply, lossless) | 8 |
| [89fa0c25 ticket-viewer: render parts, frozen state, amendments, and typed refs](http://localhost:3002/workspace/default/ticket/89fa0c25) | ticket-viewer: render parts, frozen state, amendments, and typed refs | 1, 2, 7 (UI surface) |
| [71e13480 Update agent guidance and rule entries for parts, freezing, and projected reads](http://localhost:3002/workspace/default/ticket/71e13480) | Update agent guidance and rule entries for parts, freezing, and projected reads | behavioural adoption |

Dependency order: 5a3d152c → {3d952036, 9d69e93d} → f9e70385 (after 3d952036), 4c7b884e (after 9d69e93d) → {f65f2b32, 89fa0c25, 71e13480}.

Ticket store: memory-api/.ticket. Interview record: tmp/interview-structured-ticket-entities.md. Source request: transcripts/29-07-2026_ticket-entity-structure/input.clean.md.

Every ticket in this track is authored under the fixed heading convention the spec formalises (Objective / Requirements / Design / Examples / Acceptance Criteria / Typed References), with review and validation output kept out of the description. The track therefore acts as its own positive fixture set for the migration classifier in f65f2b32.
