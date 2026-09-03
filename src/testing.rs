//! Generic test-harness primitives shared across all domains.
//!
//! Gated behind the `testing` Cargo feature so the types are never compiled
//! into production builds.  Consumers add:
//!
//! ```toml
//! [dev-dependencies]
//! memory-kernel = { path = "...", features = ["testing"] }
//! ```
//!
//! # Design
//!
//! [`SandboxSetup`] is an inversion-of-control trait that each domain
//! implements to specify how its directory layout is created inside a fresh
//! [`TempDir`].  [`Sandbox<S>`] owns the [`TempDir`] and delegates all
//! path-related concerns to the domain-supplied `S`.
//!
//! The domain implementation (`impl SandboxSetup`) is also responsible for
//! running any initialisation commands (e.g. `ticket init`) inside
//! [`SandboxSetup::setup`].  That keeps the generic struct free of
//! domain-specific binary calls.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use criterion::{Bencher, Criterion};
use tempfile::TempDir;

use crate::storage::move_kernel::{MoveBlocker, MovePlan};

/// Criterion's minimum accepted sample count for rough-magnitude benchmarks.
pub const MOVE_BENCH_SAMPLE_SIZE: usize = 10;
/// Short warm-up for filesystem-backed move benchmarks.
pub const MOVE_BENCH_WARM_UP: Duration = Duration::from_millis(50);
/// Upper bound for one heterogeneous scenario's measured phase.
pub const MOVE_BENCH_MAX_MEASUREMENT: Duration = Duration::from_secs(2);
/// Process-wide runtime cap for one benchmark executable, leaving cleanup
/// headroom below the requested ten-minute wall-time limit.
pub const MOVE_BENCH_MAX_RUNTIME: Duration = Duration::from_secs(9 * 60);

/// Derive a bounded measurement budget from one untimed calibration call.
pub fn move_bench_measurement_time(calibrated: Duration) -> Duration {
    calibrated
        .saturating_mul(MOVE_BENCH_SAMPLE_SIZE as u32)
        .mul_f64(1.2)
        .max(Duration::from_millis(100))
        .min(MOVE_BENCH_MAX_MEASUREMENT)
}

