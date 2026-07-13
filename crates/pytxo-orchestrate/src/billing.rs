use std::sync::Arc;

use pytxo_core::{
    BillingMode, BillingReconciler, ByteHeuristicEstimator, ConfigModelRouter, DomainId,
    HttpBillingReconciler, LocalHybridBilling, ManagedTransport, ModelRouter,
    NoopBillingReconciler, PytxoConfig, ReservationId, RunId, StaticPriceTable, TokenEstimator,
    TokenWallet, UsageMeter,
};
use pytxo_store::SharedStore;

#[cfg(feature = "billing-tiktoken")]
use pytxo_core::TiktokenEstimator;

/// Ultra run reconciler: noop locally, or HTTP stub when Link reconcile is enabled.
#[derive(Clone)]
pub enum RunBillingReconciler {
    Noop(NoopBillingReconciler),
    Http(HttpBillingReconciler),
}

impl BillingReconciler for RunBillingReconciler {
    fn reconcile_run_start(&self, domain_id: &DomainId, run_id: &RunId) -> pytxo_core::Result<()> {
        match self {
            Self::Noop(n) => n.reconcile_run_start(domain_id, run_id),
            Self::Http(h) => h.reconcile_run_start(domain_id, run_id),
        }
    }

    fn reconcile_run_end(
        &self,
        domain_id: &DomainId,
        run_id: &RunId,
        totals: &pytxo_core::RunUsageTotals,
    ) -> pytxo_core::Result<()> {
        match self {
            Self::Noop(n) => n.reconcile_run_end(domain_id, run_id, totals),
            Self::Http(h) => h.reconcile_run_end(domain_id, run_id, totals),
        }
    }
}

fn reconciler_for_cfg(cfg: &PytxoConfig) -> anyhow::Result<Arc<RunBillingReconciler>> {
    if !cfg.billing.link_reconcile_enabled() {
        return Ok(Arc::new(RunBillingReconciler::Noop(NoopBillingReconciler)));
    }
    let url = cfg.billing.proxy_url.trim();
    if url.is_empty() {
        anyhow::bail!(
            "billing.link_reconcile is true but billing.proxy_url is empty; set proxy_url to the Pytxo Link base URL"
        );
    }
    let http = HttpBillingReconciler::new(url);
    http.ping()
        .map_err(|e| anyhow::anyhow!("billing.link_reconcile: {e}"))?;
    Ok(Arc::new(RunBillingReconciler::Http(http)))
}

pub struct UltraRunBilling {
    pub hybrid: LocalHybridBilling<SharedStore, RunBillingReconciler>,
    pub meter: Arc<SharedStore>,
    pub reservation: Option<ReservationId>,
    pub domain_id: DomainId,
    /// Guards against double settlement when orchestrate retries run teardown.
    pub settled: bool,
}

pub fn setup_ultra_billing(
    store: Arc<SharedStore>,
    cfg: &PytxoConfig,
    domain_id: DomainId,
) -> anyhow::Result<Option<UltraRunBilling>> {
    if !cfg.billing_mode().is_ultra() {
        return Ok(None);
    }

    let reconciler = reconciler_for_cfg(cfg)?;
    let hybrid = LocalHybridBilling::new(Arc::clone(&store), reconciler, BillingMode::Ultra);

    store.ensure_account(&domain_id, cfg.billing.initial_balance_microcredits)?;

    Ok(Some(UltraRunBilling {
        hybrid,
        meter: store,
        reservation: None,
        domain_id,
        settled: false,
    }))
}

pub struct RunMetering {
    pub billing_mode: BillingMode,
    pub domain_id: DomainId,
    pub model_router: Arc<dyn ModelRouter>,
    pub managed_transport: ManagedTransport,
    pub usage_meter: Option<Arc<dyn UsageMeter>>,
    pub token_estimator: Arc<dyn TokenEstimator>,
}

pub fn metering_for_ctx(
    cfg: &PytxoConfig,
    repo_root: &std::path::Path,
    ultra: &Option<UltraRunBilling>,
) -> RunMetering {
    let domain_id =
        DomainId::from_repo_root(repo_root).unwrap_or_else(|_| DomainId("unknown".to_string()));
    let model_router: Arc<dyn ModelRouter> = Arc::new(ConfigModelRouter);
    let managed_transport = ManagedTransport {
        mode: cfg.billing_mode(),
        proxy_base_url: cfg.billing.inference_proxy_url.clone(),
        ..Default::default()
    };
    let token_estimator: Arc<dyn TokenEstimator> = {
        #[cfg(feature = "billing-tiktoken")]
        {
            if cfg.billing_mode() == BillingMode::Ultra {
                Arc::new(TiktokenEstimator)
            } else {
                Arc::new(ByteHeuristicEstimator)
            }
        }
        #[cfg(not(feature = "billing-tiktoken"))]
        {
            Arc::new(ByteHeuristicEstimator)
        }
    };
    let usage_meter: Option<Arc<dyn UsageMeter>> = ultra
        .as_ref()
        .map(|u| Arc::clone(&u.meter) as Arc<dyn UsageMeter>);
    RunMetering {
        billing_mode: cfg.billing_mode(),
        domain_id,
        model_router,
        managed_transport,
        usage_meter,
        token_estimator,
    }
}

pub fn estimate_reserve_microcredits(cfg: &PytxoConfig, plan_tasks: usize) -> i64 {
    let base = cfg.billing.reserve_microcredits;
    let per_task = 50_000_i64;
    base.max(per_task.saturating_mul(plan_tasks as i64))
}

pub fn settle_ultra_run(
    ultra: &mut UltraRunBilling,
    run_id: &RunId,
    cost_micro: i64,
) -> anyhow::Result<()> {
    if ultra.settled {
        return Ok(());
    }
    if let Some(res) = ultra.reservation.take() {
        ultra.hybrid.wallet.commit_debit(&res, cost_micro)?;
    }
    let totals = ultra.meter.run_totals(run_id)?;
    // Link `/v1/runs/end` is idempotent (ADR-0021); safe to retry on transient HTTP errors.
    ultra.hybrid.on_run_end(&ultra.domain_id, run_id, &totals)?;
    ultra.settled = true;
    Ok(())
}

#[allow(dead_code)]
pub fn price_table_cost_micro(
    cfg: &PytxoConfig,
    router: &dyn ModelRouter,
    agent: &str,
    tokens_in: u64,
    tokens_out: u64,
) -> i64 {
    let route = router.route(agent, cfg);
    let table = StaticPriceTable;
    table.cost_micro_usd(
        &route.model,
        &pytxo_core::TokenCounts {
            tokens_in,
            tokens_out,
        },
    )
}
