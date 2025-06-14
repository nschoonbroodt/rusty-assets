DROP TRIGGER IF EXISTS trigger_update_account_full_path ON accounts;
DROP FUNCTION IF EXISTS update_account_full_path();
DROP FUNCTION IF EXISTS build_account_path(account_id UUID);
DROP INDEX IF EXISTS idx_accounts_full_path;
DROP INDEX IF EXISTS idx_accounts_unique_root_name;
DROP INDEX IF EXISTS idx_accounts_parent;
DROP INDEX IF EXISTS idx_accounts_type;


DROP TABLE IF EXISTS accounts;