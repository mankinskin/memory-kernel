//! Shared "open, or initialize on first use" pattern for index-rooted
//! stores. Every domain (ticket, spec, rule, ...) defines its own store
//! type and error, so this only centralizes the retry-on-missing-workspace
//! branching and which-branch-was-taken bookkeeping; each domain still
//! supplies its own `open`/`init` closures and any extra post-init work
//! (initial scan, tracing spans, ...).

/// Implemented by a domain's error type so [`open_or_init`] can tell "no
/// store exists yet at this path" apart from any other failure.
pub trait NotFoundError {
    fn is_workspace_not_found(&self) -> bool;
}

/// Which branch [`open_or_init`] took, so callers that need to know (e.g.
/// to run a post-init scan, or record it in a tracing span) can match on it
/// without re-deriving it themselves.
pub enum Opened<S> {
    Existing(S),
    Initialized(S),
}

impl<S> Opened<S> {
    pub fn into_inner(self) -> S {
        match self {
            Opened::Existing(store) | Opened::Initialized(store) => store,
        }
    }

    pub fn was_initialized(&self) -> bool {
        matches!(self, Opened::Initialized(_))
    }
}

/// Try `open`; if it fails because the workspace hasn't been created yet,
/// fall back to `init`. Any other error from `open`, or any error from
/// `init`, propagates unchanged.
pub fn open_or_init<S, E>(
    open: impl FnOnce() -> Result<S, E>,
    init: impl FnOnce() -> Result<S, E>,
) -> Result<Opened<S>, E>
where
    E: NotFoundError,
{
    match open() {
        Ok(store) => Ok(Opened::Existing(store)),
        Err(error) if error.is_workspace_not_found() =>
            init().map(Opened::Initialized),
        Err(error) => Err(error),
    }
}

