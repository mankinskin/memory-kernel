<!-- aligned-structure:v2 -->

# Motivation

Playwright, TypeScript checks, wasm-pack browser tests, and browser performance output currently produce tool-local reports. `test-api` already persists interoperable validation and benchmark records, but lacks a documented boundary for structured browser result ingestion and retained diagnostic artifacts.

# Dependent expectation

If this spec is implemented, dependents can rely on repository-native `npm`, Playwright, and `wasm-pack` subprocesses emitting a versioned structured payload that a thin test-api adapter validates and records as traceable `ValidationExecution` or `BenchmarkExecution` evidence. Human console text is diagnostic-only and is never parsed as the source of truth.

## Architecture and rejected alternatives

The runner invokes the owning repository command with a structured reporter/result-file option. The reporter emits one terminal result per test attempt plus a run envelope. A Rust adapter owns process exit interpretation, schema validation, durable test-api recording, artifact registration, and final aggregate outcome. This preserves native tool behavior and avoids embedding a Node runtime in Rust. A generic subprocess adapter is preferred over viewer-specific ingestion; browser/tool-specific reporters are thin producers only. Embedding Playwright or scraping console output is rejected because it couples test-api to tool internals and loses retry/artifact structure.

## Provenance and payload mapping

Every run envelope includes schema version, run ID, correlation/session ID, command, working directory, commit, source test path, test ID/title, project/profile, browser and transport, retry index and maximum, timestamps, and environment-manifest reference. Each test attempt includes status, duration, error classification, and artifact references for screenshots, traces, videos, frontend/backend logs, browser diagnostics, benchmark samples, and manifests.

- Playwright `passed` maps to `ValidationOutcome::Passed` only when the final retry attempt passes. `failed`, timed out, assertion failure, reporter parse failure, or nonzero process failure maps to `Failed`; earlier failed retries remain attached as attempt/artifact evidence and must not be discarded.
- Missing browser binary, unavailable WebGPU capability, intentionally unsupported profile, unavailable external service, or explicitly skipped required capability maps to `Blocked` with a reason and probe artifact. A generic skip without a declared capability reason is `Failed`.
- wasm-pack browser tests use the same validation envelope with `transport=wasm-pack`; a passing unit/browser test proves its stated behavior only, never a performance budget.
- Benchmark samples create `BenchmarkExecution` records with the workload, environment-qualified baseline reference, median/p95/p99/long-frame attachments, and budget decision. A regression or malformed sample is `Failed`; a missing required environment or baseline is `Blocked`.

Artifacts are retained as stable path or URI references outside the compact execution record, with a manifest binding each identity to content type, producer, correlation ID, and retention class. The fast PR profile retains failure diagnostics; release-browser retains all browser diagnostics; nightly performance/soak retains benchmark and environment evidence; hardware/on-demand retains adapter and driver evidence. The adapter links `spec_ids`, `ticket_ids`, and correlated `log_ids` into existing `ValidationLinks`, and writes `source_path`, `test_id`, `domain`, `operation`, `transport`, and `run_id` into existing `ValidationProvenance`. Retry, profile, commit, correlation ID, and artifact identities are retained in the reporter envelope/artifact manifest until the test-api model grows typed fields.

# Guards

- [`val-viewer-first-batch`](.test/default/specs/val-viewer-first-batch.json) is the aggregate guard for the first consumer of this design.
- The adapter implementation must add focused schema-validation, retry/outcome mapping, blocked-capability, provenance, and artifact-manifest tests before an execution is considered passing.

# Positions

- `implemented` — `memory-api/crates/test-api/src/lib.rs`: `ValidationExecution`, `ValidationLinks`, and `ValidationProvenance` provide the durable execution, traceability, and base provenance fields.
- `implemented` — `memory-api/crates/test-api/src/benchmark.rs`: `BenchmarkExecution` provides durable benchmark measurements and a baseline budget field.
- `not-implemented` — versioned browser reporter envelope and generic subprocess/result adapter.
- `not-implemented` — typed storage for retry, browser profile, commit, correlation ID, and artifact manifest identity beyond the current provenance fields.

# Governing-rule requirement

PolicyRule `84fa9769-cff9-4d89-9068-88474584b4b3` requires the ticket/spec routing used for this design. PolicyRule `397b0447-135e-4d35-ad05-bcc69047d2c0` requires browser-facing evidence to be validated through the browser quality gates.

# Traceability

- [93b8a331 Browser and TypeScript automated test integration strategy](memory-api/.ticket/tickets/93b8a331-da80-4fef-b13d-7f277cadb15f/ticket.toml)
- [956485ad Robust browser, observability, and performance validation strategy](memory-viewers/.ticket/tickets/956485ad-2e80-4a4c-b5ec-42bac2c7c295/ticket.toml)
- [e302d4c3 Cross-viewer measurable browser and GPU validation](.spec/specs/e302d4c3-c24f-4778-bef0-453d3c1997bb/spec.toml)
- [b06c9df8 Structured tracing for WASM frontend](viewer-api/.spec/specs/b06c9df8-2866-433a-af73-ae9b1f4a0f0a/spec.toml)
- [479e226a WASM tracing file sink](viewer-api/.spec/specs/479e226a-b4ef-4e30-ade0-ebdabbf956ed/spec.toml)