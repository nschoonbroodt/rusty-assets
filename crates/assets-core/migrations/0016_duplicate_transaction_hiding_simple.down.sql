-- Rollback simple duplicate tracking
DROP INDEX IF EXISTS idx_transactions_merged_into;
DROP INDEX IF EXISTS idx_transactions_is_duplicate;
ALTER TABLE transactions DROP COLUMN IF EXISTS merged_into_transaction_id;
ALTER TABLE transactions DROP COLUMN IF EXISTS is_duplicate;
