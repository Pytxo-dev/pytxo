use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use chrono::Utc;
use pytxo_core::{
    ArbitrageSample, DomainId, ModelId, PytxoError, ReservationId, Result, RunId, RunUsageTotals,
    TokenCounts, TokenWallet, UsageKey, UsageMeter, UsageSource,
};
use rusqlite::params;

use crate::store::PytxoStore;

/// Thread-safe store handle for billing traits (`Mutex` newtype).
pub struct SharedStore(pub Mutex<PytxoStore>);

impl SharedStore {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self(Mutex::new(PytxoStore::open(path)?)))
    }

    pub fn lock(&self) -> Result<MutexGuard<'_, PytxoStore>> {
        self.0
            .lock()
            .map_err(|_| PytxoError::Store("store mutex poisoned".into()))
    }
}

impl TokenWallet for SharedStore {
    fn balance_microcredits(&self, domain_id: &DomainId) -> Result<i64> {
        let guard = self.lock()?;
        wallet_balance(&guard, domain_id)
    }

    fn ensure_account(&self, domain_id: &DomainId, initial_balance: i64) -> Result<()> {
        let guard = self.lock()?;
        wallet_ensure_account(&guard, domain_id, initial_balance)
    }

    fn reserve(&self, domain_id: &DomainId, amount: i64, run_id: &RunId) -> Result<ReservationId> {
        let guard = self.lock()?;
        wallet_reserve(&guard, domain_id, amount, run_id)
    }

    fn commit_debit(&self, reservation: &ReservationId, actual: i64) -> Result<()> {
        let guard = self.lock()?;
        wallet_commit_debit(&guard, reservation, actual)
    }

    fn release(&self, reservation: &ReservationId) -> Result<()> {
        let guard = self.lock()?;
        wallet_release(&guard, reservation)
    }
}

impl UsageMeter for SharedStore {
    fn record_context_arbitrage(&self, key: &UsageKey, samples: &[ArbitrageSample]) -> Result<()> {
        let guard = self.lock()?;
        meter_record_arbitrage(&guard, key, samples)
    }

    fn record_provider_usage(
        &self,
        key: &UsageKey,
        actual: &TokenCounts,
        model: &ModelId,
        cost_micro_usd: i64,
        source: UsageSource,
    ) -> Result<()> {
        let guard = self.lock()?;
        meter_record_provider(&guard, key, actual, model, cost_micro_usd, source)
    }

    fn run_totals(&self, run_id: &RunId) -> Result<RunUsageTotals> {
        let guard = self.lock()?;
        meter_run_totals(&guard, run_id)
    }
}

fn wallet_balance(store: &PytxoStore, domain_id: &DomainId) -> Result<i64> {
    let mut stmt = store
        .conn
        .prepare(
            "SELECT balance_microcredits - reserved_microcredits FROM wallet_accounts WHERE domain_id = ?1",
        )
        .map_err(store_err)?;
    let available: Option<i64> = stmt
        .query_row(params![domain_id.as_str()], |row| row.get(0))
        .ok();
    Ok(available.unwrap_or(0))
}

fn wallet_ensure_account(
    store: &PytxoStore,
    domain_id: &DomainId,
    initial_balance: i64,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    store
        .conn
        .execute(
            "INSERT INTO wallet_accounts (domain_id, balance_microcredits, reserved_microcredits, synced_at)
             VALUES (?1, ?2, 0, ?3)
             ON CONFLICT(domain_id) DO NOTHING",
            params![domain_id.as_str(), initial_balance, now],
        )
        .map_err(store_err)?;
    Ok(())
}

fn wallet_reserve(
    store: &PytxoStore,
    domain_id: &DomainId,
    amount: i64,
    run_id: &RunId,
) -> Result<ReservationId> {
    if amount <= 0 {
        return Err(PytxoError::Billing(
            "reserve amount must be positive".into(),
        ));
    }
    let reservation = ReservationId::new();
    let now = Utc::now().to_rfc3339();
    let tx = store.conn.unchecked_transaction().map_err(store_err)?;
    {
        let available: i64 = tx
            .query_row(
                "SELECT balance_microcredits - reserved_microcredits FROM wallet_accounts WHERE domain_id = ?1",
                params![domain_id.as_str()],
                |row| row.get(0),
            )
            .map_err(|_| {
                PytxoError::Billing(format!(
                    "wallet account missing for domain {}",
                    domain_id.as_str()
                ))
            })?;
        if available < amount {
            return Err(PytxoError::Billing(format!(
                "insufficient credits: need {amount}, have {available}"
            )));
        }
        tx.execute(
            "UPDATE wallet_accounts SET reserved_microcredits = reserved_microcredits + ?1 WHERE domain_id = ?2",
            params![amount, domain_id.as_str()],
        )
        .map_err(store_err)?;
        tx.execute(
            "INSERT INTO wallet_reservations (id, domain_id, run_id, amount_microcredits, status, created_at)
             VALUES (?1, ?2, ?3, ?4, 'held', ?5)",
            params![
                reservation.0,
                domain_id.as_str(),
                run_id.0,
                amount,
                now
            ],
        )
        .map_err(store_err)?;
    }
    tx.commit().map_err(store_err)?;
    Ok(reservation)
}

