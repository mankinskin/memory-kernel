# [profiling] Validate browser profiling pipeline (trace capture + wasm benches)

Phase-1 implementation is landed and compiles; this ticket covers the remaining
browser-run validation to produce real evidence artifacts.

## Implemented (no-op to verify here)

- `profile-browser` feature, `profile_scope!` macro, `graph3d::render_frame`
  instrumentation, tracing-wasm timeline config, Playwright `withBrowserTrace`
  helper, `graph3d-profiling-suite.ts`, `tests/graph3d_bench.rs`.

## Acceptance Criteria

- [ ] `wasm-pack test --chrome --headless viewer-api/viewer-api/frontend/dioxus`
      runs the two benches and prints per-iter µs for `bench_perspective_projection`
      and `bench_look_at_view_matrix`.
- [ ] A viewer is served from a `--features profile-browser` trunk build and the
      `graph3d-profiling-suite` produces `playwright-report/profiles/<viewer>-graph3d.json`.
- [ ] The captured trace contains `graph3d::render_frame` marks under the
      `blink.user_timing` category (confirms tracing-wasm → performance timeline).
- [ ] Wire `registerGraph3dProfilingSuite` into at least one viewer's
      `e2e-release` suite registrar (ticket-viewer or spec-viewer).
- [ ] Browser verification in external Chromium; record window resolution.

## Commands

```bash
# wasm benches
wasm-pack test --chrome --headless viewer-api/viewer-api/frontend/dioxus
# profiling trace (serve with feature first)
cd viewer-api/viewer-api/frontend/dioxus && trunk serve --features profile-browser
```

## Files

- src/profiling.rs, src/graph3d/render.rs, src/tracing_setup/mod.rs, Cargo.toml
- e2e/shared/profiling.ts, e2e/shared/suites/graph3d-profiling-suite.ts
- tests/graph3d_bench.rs