//! Shared domain types and configuration for Pytxo.

mod ade_registry;
mod background_process;
mod billing;
mod child_env;
mod cloud;
mod config;
mod coordinator;
mod error;
mod execution;
mod fleet;
mod ids;
mod moat;
mod path_util;
mod plan;
mod project;
mod review;
pub mod routing;
mod service_health;
mod task;
mod trust;
mod upgrade_guard;
pub use upgrade_guard::UpgradeGuard;

pub use ade_registry::{
    ade_can_dispatch, ade_on_path, all_ade_clis, detect_on_path, format_agents_list, resolve_ade,
    AdeAuthPolicy, AdeCliSpec, AdeDispatchPolicy,
};
pub use background_process::background_command;
pub use billing::{
    all_byok_key_envs, all_providers, default_token_estimator, find_custom_provider, get_provider,
    inject_byok_env, key_configured, key_env_configured, list_provider_status, list_static_models,
    load_custom_providers, provider_from_hint, providers_json_path, resolve_openai_base_url,
    ArbitrageSample, ArbitrageYield, BillingConfig, BillingMode, BillingReconciler,
    ByteHeuristicEstimator, CliAdapter, ConfigModelRouter, CustomProviderSpec,
    HttpBillingReconciler, LocalHybridBilling, ManagedTransport, ModelId, ModelRoute, ModelRouter,
    NoopBillingReconciler, ProviderId, ProviderSpec, ProviderStatus, ReservationId, RunUsageTotals,
    StaticPriceTable, TiktokenEstimator, TokenCounts, TokenEstimator, TokenWallet, UsageKey,
    UsageMeter, UsageSource,
};
pub use child_env::ChildLaunchEnv;
pub use cloud::{
    cloud_path_denied, cloud_sync_manifest, collect_sync_paths, content_hash,
    delta_from_overlay_upper, is_overlay_upper, overlay_upper_cloud_delta, validate_cloud_content,
    validate_cloud_upload, CacheLookup, CachePut, CachedScaffold, CloudConfig, CloudDispatcher,
    CloudSyncManifest, CloudSyncManifestEntry, ContextCache, ExecRequest, ExecResponse,
    HttpCloudDispatcher, HttpContextCache, McpHubConfig, NoopCloudDispatcher, NoopContextCache,
    OverlayDelta, StartSandboxRequest, StartSandboxResponse, SyncFile,
};
pub use config::{BlastConfig, PytxoConfig};
pub use coordinator::{
    validate_route_proposal, CoordinatorConfig, CoordinatorIntent, CoordinatorProfile,
    CoordinatorTransport, RouteConstraintViolation, RouteModel, RouteProposal,
    RoutingCatalogSnapshot, DEFAULT_COORDINATOR_MODEL, DEFAULT_COORDINATOR_PROVIDER,
};
pub use error::{PytxoError, Result};
pub use execution::ExecutionBackend;
pub use fleet::{FleetManifest, FleetMeta, FleetNode, FleetPlan, FleetWave};
pub use ids::{AgentId, RunId, TaskId};
pub use moat::{
    conflict_error, normalize_claim_path, paths_claim_overlap, root_scoped_claim, DomainId,
    FidelityTier, IsolationBackend, IsolationCtx, IsolationMode, LiveAgent, NetworkPolicy,
    NetworkPolicyEngine, PathConflict, PermissionEngine, PermissionProfile, RaceShield,
    ScaffoldResult, ScaffoldStats, SignalCore, StdinBuffer, WorkspaceHandle,
};
pub use path_util::{canonical_repo_root, strip_extended_path};
pub use plan::{AgentAssignment, ConflictPair, ExecutionPlan, ScheduledTask};
pub use project::{ProjectManifest, ProjectMeta, ProjectRoot};
pub use review::{
    prepared_manifest_digest, require_candidate_verification_contract, CandidateCheckEvidence,
    CandidateInventoryFile, CandidateVerificationEvidence, PreparedBlobChunk, PreparedRunFile,
    PreparedRunFileKind, PreparedRunManifest, PreparedRunSummary, RunApplyError,
};
pub use service_health::response_ok as service_health_ok;
pub use service_health::{health_lists_provider, providers_configured};
pub use task::{AgentSpec, Task};
pub use trust::{default_trust_path, TrustedDomain, TrustedDomainStore};

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let id = super::RunId::new();
        assert!(!id.0.is_empty());
    }
}
