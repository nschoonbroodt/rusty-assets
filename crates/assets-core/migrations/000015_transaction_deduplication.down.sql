-- Rollback transaction deduplication support

DROP VIEW IF EXISTS v_transactions_with_duplicates;
DROP FUNCTION IF EXISTS fn_find_potential_duplicates(UUID, DECIMAL, INTEGER);
DROP TABLE IF EXISTS transaction_matches;

-- Remove columns from transactions table
ALTER TABLE transactions DROP COLUMN IF EXISTS external_reference;
ALTER TABLE transactions DROP COLUMN IF EXISTS import_batch_id;
ALTER TABLE transactions DROP COLUMN IF EXISTS import_source;
