//! Engineering moat trait contracts ([[signal-core]], [[blast-shield]], [[race-shield]]).

pub mod blast;
pub mod race;
pub mod signal;

pub use blast::{IsolationBackend, IsolationCtx, IsolationMode, WorkspaceHandle};
pub use race::{
    conflict_error, normalize_claim_path, paths_claim_overlap, LiveAgent, PathConflict, RaceShield,
    StdinBuffer,
};
pub use signal::{FidelityTier, ScaffoldResult, ScaffoldStats, SignalCore};
