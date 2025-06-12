-- Remove the full_path functionality
DROP TRIGGER IF EXISTS trigger_update_account_full_path ON accounts;
DROP FUNCTION IF EXISTS update_account_full_path();
DROP FUNCTION IF EXISTS build_account_path(UUID);
DROP INDEX IF EXISTS idx_accounts_full_path;
ALTER TABLE accounts DROP COLUMN IF EXISTS full_path;