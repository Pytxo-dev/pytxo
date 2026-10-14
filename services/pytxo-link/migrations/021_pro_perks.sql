-- Pytxo Pro perks: private shared receipts and private Pybench results.
-- A private receipt page is readable only by its owner; private Pybench
-- results never appear on the public board. Defaults keep existing rows public.
ALTER TABLE shared_receipts ADD COLUMN private BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE bench_results ADD COLUMN private BOOLEAN NOT NULL DEFAULT false;
CREATE INDEX bench_results_public_created ON bench_results(created_at DESC) WHERE NOT private AND deleted_at IS NULL;
