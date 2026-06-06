use std::sync::Arc;

use crate::billing::{BillingMode, RunUsageTotals};
use crate::{DomainId, ReservationId, Result, RunId};

pub trait TokenWallet: Send + Sync {
    fn balance_microcredits(&self, domain_id: &DomainId) -> Result<i64>;

    fn reserve(&self, domain_id: &DomainId, amount: i64, run_id: &RunId) -> Result<ReservationId>;

    fn commit_debit(&self, reservation: &ReservationId, actual: i64) -> Result<()>;

    fn release(&self, reservation: &ReservationId) -> Result<()>;

    fn ensure_account(&self, domain_id: &DomainId, initial_balance: i64) -> Result<()>;
}

pub trait BillingReconciler: Send + Sync {
    fn reconcile_run_start(&self, domain_id: &DomainId, run_id: &RunId) -> Result<()>;

    fn reconcile_run_end(
        &self,
        domain_id: &DomainId,
        run_id: &RunId,
        totals: &RunUsageTotals,
    ) -> Result<()>;
}

/// Stub until Pytxo Link HTTP client exists.
#[derive(Clone, Debug, Default)]
pub struct NoopBillingReconciler;

impl BillingReconciler for NoopBillingReconciler {
    fn reconcile_run_start(&self, _domain_id: &DomainId, _run_id: &RunId) -> Result<()> {
        Ok(())
    }

    fn reconcile_run_end(
        &self,
        _domain_id: &DomainId,
        _run_id: &RunId,
        _totals: &RunUsageTotals,
    ) -> Result<()> {
        Ok(())
    }
}

/// Hybrid: local wallet authority + cloud reconcile hooks on run boundaries.
pub struct LocalHybridBilling<W, R> {
    pub wallet: Arc<W>,
    pub reconciler: Arc<R>,
    pub mode: BillingMode,
}

impl<W: TokenWallet, R: BillingReconciler> LocalHybridBilling<W, R> {
    pub fn new(wallet: Arc<W>, reconciler: Arc<R>, mode: BillingMode) -> Self {
        Self {
            wallet,
            reconciler,
            mode,
        }
    }

    pub fn on_run_start(&self, domain_id: &DomainId, run_id: &RunId) -> Result<()> {
        if !self.mode.is_ultra() {
            return Ok(());
        }
        self.reconciler.reconcile_run_start(domain_id, run_id)
    }

    pub fn on_run_end(
        &self,
        domain_id: &DomainId,
        run_id: &RunId,
        totals: &RunUsageTotals,
    ) -> Result<()> {
        if !self.mode.is_ultra() {
            return Ok(());
        }
        self.reconciler.reconcile_run_end(domain_id, run_id, totals)
    }
}
