/// A cross-cutting contract for artifacts that need to interoperate within a
/// workflow workspace.
///
/// Shared kernel artifacts use this to expose their stable class and report
/// dynamic contract violations without depending on a domain crate.
pub trait InteroperableArtifact {
    /// The specific class or type of the artifact, such as "move-journal".
    fn artifact_class(&self) -> &'static str;

    /// Return dynamic interoperability gaps not guaranteed by the type's
    /// structure.
    fn interoperability_gaps(&self) -> Vec<&'static str> {
        Vec::new()
    }
}
