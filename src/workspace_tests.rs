use crate::discovery::STORE_MARKERS;
use tempfile::tempdir;

use super::*;

#[path = "workspace_tests/discovery.rs"]
mod discovery;
#[path = "workspace_tests/paths.rs"]
mod paths;
#[path = "workspace_tests/selectors.rs"]
mod selectors;
#[path = "workspace_tests/store_resolution.rs"]
mod store_resolution;