fn wallet_commit_debit(store: &PytxoStore, reservation: &ReservationId, actual: i64) -> Result<()> {
    let tx = store.conn.unchecked_transaction().map_err(store_err)?;
    let (domain_id, held): (String, i64) = tx
        .query_row(
            "SELECT domain_id, amount_microcredits FROM wallet_reservations WHERE id = ?1 AND status = 'held'",
            params![reservation.0],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| {
            PytxoError::Billing(format!(
                "reservation not held: {}",
                reservation.0
            ))
        })?;
    let debit = actual.max(0);
    tx.execute(
        "UPDATE wallet_accounts
         SET balance_microcredits = balance_microcredits - ?1,
             reserved_microcredits = reserved_microcredits - ?2
         WHERE domain_id = ?3",
        params![debit, held, domain_id],
    )
    .map_err(store_err)?;
    tx.execute(
        "UPDATE wallet_reservations SET status = 'committed' WHERE id = ?1",
        params![reservation.0],
    )
    .map_err(store_err)?;
    tx.commit().map_err(store_err)?;
    Ok(())
}

fn wallet_release(store: &PytxoStore, reservation: &ReservationId) -> Result<()> {
    let tx = store.conn.unchecked_transaction().map_err(store_err)?;
    let (domain_id, held): (String, i64) = tx
        .query_row(
            "SELECT domain_id, amount_microcredits FROM wallet_reservations WHERE id = ?1 AND status = 'held'",
            params![reservation.0],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| {
            PytxoError::Billing(format!(
                "reservation not held: {}",
                reservation.0
            ))
        })?;
    tx.execute(
        "UPDATE wallet_accounts SET reserved_microcredits = reserved_microcredits - ?1 WHERE domain_id = ?2",
        params![held, domain_id],
    )
    .map_err(store_err)?;
    tx.execute(
        "UPDATE wallet_reservations SET status = 'released' WHERE id = ?1",
        params![reservation.0],
    )
    .map_err(store_err)?;
    tx.commit().map_err(store_err)?;
    Ok(())
}

fn meter_record_arbitrage(
    store: &PytxoStore,
    key: &UsageKey,
    samples: &[ArbitrageSample],
) -> Result<()> {
    if samples.is_empty() {
        return Ok(());
    }
    let now = Utc::now().to_rfc3339();
    let tx = store.conn.unchecked_transaction().map_err(store_err)?;
    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO arbitrage_samples (
                    run_id, agent_id, domain_id, path, raw_bytes, scaffolded_bytes,
                    raw_tokens_in, sent_tokens_in, saved_tokens, reduction_pct, fallback_raw, ts
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            )
            .map_err(store_err)?;
        for s in samples {
            stmt.execute(params![
                key.run_id.0,
                key.agent_id.0,
                key.domain_id.as_str(),
                s.path,
                s.original_bytes as i64,
                s.scaffolded_bytes as i64,
                s.raw_tokens_in as i64,
                s.sent_tokens_in as i64,
                s.saved_tokens as i64,
                s.reduction_pct,
                s.fallback_raw as i32,
                now,
            ])
            .map_err(store_err)?;
        }
    }
    tx.commit().map_err(store_err)?;
    Ok(())
}

fn meter_record_provider(
    store: &PytxoStore,
    key: &UsageKey,
    actual: &TokenCounts,
    model: &ModelId,
    cost_micro_usd: i64,
    source: UsageSource,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    store
        .conn
        .execute(
            "INSERT INTO usage_records (
                run_id, agent_id, domain_id, model, tokens_in_billed, tokens_in_sent,
                tokens_out, cost_micro_usd, source, ts
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                key.run_id.0,
                key.agent_id.0,
                key.domain_id.as_str(),
                model.as_str(),
                actual.tokens_in as i64,
                actual.tokens_in as i64,
                actual.tokens_out as i64,
                cost_micro_usd,
                source.as_str(),
                now,
            ],
        )
        .map_err(store_err)?;
    Ok(())
}

fn meter_run_totals(store: &PytxoStore, run_id: &RunId) -> Result<RunUsageTotals> {
    let mut stmt = store
        .conn
        .prepare(
            "SELECT
                COALESCE(SUM(tokens_in_billed), 0),
                COALESCE(SUM(tokens_in_sent), 0),
                COALESCE(SUM(tokens_out), 0),
                COALESCE(SUM(cost_micro_usd), 0)
             FROM usage_records WHERE run_id = ?1",
        )
        .map_err(store_err)?;
    let (billed, sent, out, cost): (i64, i64, i64, i64) = stmt
        .query_row(params![run_id.0], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(store_err)?;

    let saved: i64 = store
        .conn
        .query_row(
            "SELECT COALESCE(SUM(saved_tokens), 0) FROM arbitrage_samples WHERE run_id = ?1",
            params![run_id.0],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(RunUsageTotals {
        tokens_in_billed: billed as u64,
        tokens_in_sent: sent as u64,
        tokens_out: out as u64,
        saved_tokens: saved as u64,
        cost_micro_usd: cost,
    })
}

impl PytxoStore {
    pub fn wallet_balance_microcredits(&self, domain_id: &DomainId) -> Result<i64> {
        wallet_balance(self, domain_id)
    }

    pub fn arbitrage_saved_tokens_for_run(&self, run_id: &str) -> Result<u64> {
        let saved: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(saved_tokens), 0) FROM arbitrage_samples WHERE run_id = ?1",
                params![run_id],
                |row| row.get(0),
            )
            .map_err(store_err)?;
        Ok(saved as u64)
    }
}

fn store_err(e: rusqlite::Error) -> PytxoError {
    PytxoError::Store(e.to_string())
}
