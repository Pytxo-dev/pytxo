//! Shared domain types and configuration for Pytxo.

mod billing;
mod child_env;
mod cloud;
mod config;
mod error;
mod execution;
mod ids;
mod moat;
mod path_util;
mod plan;
mod project;
mod task;

pub use billing::{
    ArbitrageSample, ArbitrageYield, BillingConfig, BillingMode, BillingReconciler,
    ByteHeuristicEstimator, CliAdapter, ConfigModelRouter, HttpBillingReconciler,
    LocalHybridBilling, ManagedTransport, ModelId, ModelRoute, ModelRouter, NoopBillingReconciler,
    ProviderId, ReservationId, RunUsageTotals, StaticPriceTable, TokenCounts, TokenEstimator,
    TokenWallet, UsageKey, UsageMeter, UsageSource,
};
pub use child_env::ChildLaunchEnv;
pub use cloud::{
    content_hash, CacheLookup, CachePut, CachedScaffold, CloudConfig, CloudDispatcher,
    ContextCache, ExecRequest, ExecResponse, HttpCloudDispatcher, HttpContextCache,
    McpHubConfig, NoopCloudDispatcher, NoopContextCache, StartSandboxRequest,
    StartSandboxResponse, SyncFile,
};
pub use config::PytxoConfig;
pub use error::{PytxoError, Result};
pub use execution::ExecutionBackend;
pub use ids::{AgentId, RunId, TaskId};
pub use moat::{
    conflict_error, normalize_claim_path, paths_claim_overlap, root_scoped_claim, DomainId,
    FidelityTier, IsolationBackend, IsolationCtx, IsolationMode, LiveAgent, PathConflict,
    PermissionEngine, PermissionProfile, RaceShield, ScaffoldResult, ScaffoldStats, SignalCore,
    StdinBuffer, WorkspaceHandle,
};
pub use path_util::{canonical_repo_root, strip_extended_path};
pub use plan::{AgentAssignment, ConflictPair, ExecutionPlan, ScheduledTask};
pub use project::{ProjectManifest, ProjectMeta, ProjectRoot};
pub use task::{AgentSpec, Task};

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let id = super::RunId::new();
        assert!(!id.0.is_empty());
    }
}
