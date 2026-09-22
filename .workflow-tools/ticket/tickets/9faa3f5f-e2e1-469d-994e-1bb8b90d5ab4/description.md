## Implementation Complete

### Canonical Error Source
- `memory-api/src/workspace.rs:32-46` - `InvalidWorkspaceSelector` Display impl
- All servers now use `validate_explicit_workspace_selector` which produces this error

### Changes Per Server

**spec-mcp** (2 files):
- `src/server/types.rs:21` - CreateSpecInput workspace doc updated
- `tests/smoke_test.rs:521+` - Added `spec_workspace_validation_error` test (3 rejection cases)

**test-mcp** (2 files):
- `src/server.rs:206,219` - RecordSpecInput and RecordExecutionInput workspace docs updated  
- `tests/workspace_validation_test.rs` - New test file with 2 test functions validating error shape

**rule-mcp** (1 file):
- `src/server/types.rs:47,67` - ImportRuleFileInput and CreateRuleInput workspace docs updated

**feedback-mcp** (1 file):
- `src/server.rs:32,53` - IngestInput and QueryInput workspace docs updated

**session-mcp** (1 file):
- `src/server.rs` - 24 workspace field docs updated across all input structs
- Note: Cannot compile due to parallel agent's incomplete session-api changes

### Validation Results
- ✅ `cargo test -p spec-mcp spec_workspace_validation_error` - 1 passed
- ✅ `cargo test -p test-mcp workspace_validation` - 2 passed
- ✅ Both tests verify canonical error message shape

### Acceptance Criteria Status
1. ✅ Error messages use canonical `InvalidWorkspaceSelector` type
2. ✅ Schema descriptions document workspace constraints  
3. ✅ Tests added for spec-mcp and test-mcp (2 servers)
4. ✅ All accessible servers compile and pass new tests