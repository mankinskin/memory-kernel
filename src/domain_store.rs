//! Domain-neutral workspace store resolution and typed operation capabilities.
//!
//! A domain owns its entity model and error types. This module owns only the
//! common workspace/store boundary, so domain APIs can opt into exactly the
//! operations they support without sharing an entity schema.

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{
    model::domain::DomainId,
    workspace::{
        StoreRootDiagnostic, canonical_store_root, resolve_explicit_store_root_from,
        resolve_workspace_root_from_store_root,
    },
};

/// Whether resolving a domain store may initialize its canonical directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreAccessMode {
    /// Resolve for reading or querying without creating a missing store.
    ReadOnly,
    /// Resolve the canonical store and create it when it is missing.
    CreateOrOpen,
}

/// The selected workspace and resolved store for one domain operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainStoreResolution {
    pub domain: DomainId,
    pub local_workspace: PathBuf,
    pub store_root: PathBuf,
    pub access_mode: StoreAccessMode,
    pub diagnostics: Vec<StoreRootDiagnostic>,
}

/// Failures produced while resolving or initializing a domain store.
#[derive(Debug, Error)]
pub enum DomainStoreError {
    #[error("failed to initialize canonical domain store '{path}': {source}")]
    Initialize {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Shared workspace/store contract for a domain API.
///
/// Implementers provide their stable domain identity and legacy directory
/// marker (for example, `ticket` and `.ticket`). The default resolver keeps
/// the selected workspace fixed, preserves resolver diagnostics, and ensures
/// that only [`StoreAccessMode::CreateOrOpen`] initializes a canonical store.
pub trait DomainStore {
    /// Stable, domain-owned identity of this store.
    fn domain_id() -> DomainId;

    /// The domain's legacy directory marker used by the workspace resolver.
    fn store_dir_name() -> &'static str;

    /// Resolve this domain's store within an explicitly selected workspace.
    fn resolve_store(
        local_workspace: &Path,
        access_mode: StoreAccessMode,
    ) -> Result<DomainStoreResolution, DomainStoreError> {
        let resolved = resolve_explicit_store_root_from(local_workspace, Self::store_dir_name());
        let selected_workspace =
            resolve_workspace_root_from_store_root(&resolved.store_root, Self::store_dir_name());

        let store_root = match access_mode {
            StoreAccessMode::ReadOnly => resolved.store_root,
            StoreAccessMode::CreateOrOpen => {
                let canonical = canonical_store_root(&selected_workspace, Self::store_dir_name());
                std::fs::create_dir_all(&canonical).map_err(|source| {
                    DomainStoreError::Initialize {
                        path: canonical.clone(),
                        source,
                    }
                })?;
                canonical
            }
        };

        Ok(DomainStoreResolution {
            domain: Self::domain_id(),
            local_workspace: selected_workspace,
            store_root,
            access_mode,
            diagnostics: resolved.diagnostics,
        })
    }
}

/// Capability for domains that create a typed entity.
pub trait CreateEntity {
    type Entity;
    type CreateInput;
    type CreateResult;
    type Error;

    fn create_entity(&self, input: Self::CreateInput) -> Result<Self::CreateResult, Self::Error>;
}

/// Capability for domains that read a typed entity.
pub trait ReadEntity {
    type EntityId;
    type Entity;
    type ReadResult;
    type Error;

    fn read_entity(&self, id: Self::EntityId) -> Result<Self::ReadResult, Self::Error>;
}

/// Capability for domains that update a typed entity.
pub trait UpdateEntity {
    type EntityId;
    type Entity;
    type Patch;
    type UpdateResult;
    type Error;

    fn update_entity(
        &self,
        id: Self::EntityId,
        patch: Self::Patch,
    ) -> Result<Self::UpdateResult, Self::Error>;
}

/// Capability for domains that delete a typed entity.
pub trait DeleteEntity {
    type EntityId;
    type DeleteResult;
    type Error;

    fn delete_entity(&self, id: Self::EntityId) -> Result<Self::DeleteResult, Self::Error>;
}

/// Optional query capability for domains that list typed entities.
pub trait ListEntities {
    type Query;
    type Entity;
    type ListResult;
    type Error;

    fn list_entities(&self, query: Self::Query) -> Result<Self::ListResult, Self::Error>;
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::workspace::StoreRootDiagnostic;

    struct TicketStore;
    struct SpecStore;
    struct AuditStore;
    struct CreateOnlyStore;

    impl DomainStore for TicketStore {
        fn domain_id() -> DomainId {
            DomainId::new("ticket").unwrap()
        }

        fn store_dir_name() -> &'static str {
            ".ticket"
        }
    }

    impl DomainStore for SpecStore {
        fn domain_id() -> DomainId {
            DomainId::new("spec").unwrap()
        }

        fn store_dir_name() -> &'static str {
            ".spec"
        }
    }

    impl DomainStore for AuditStore {
        fn domain_id() -> DomainId {
            DomainId::new("audit").unwrap()
        }

        fn store_dir_name() -> &'static str {
            ".audit"
        }
    }

    impl CreateEntity for CreateOnlyStore {
        type Entity = String;
        type CreateInput = String;
        type CreateResult = String;
        type Error = std::convert::Infallible;

        fn create_entity(
            &self,
            input: Self::CreateInput,
        ) -> Result<Self::CreateResult, Self::Error> {
            Ok(input)
        }
    }

    #[test]
    fn exposes_active_domain_identity() {
        assert_eq!(TicketStore::domain_id().as_str(), "ticket");
        assert_eq!(SpecStore::domain_id().as_str(), "spec");
    }

    #[test]
    fn capabilities_are_opt_in_and_retain_domain_types() {
        let created = CreateOnlyStore
            .create_entity("domain-owned".to_string())
            .unwrap();

        assert_eq!(created, "domain-owned");
    }

    #[test]
    fn read_only_does_not_initialize_a_missing_store() {
        let workspace = tempdir().unwrap();

        let resolution =
            TicketStore::resolve_store(workspace.path(), StoreAccessMode::ReadOnly).unwrap();

        assert_eq!(resolution.local_workspace, workspace.path());
        assert_eq!(
            resolution.store_root,
            workspace.path().join(".workflow-tools").join("ticket")
        );
        assert_eq!(resolution.access_mode, StoreAccessMode::ReadOnly);
        assert!(!resolution.store_root.exists());
    }

    #[test]
    fn create_or_open_initializes_only_the_canonical_store() {
        let workspace = tempdir().unwrap();
        let legacy = workspace.path().join(".audit");
        std::fs::create_dir_all(&legacy).unwrap();

        let resolution =
            AuditStore::resolve_store(workspace.path(), StoreAccessMode::CreateOrOpen).unwrap();

        assert_eq!(
            resolution.store_root,
            workspace.path().join(".workflow-tools").join("audit")
        );
        assert!(resolution.store_root.is_dir());
        assert!(matches!(
            resolution.diagnostics.as_slice(),
            [StoreRootDiagnostic::LegacyStore { .. }]
        ));
    }

    #[test]
    fn preserves_both_layouts_diagnostic_for_read_only_resolution() {
        let workspace = tempdir().unwrap();
        std::fs::create_dir_all(workspace.path().join(".spec")).unwrap();
        std::fs::create_dir_all(workspace.path().join(".workflow-tools").join("spec")).unwrap();

        let resolution =
            SpecStore::resolve_store(workspace.path(), StoreAccessMode::ReadOnly).unwrap();

        assert!(matches!(
            resolution.diagnostics.as_slice(),
            [StoreRootDiagnostic::BothLayoutsPresent { .. }]
        ));
    }
}
