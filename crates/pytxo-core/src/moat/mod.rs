//! Engineering moat trait contracts ([[signal-core]], [[blast-shield]], [[race-shield]]).
//! Local capability ladder: [[permission-profile-engine]] (ADR-0008).

pub mod blast;
pub mod permission;
pub mod race;
pub mod signal;

pub use blast::{IsolationBackend, IsolationCtx, IsolationMode, WorkspaceHandle};
pub use permission::{
    DomainId, NetworkPolicy, NetworkPolicyEngine, PermissionEngine, PermissionProfile,
};
pub use race::{
    conflict_error, normalize_claim_path, paths_claim_overlap, root_scoped_claim, LiveAgent,
    PathConflict, RaceShield, StdinBuffer,
};
pub use signal::{FidelityTier, ScaffoldResult, ScaffoldStats, SignalCore};
