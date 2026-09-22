# Goal

Create a single, domain-parameterized rendering abstraction that all store indexes (ticket, spec, rule, audit) can use, eliminating duplication and enabling consistent, testable formatting across domains.

## Problem

Currently, each store index has its own `render_entry_block` and `render_catalog_markdown` implementation in separate crates (ticket-api, spec-api, rule-api, audit-api). They all follow the same pattern (entry line + link + optional metadata) but are implemented with duplicated logic. Adding a new feature (like sorting or filtering) requires changes in four places. There is no single contract or abstraction for what "rendering a store index" means.

## Scope

**Per Q7 decision (hybrid trait-based abstraction):**

- Define a generic `StoreIndexRenderer<T>` trait in `memory-index/src/rendering.rs` (new module in memory-api):
  ```rust
  pub trait IndexEntryFormatter {
    /// Return the formatted text representation of a single entry
    fn format_entry(&self, entry: &DomainEntry) -> String;
    /// Return the group key for clustering entries
    fn group_key(&self, entry: &DomainEntry) -> String;
    /// Return the display name for a group
    fn group_title(&self, key: &str, count: usize) -> String;
  }
  ```
- Each domain implements `IndexEntryFormatter`:
  - **Ticket**: Formats as `[short_id title](path)`, groups by state, titles like "State: new (N tickets)"
  - **Spec**: Formats as `[indent] [slug](path)`, groups by component, indentation based on depth
  - **Rule**: Formats as `[slug-leaf](path)`, groups by three-segment prefix
  - **Audit**: Formats as `[path](path) — category · message`, groups by severity
  
- Generic `render_catalog` function:
  ```rust
  pub fn render_catalog<E, F>(entries: Vec<E>, formatter: &F) -> String
  where
    F: IndexEntryFormatter
  ```
  Applies the formatter's logic for grouping, ordering, and wrapping (collapsible or plain).

- Move `DomainEntry` trait to a shared definition (or use generics) so all domains can work with a common entry interface.
- Update each domain's `store_index.rs` to use the generic renderer with its specific formatter.
- Move and share ordering logic (effort→created_at, DFS, severity) into the generic renderer.
- Add tests to `memory-index/tests/rendering_*.rs` that verify each domain's formatter produces the expected output.

## Acceptance criteria

- `memory-index/src/rendering.rs` defines `IndexEntryFormatter` trait.
- All four domains (ticket, spec, rule, audit) implement the trait.
- Generic `render_catalog` function is the single code path for all index generation.
- Existing `*store-index --check` tests pass with identical output.
- New tests in `memory-index/tests/` verify formatter outputs for each domain.
- Code duplication across `*-api/src/store_index.rs` files is eliminated (DRY).

## Non-goals

- No change to sidecar formats or store structures.
- No UI changes — only rendering abstraction.
- No new features in this ticket — only consolidation.