/// Construct the low-sample Criterion configuration shared by move benches.
pub fn move_bench_criterion() -> Criterion {
    let deadline = std::env::var("MOVE_BENCH_DEADLINE_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(MOVE_BENCH_MAX_RUNTIME)
        .min(MOVE_BENCH_MAX_RUNTIME);
    std::thread::Builder::new()
        .name("move-bench-deadline".to_string())
        .spawn(move || {
            std::thread::park_timeout(deadline);
            eprintln!(
                "move benchmark wall-time cap reached after {:.0}s",
                deadline.as_secs_f64()
            );
            std::process::exit(124);
        })
        .expect("start move benchmark deadline watchdog");
    Criterion::default()
        .sample_size(MOVE_BENCH_SAMPLE_SIZE)
        .warm_up_time(MOVE_BENCH_WARM_UP)
        .measurement_time(Duration::from_millis(500))
}

/// Run a benchmark while excluding fixture setup from the measured duration.
pub fn iter_move_benchmark<Input, Setup, Measure>(
    bencher: &mut Bencher<'_>,
    mut setup: Setup,
    mut measure: Measure,
) where
    Setup: FnMut() -> Input,
    Measure: FnMut(Input),
{
    bencher.iter_custom(|iterations| {
        let mut elapsed = Duration::ZERO;
        for _ in 0..iterations {
            let input = setup();
            let started = Instant::now();
            measure(input);
            elapsed += started.elapsed();
        }
        elapsed
    });
}

/// Remove only the two blockers intentionally created by isolated fixtures.
pub fn drop_fixture_blockers(plan: &mut MovePlan) {
    plan.blockers.retain(|blocker| {
        !matches!(
            blocker,
            MoveBlocker::PathReferenceScanUnavailable { .. }
                | MoveBlocker::DirtyTrackedFiles { .. }
        )
    });
}

/// Shared source/target git workspace for filesystem-backed move benchmarks.
pub struct MoveBenchmarkWorkspace {
    _dir: TempDir,
    repo_root: PathBuf,
    source_root: PathBuf,
    target_root: PathBuf,
}

impl MoveBenchmarkWorkspace {
    /// Create and initialize one reusable source/target fixture workspace.
    pub fn new() -> Self {
        let dir = TempDir::new().expect("benchmark tempdir");
        let repo_root = dir.path().to_path_buf();
        let source_root = repo_root.join("source-workspace");
        let target_root = repo_root.join("target-workspace");
        std::fs::create_dir_all(&source_root).expect("create benchmark source workspace");
        std::fs::create_dir_all(&target_root).expect("create benchmark target workspace");
        let status = Command::new("git")
            .current_dir(&repo_root)
            .args(["init", "--quiet"])
            .status()
            .expect("run git init");
        assert!(status.success(), "git init failed");
        Self {
            _dir: dir,
            repo_root,
            source_root,
            target_root,
        }
    }

    /// Root of the temporary git repository.
    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    /// Source workspace used by the move domain.
    pub fn source_root(&self) -> &Path {
        &self.source_root
    }

    /// Destination workspace used by the move domain.
    pub fn target_root(&self) -> &Path {
        &self.target_root
    }

    /// Remove fixture contents while preserving the initialized git repository.
    pub fn reset(&self) {
        for root in [&self.source_root, &self.target_root] {
            if root.exists() {
                fs::remove_dir_all(root).expect("reset benchmark workspace");
            }
            fs::create_dir_all(root).expect("recreate benchmark workspace");
        }
    }
}

// ---------------------------------------------------------------------------
// Paths produced by domain setup
// ---------------------------------------------------------------------------

/// Directory layout produced by a [`SandboxSetup`] implementation.
pub struct SandboxPaths {
    /// Directory that the CLI receives as `--index-root`.
    pub index_root: PathBuf,
    /// Root of the workspace (the parent project directory).
    ///
    /// For "flat" sandboxes this is equal to `index_root`.  For workspace
    /// sandboxes (e.g. when the index lives under `.ticket/`) this is the
    /// parent of that subdirectory.
    pub workspace_root: PathBuf,
}

// ---------------------------------------------------------------------------
// IoC trait — implemented once per domain
// ---------------------------------------------------------------------------

/// Domain-specific sandbox initialisation contract.
///
/// Implement this trait once per domain (e.g. `TicketFlatSetup`) and use it
/// as the type parameter for [`Sandbox<S>`].  The generic struct handles
/// temp-directory lifecycle; the implementation handles domain path layout and
/// any required store initialisation commands.
pub trait SandboxSetup {
    /// Called immediately after the [`TempDir`] is created.
    ///
    /// Implementations should:
    /// 1. Create any required subdirectories (e.g. `.ticket/`).
    /// 2. Run domain initialisation commands (e.g. `ticket init`).
    /// 3. Return the resulting [`SandboxPaths`].
    fn setup(workspace_root: &Path) -> SandboxPaths;
}

// ---------------------------------------------------------------------------
// Generic sandbox struct
// ---------------------------------------------------------------------------

/// An isolated, self-cleaning test sandbox backed by a [`TempDir`].
///
/// `S` is a [`SandboxSetup`] implementation that controls the directory layout
/// and store initialisation for the domain under test.  The `TempDir` is kept
/// alive for the sandbox's lifetime and deleted when it is dropped.
///
/// ```ignore
/// let s = Sandbox::<MyDomainSetup>::new();
/// // use s.index_root() / s.workspace_root() in commands
/// ```
pub struct Sandbox<S: SandboxSetup> {
    _dir: TempDir,
    paths: SandboxPaths,
    _marker: std::marker::PhantomData<fn() -> S>,
}

impl<S: SandboxSetup> Sandbox<S> {
    /// Create a new sandbox by calling [`SandboxSetup::setup`] on a fresh
    /// [`TempDir`].
    pub fn new() -> Self {
        let dir = TempDir::new().expect("failed to create sandbox temp dir");
        let paths = S::setup(dir.path());
        Self {
            _dir: dir,
            paths,
            _marker: std::marker::PhantomData,
        }
    }

    /// Path supplied as `--index-root` to CLI commands.
    pub fn index_root(&self) -> &Path {
        &self.paths.index_root
    }

    /// Root directory of the workspace (parent of the index root for
    /// workspace-layout sandboxes; equal to `index_root` for flat sandboxes).
    pub fn workspace_root(&self) -> &Path {
        &self.paths.workspace_root
    }
}
