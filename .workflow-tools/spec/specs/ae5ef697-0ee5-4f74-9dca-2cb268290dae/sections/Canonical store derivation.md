Every active domain derives its canonical store as `<local_workspace>/.workflow-tools/<domain>` through the shared workspace contract. The active domain set is Ticket, Spec, Test, Session, Feedback, Audit, and Log.

- A workspace-based create or write targets only the canonical path.
- If canonical storage is absent and a legacy `<local_workspace>/.<domain>` store exists, reads may use that legacy store and return a legacy diagnostic. Do not initialize legacy storage.
- If both layouts exist, select the canonical store and return `BothLayoutsPresent`; do not fail as ambiguous or silently select legacy storage.
- Preserve supported direct concrete canonical-store overrides. Reject legacy-shaped write overrides, including nonexistent explicit legacy paths, instead of creating or redirecting them.
- Read-only access to a missing store does not initialize a store.