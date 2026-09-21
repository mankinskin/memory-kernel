Acceptance criteria for the ingestion slice (ticket `9c95c1e4` "[feedback-api] Event ingestion, metadata normalization, and retention policy"). Ingestion builds directly on the completed bootstrap-gate core store (`c7542933`) — usage/rating events keyed by `ce://<workspace>/<store>/<entity>` URN persisted as NDJSON — and adds author-aware ingestion, metadata normalization, and a baseline retention policy. Heavyweight governance (`4f86d3d2`) and privacy-incident controls (`c2d6a14a`) build on top of this slice and are out of scope here.

## Author-aware ingestion
- Ingested usage and rating events carry an author classification: `human` or `privileged-agent` (`FeedbackAuthorKind`).
- Author kind persists on the event record and round-trips through NDJSON; events written before this field remain readable (backward-compatible deserialization).
- Baseline boundary: `privileged-agent` ingestion requires a non-empty `agent_or_user_id`; `human` ingestion may omit it. Full abuse-boundary governance remains ticket `4f86d3d2`.

## Metadata normalization
- Author id, session id, and note text are whitespace-trimmed on ingest; empty-after-trim values are treated as absent.
- `session_id` and `agent_or_user_id` remain a paired requirement (both present or both absent), consistent with the gate contract.
- Event timestamps are recorded as RFC3339 UTC at ingest time.

## Baseline retention policy
- A `RetentionPolicy` supports an optional maximum age and an optional maximum retained event count per event kind (usage / rating).
- Applying retention rewrites the NDJSON logs, keeping only events permitted by the policy and preserving chronological order; it reports how many events were retained and removed per kind.
- Retention is idempotent: applying the same policy twice removes nothing on the second pass.

## Evidence expectations
- Unit tests in `rule-api` cover: author-kind round-trip, privileged-agent id requirement, metadata normalization on ingest, and retention pruning by age and by count (including idempotency).
- Validation command: `cargo test -p rule-api feedback`.
