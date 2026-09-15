pub mod cross_store_edges;
pub mod discovery;
pub mod error;
pub mod generated_markdown;
pub mod index_generator;
pub mod interoperability;
pub mod model;
pub mod operation_journal;
pub mod query;
pub mod runtime;
pub mod storage;
pub mod workspace;
pub mod workspace_policy;

#[cfg(feature = "testing")]
pub mod testing;

// Re-export board types at the crate root for convenient access.
pub use storage::{
    BoardCleanPreview, BoardCleanResult, BoardConfig, BoardEntry, BoardEntryStatus, BoardError,
    BoardReconcileResult, BoardSnapshot, EntityStore, ReconcileAction,
};

// Re-export index entry schema types at the crate root for convenient access.
pub use discovery::{
    discover_stores, reconcile_stores, summarize, DiscoveredStore, IntegrationStatus,
    ReconcileSummary, StoreReport, STORE_MARKERS,
};
pub use interoperability::InteroperableArtifact;
pub use model::{
    domain::{
        DomainId, DomainSchemaVersion, EntityOwnershipRegistry, EntityTypeId,
        EntityTypeSchemaVersion, IdentifierError, OwnershipError,
    },
    event::{EventEnvelope, MutationOperation},
    hook::{HookChain, HookFailure, HookOutcome, MutationHook, MutationOutcome, VetoCause},
    index_entry::{ContentKind, IndexEntry, IndexRef, IndexRelations, RelationKind},
    index_sidecar::{
        read_sidecar, write_sidecar, IndexSidecar, SidecarError, SidecarValidationIssue,
    },
    migration::{
        EntityTypeMigrationStep, ExternalUrnConstraintViolation, ExternalUrnVersionConstraint,
        InactiveEntityTypeReportEntry, MigrationDryRunReport, MigrationJournal,
        MigrationJournalError, MigrationPhase, MigrationPhaseError, MigrationPhaseRecord,
    },
    urn::{Urn, UrnError, UrnResolver, URN_SCHEME},
    workspace_capability::{
        DomainCapabilityState, DomainDependency, UnavailableCapability, WorkspaceCapabilities,
    },
};
pub use operation_journal::{
    OperationJournal, OperationJournalLinks, OperationJournalPhase, OperationJournalStep,
    OperationPreflight, OperationRecovery, OperationReversibility,
    OPERATION_JOURNAL_SCHEMA_VERSION,
};
