## Goal
Add a reverse-sync workflow for rule-generated artifacts so edits in a generated file can be written back to the originating rule bodies by canonical rule id.

## Problem
Forward generation already emits `rule-api:entry` markers that include canonical rule ids, but reverse update today only supports heading-based import that creates new slugs. This prevents safe in-place round-tripping for generated artifacts like agent files.

## Scope (v1)
- Add `rule sync-rules --file <path>` for non-spec-doc generated artifacts.
- Parse generated artifact entry markers and reconstruct per-entry bodies in a way that inverts markdown rendering semantics.
- For each parsed entry id: compare against store value and update body when changed.
- Support `--dry-run` and `--check` to report drift without mutating store.

## Non-goals (v1)
- No `--config` + `--target` convenience selector.
- No stale-output deletion or generated-target bookkeeping changes.
- No spec-doc reverse-sync support.

## Acceptance Criteria
1. `sync-rules --file` rejects non-generated files with a clear guard error.
2. `sync-rules --file` rejects spec-doc artifacts with a clear unsupported error.
3. `sync-rules --file` updates only existing rules referenced by entry ids.
4. Unknown entry ids fail with an orphan-id error that identifies missing ids.
5. `--dry-run` returns per-entry change status and performs no writes.
6. `--check` exits non-zero when any rule body would change.
7. Frontmatter round-trip is explicit: when a generated artifact has leading YAML frontmatter above `rule-api:file generated=true`, reverse-sync re-attaches that frontmatter to the first entry body so a subsequent generate check is clean.
8. Reverse-sync explicitly inverts forward normalization semantics: parsed entry bodies are reconstructed to match forward behavior for body trimming and line-ending normalization (CRLF/LF), so no normalization-only drift remains after reverse-sync + regenerate.
9. Apply is atomic at command scope: if any entry fails validation or persistence, no entry body updates are committed.
10. Marker identity semantics are explicit: `id` is authoritative for lookup and update; `slug` is metadata only and does not affect selection. A mismatched or edited slug does not redirect updates by slug.
11. Round-trip regression uses a frontmatter-bearing generated artifact (roast agent target), then edits an entry body, runs sync-rules, and verifies `generate-target --check` is clean.

## Traceability
- Implementation ticket path: C:/Users/linus/git/graph_app/workflow-tools/.workflow-tools/ticket/tickets/331f331d-b618-42db-9284-195c2e410a11

## Evidence Plan
- Unit tests for generated-artifact parser and apply loop.
- Command tests for guard failures, unknown ids, dry-run/check behavior, id-authoritative marker handling, and atomic-failure behavior.
- Integration-style round-trip regression test covering frontmatter, trimming, and line-ending normalization on a real generated target path.
