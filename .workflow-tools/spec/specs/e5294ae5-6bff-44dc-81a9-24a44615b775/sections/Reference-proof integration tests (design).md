Design for ticket 60114a17 (precursor to 2cc7680c). Resolves the reviewer's open
questions and satisfies the locked decisions (durable proof home in memory-kernel;
one op across CLI/MCP/HTTP; mandatory success AND error-envelope/status assertions;
no regression of `default = []` independently-selectable transports).

### Smallest realistic domain operation

`describe(id) -> Item` over a tiny fixed in-test registry. Not placeholder
product code, but a real request/response with a success and a failure branch:

- Registry (fixture): `{ "harness" => "Shared transport harness" }`.
- Success: `describe("harness") -> Item { id: "harness", summary: "Shared transport harness" }`.
- Failure: `describe(<unknown>) -> DomainError::NotFound(id)`.

This is the smallest op that still exercises input parsing, a structured success
payload, and a domain error that must be normalized differently per transport —
the mechanic the shared harness exists to provide.

### Exact success output shape

Canonical JSON item (serde): `{"id":"harness","summary":"Shared transport harness"}`.

- CLI: emitted through `Output::json(item)` + `write_output`, i.e. the JSON line
  followed by exactly one trailing newline.
- MCP: the same item value returned from the domain tool call.
- HTTP: `200 OK` with that JSON body.

### Exact error envelope / status per transport

Domain error message string: `unknown item: <id>`.

- CLI: `cli::run_from(..)` returns `Err(HarnessError::Domain("unknown item: <id>"))`;
  invalid/unparseable args return `Err(HarnessError::Arguments(..))`. Assert the
  variant and the message.
- MCP: the domain handler surfaces the same domain failure; assert it maps to a
  harness domain error carrying `unknown item: <id>` (transport-tagged context
  preserved via `HarnessError::Transport` only for startup/IO failures).
- HTTP: `404 NOT_FOUND` with `http::HttpError` envelope
  `{"code":"not_found","message":"unknown item: <id>"}`. Assert BOTH the status
  code and the JSON envelope (`HttpError::status()` mapping is the primary proof).

### memory-kernel test layout and harness resolution

- Location: `memory-kernel/crates/transport-harness/tests/reference_proof.rs`
  (an integration test in the harness crate). This keeps the harness self-proving
  and resolves the harness by path within its own workspace — no external crate,
  no git dependency needed for the proof itself.
- Run gate: `cargo test -p transport-harness --all-features` (CLI/MCP/HTTP modules
  all compiled). Individual-transport runs use the matching single feature.
- Fixture domain (`describe`, `Item`, `DomainError`, registry) lives inline in the
  test module — dev-only, never shipped in the library surface, so `default = []`
  slimness is untouched.
- CLI proof: call `cli::run_from([args], &mut Vec<u8>, dispatch)` — no process
  spawn; assert the buffer bytes (success) and the returned `HarnessError` (error).
- MCP proof: construct the domain `ServerHandler`, invoke the `describe` tool
  routing in-process; assert the success item and the domain-error mapping without
  needing a live stdio peer.
- HTTP proof: build the domain `Router`, bind `127.0.0.1:0` (ephemeral port),
  serve on a background task, and issue real requests to `/describe/{id}`; assert
  `200`+item and `404`+envelope.

### context-engine reference disposition

Retained as a thin, compile-only consumer. `workflow-tools-contract-reference`
keeps proving that the branch-pinned git dependency resolves and that all three
gated binaries build/run through the production harness (`default = []` preserved).
All behavioral success/error assertions live in the memory-kernel integration
test above; context-engine does not duplicate them.

### Acceptance

Accepted 2026-07-25: this design unambiguously specifies the domain op, the three
transport surfaces, the exact success output and error envelope/status per
transport, the memory-kernel test location + harness resolution, and the
context-engine reference disposition. It honors every locked review decision and
resolves all four open questions. Ready to hand to ticket 2cc7680c for
implementation.