# Multi-Client Guidance Rendering

## Problem

The repository maintains agent-facing guidance (instructions, agent definitions, prompts, skills, root guidance) as hand-owned markdown under `.agents/**`, `AGENTS.md`, and `.github/copilot-instructions.md`. Every additional agent client (Cline, OpenCode, and future clients) needs the *same* guidance expressed in a *different* container format: different frontmatter key sets, different file layout, different discovery entry point.

Maintaining N near-identical copies that differ only by consumer protocol is unsustainable. The rule system already solves the hard part — canonical fragments with deterministic composition — and `.clinerules/` proves the projection works. What is missing is the ability to vary the *rendering* per client.

## Prior Decision Reversal

An earlier effort (ticket `14c0995c` and children) decoupled `.agents/**` from the rule generator because the rule-target system added complexity without eliminating meaningful duplication. That deletion has already landed: 846 lines of agent-guidance target config were removed and all provenance markers stripped.

That judgement was correct **for single-client rendering**. It does not hold for multi-client rendering, which is the use case this spec covers. This spec supersedes that direction rather than reverting it: the track builds forward from today's hand-owned files as the import source.

## Structural Gap

Three properties block a multi-client design today:

1. **No templating engine.** Rendering is `String::push_str` concatenation in `memory-api/crates/memory-api/src/generated_markdown.rs`. No handlebars/tera/minijinja/askama dependency exists anywhere in the workspace.
2. **No front-matter field on `RenderTarget`.** The resolved target model carries only `repo_scope`, `file_kind`, `path_scope`, `section`, `state`, `nodes`, `output_path`.
3. **Front matter lives inside rule bodies.** `skip_provenance_for_yaml_frontmatter` hoists the *first matched entry's* YAML block above the provenance comment. This makes the shared fragment client-specific, which defeats the entire premise: two clients needing different frontmatter for the same guidance require two different rule bodies.

## Confirmed Decisions

| Decision | Resolution |
| --- | --- |
| Canonical source | The `.rule/` stores. All client outputs are generated. |
| Committed vs generated | All client outputs are gitignored and produced at install time. Root `AGENTS.md` is the sole exemption. |
| Bootstrap layer | `AGENTS.md` stays committed and directs the reader to committed install instructions. |
| Front-matter location | Structured metadata fields on the rule entry; `body.md` becomes pure prose. |
| Rendering mechanism | A Rust templating engine (`minijinja`) plus per-client template files that read the structured metadata. |
| Client set (v1) | GitHub Copilot / VS Code, Cline, OpenCode. |
| Install surface | Both a `rule install --client <name>` subcommand and a top-level `install-guidance.sh` convenience wrapper. |
| Install configuration | A committed availability manifest plus a gitignored per-machine selection lockfile. Surfaces are CLI-configurable. |
| Authoring loop | Edit the generated file, then `rule sync-rules` reverse-syncs it back into the store. |
| Prior decommissioning tickets | Closed as done; the new track builds forward. |
| Drifted rule bodies | Current hand-owned files are re-imported as the source of truth; the 96 stale orphans are retired. |
| Untracking strategy | Big-bang: a single commit untracks `.agents/**` once the generator is proven. |

## Scope

Generated surfaces in v1:

- Instruction files (`.agents/instructions/**`)
- Agent definitions (`.agents/agents/*.agent.md`)
- Prompt files (`.agents/prompts/*.prompt.md`)
- Skill files (`.agents/skills/*/SKILL.md`), including vendored-skill installation
- Root `AGENTS.md` and `.github/copilot-instructions.md`
- Client entry config (`opencode.json` instructions pointer, `.vscode/settings.json` discovery wiring)
- Hook manifests (`.github/hooks/`, `.clinerules/hooks/`)

## Non-Goals

- Reverting the deleted rule-target configs. The track re-imports from current files instead.
- Supporting clients beyond Copilot, Cline, and OpenCode in v1.
- Replacing `skills-lock.json` as the vendoring mechanism; it is invoked by the installer, not superseded.
- Machine-specific absolute path addressing anywhere in generated-target state.

## Architecture

```
.rule/ stores  ──(structured metadata + prose body)──┐
                                                      │
guidance-install.toml (availability manifest)         │
        │                                             ▼
        ├──> rule install --client <name> ──> minijinja client profile
        │                                             │
        └──> .guidance-install.lock (per machine)      ▼
                                          gitignored client outputs
                                          (.agents/**, .clinerules/**,
                                           opencode.json, hooks, skills)
```

Reverse path: edit a generated file, then `rule sync-rules` parses provenance markers, re-attaches structured metadata, and writes back to the owning rule entry.

## Acceptance Criteria

1. A single canonical rule fragment renders correctly into Copilot, Cline, and OpenCode output without any client-specific text stored in the fragment body.
2. Frontmatter is produced by the client profile from structured rule-entry metadata, not hoisted from a body.
3. `rule install --client <name>` materializes every selected surface for that client, and re-running is idempotent.
4. `install-guidance.sh` provides the same capability through the existing install-script family shape.
5. Golden-file fixtures for each client profile render byte-identical.
6. Render → `sync-rules` → re-render is idempotent, including structured metadata.
7. A pre-commit drift gate fails when the rule store and rendered fixtures disagree.
8. Each of the three clients loads its installed guidance in a live smoke test.
9. No generated-target state record contains a machine-specific absolute path.
10. `sync-targets` refuses to overwrite a marker-free file without an explicit force flag.
11. A fresh clone contains `AGENTS.md`, which directs the reader to install instructions; all other guidance surfaces are absent until install runs.

## Validation Plan

- Golden-file snapshot tests per client profile per surface.
- Round-trip idempotence test over the full store.
- Pre-commit drift gate covering rule store versus rendered fixtures.
- Live client smoke test for Copilot, Cline, and OpenCode.
- Fresh-clone bootstrap test: clone, confirm only `AGENTS.md` is present, run the installer, confirm all surfaces materialize.

## Risks

| Risk | Mitigation |
| --- | --- |
| `sync-targets` silently overwrites hand-owned files on first re-adoption | Overwrite protection ticket lands before any target is re-added |
| Reverse-sync is still a draft contract but is now load-bearing | Reverse-sync is an explicit hard blocker of the untracking cutover |
| Big-bang untracking loses per-file guidance git history | Untracking is gated behind byte-identical golden fixtures and the drift gate |
| 96 drifted orphan rule bodies conflict with current file content | Current files win; orphans are retired in a dedicated reconciliation ticket |
| Template files become an unmaintained surface | Golden fixtures cover every profile/surface pair |
