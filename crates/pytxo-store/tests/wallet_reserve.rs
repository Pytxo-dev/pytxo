use pytxo_core::{DomainId, ReservationId, RunId, TokenWallet};
use pytxo_store::SharedStore;

#[test]
fn reserve_commit_and_release() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("pytxo.db");
    let store = SharedStore::open(&db).unwrap();
    let domain = DomainId("domain-test".to_string());
    store.ensure_account(&domain, 1_000_000).unwrap();

    let run_id = RunId::new();
    let res = store.reserve(&domain, 100_000, &run_id).unwrap();
    assert!(store.balance_microcredits(&domain).unwrap() < 1_000_000);

    store.commit_debit(&res, 50_000).unwrap();
    assert_eq!(store.balance_microcredits(&domain).unwrap(), 950_000);

    let run2 = RunId::new();
    let res2 = store.reserve(&domain, 200_000, &run2).unwrap();
    store.release(&res2).unwrap();
    assert_eq!(store.balance_microcredits(&domain).unwrap(), 950_000);

    let _ = ReservationId::new();
}

#[test]
fn reserve_fails_when_insufficient() {
    let dir = tempfile::tempdir().unwrap();
    let store = SharedStore::open(&dir.path().join("pytxo.db")).unwrap();
    let domain = DomainId("d".to_string());
    store.ensure_account(&domain, 100).unwrap();
    let err = store
        .reserve(&domain, 500, &RunId::new())
        .unwrap_err()
        .to_string();
    assert!(err.contains("insufficient"));
}
