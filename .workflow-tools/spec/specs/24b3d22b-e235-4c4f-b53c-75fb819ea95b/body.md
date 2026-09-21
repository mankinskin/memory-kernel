## Problem

A ticket entity today is a single mutable blob. Its directory holds `ticket.toml`, one `description.md`, `history.ndjson`, and an unused empty `assets/` scaffold (memory-api/crates/ticket-api/src/model/filesystem.rs). All metadata lives in one untyped `extra` map (memory-api/crates/memory-api/src/model/entity.rs).

`update_ticket` defaults `description_mode` to `replace` and unconditionally overwrites `description.md` with no shrink guard and no lifecycle guard (memory-api/crates/ticket-api/src/storage/store.rs). Two failure modes follow:

1. **Destructive overwrite.** An agent adding a review result replaces the whole objective. Recovery exists only via manual `undo` against `history.ndjson`.
2. **Objective erosion by accretion.** Agents avoiding overwrite append instead, so review, status, validation, and handoff content accumulates inside the objective. Over 50 tickets already carry `## Review` / `## Status` / `## Validation` / `## Handoff` headings in their description; the largest are 1286, 1280, and 1088 lines. The stable requirement becomes unfindable, and every read pulls the entire history into an agent's context.

ticket-api has no section API (spec-api has one), no attachments, no read projection, and no freeze mechanism.

## Goal

Make a ticket a rich, deeply grounded, self-contained mini-plan: a set of typed content parts stored as separate files, indexed by the manifest, with planning parts frozen once the ticket is planned, external context reached through typed references rather than inlined, and reads projected so an agent loads only the parts its role needs.

## Model

### Parts

A ticket directory gains `parts/`, holding one markdown file per part. `ticket.toml` gains a `[[parts]]` table parsed by the ticket manifest model in `memory-api/crates/ticket-api/src/model/filesystem.rs`, so the `[[parts]]` and `[[refs]]` schemas are defined in one place. Each `[[parts]]` entry carries a stable opaque `id` assigned at creation, `kind`, `path`, `frozen`, `created_at`, and optional `supersedes`. Manifest order remains the display/creation order, but the index is display-only and is never an addressing key; parts are addressed by `id`.

Core part kinds are a schema-validated vocabulary understood by projections:

| Kind | Role |
|---|---|
| `objective` | The stable statement of what this ticket exists to achieve |
| `requirements` | Exact requirements the implementation must satisfy |
| `design` | Chosen approach and structure |
| `examples` | Concrete illustrative cases |
| `acceptance_criteria` | Verifiable conditions for closing |
| `review` | Review findings, one part per review pass |
| `validation` | Test and validation evidence |
| `notes` | Working notes |
| `amendment` | A correction that supersedes a frozen planning part |

Free-form kinds outside this vocabulary are permitted and stored as opaque attachments. They are preserved, listed, and retrievable, but projections treat them as untyped. The manifest schema defines optional `supersedes` from the start, even though only `amendment` parts populate it with the frozen part id they supersede.

### Freezing

Entering `planned` freezes every planning part: `objective`, `requirements`, `acceptance_criteria`, `design`, and `examples`. `review`, `validation`, `notes`, `amendment`, and free-form parts remain writable in every state, so recording progress never requires touching the plan.

A write targeting a frozen part is **hard rejected** with an explicit error. The correction is recorded as a new `amendment` part referencing the frozen part it supersedes. Nothing is silently lost and superseded intent stays visible. There is no `--force`.

Unfreezing happens only by transitioning the ticket back to a pre-`planned` state. Re-entering `planned` re-freezes and cuts a new plan revision. Re-planning is therefore visible in ticket state and history, and no separate unfreeze axis exists. There is no privileged bypass: migration uses this same unfreeze-by-state-transition path before it can split a planned ticket, then re-enters `planned` afterward.

### Write API

`description_mode` loses its default. Callers must state `replace` or `append` explicitly; an omitted mode is an error, not a silent overwrite. This is a breaking change to every existing call site and is intentional — the silent default is the direct cause of the overwrite failures above.

### Typed references

`ticket.toml` gains a `[[refs]]` table for references to non-ticket entities: spec, test execution, log, rule, file path, commit. Each entry carries `kind`, a canonical URN, and an optional note. This absorbs the existing `related_specs` field. Edges (memory-api/crates/memory-api/src/model/edge.rs) remain ticket-to-ticket only and keep expressing dependency structure.

### Projected reads

Reads accept a named view profile or an explicit part list. Profiles are role-shaped bundles, each matching one agent job:

| Profile | Parts |
|---|---|
| `summary` | metadata + `objective` |
| `plan` | metadata + `objective` + `requirements` + `design` + `examples` + `acceptance_criteria` + refs |
| `review` | metadata + `acceptance_criteria` + `review` + `validation` |
| `full` | everything, including free-form parts |

`--parts a,b,c` selects an explicit set for cases no profile covers.

### Migration

A one-shot migration splits existing descriptions. It recognises `## Review`, `## Status`, `## Validation`, and `## Handoff` headings and moves those sections into their typed parts. It creates one `notes` part per matched heading and never merges multiple `notes`-kind sections into a single part. Heading provenance is preserved. All content it cannot confidently classify — unrecognised headings, mid-description asides, prose fitting no core kind — stays in `objective` verbatim. Nothing is lost and nothing is guessed. The tool produces a dry-run report first; apply is a separate step. If a ticket is already in `planned`, migration first transitions it back to a pre-`planned` state, performs the split, and then re-enters `planned` so the plan is re-frozen and a new plan revision is cut.

### Validation evidence

Each acceptance criterion is evidenced by a test-api validation execution linked to the implementing ticket.

## Non-goals

- Reworking the edge model or migrating existing free-form edge kinds.
- A general query expression language over tickets. Profiles plus an explicit part list are the whole projection surface for this spec.
- Retention limits or compaction of `history.ndjson`.

## Acceptance criteria

1. A ticket directory holds `parts/` files indexed by a `[[parts]]` manifest table; each entry carries a stable opaque `id`, `kind`, `path`, `frozen`, `created_at`, and optional `supersedes`; core kinds are schema-validated and free-form kinds round-trip as opaque attachments.
2. Transitioning a ticket to `planned` marks all five planning parts frozen in the manifest.
3. A write to a frozen part is rejected with an actionable error naming the part id and the amendment path; no `--force` exists.
4. Transitioning back to a pre-`planned` state clears the frozen flags; there is no privileged bypass path, and re-entering `planned` re-freezes.
5. `update_ticket` with a `description` and no `description_mode` is rejected.
6. `[[refs]]` round-trips all six external entity kinds and existing `related_specs` values migrate into it without loss.
7. Each of the four view profiles returns exactly the parts tabulated above, and `--parts` returns exactly the requested set.
8. Migration dry-run reports, for every affected ticket, which sections move and which content stays in `objective`; apply preserves heading provenance, creates one `notes` part per matched heading without merging notes sections, and migrates a ticket already in `planned` by transitioning it back to a pre-`planned` state, performing the split, and re-entering `planned` to cut a new plan revision.
9. A legacy ticket with no `[[parts]]` table is still readable: view profiles and `--parts` treat `description.md` as the sole `objective` part.