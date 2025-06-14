-- Drop initial schema
-- Drop any remaining views that might depend on the tables
DROP VIEW IF EXISTS v_account_balances;
DROP VIEW IF EXISTS v_balance_sheet_accounts; 
DROP VIEW IF EXISTS v_income_statement_accounts;
DROP VIEW IF EXISTS v_all_transactions_with_duplicate_status;

-- Drop the constraint trigger first
DROP TRIGGER IF EXISTS trigger_validate_transaction_balance ON journal_entries;
-- Drop the function
DROP FUNCTION IF EXISTS validate_transaction_balance();
-- Drop indexes first (if they are not automatically dropped with tables, though most are)
DROP INDEX IF EXISTS idx_journal_entries_amount;
DROP INDEX IF EXISTS idx_journal_entries_account;
DROP INDEX IF EXISTS idx_journal_entries_transaction;
DROP INDEX IF EXISTS idx_transactions_created_by;
DROP INDEX IF EXISTS idx_transactions_date;
DROP INDEX IF EXISTS idx_accounts_parent;
DROP INDEX IF EXISTS idx_accounts_type;
DROP INDEX IF EXISTS idx_accounts_unique_root_name;
-- Drop constraints on accounts table before dropping the table
ALTER TABLE IF EXISTS accounts DROP CONSTRAINT IF EXISTS uq_account_name_parent;
ALTER TABLE IF EXISTS accounts DROP CONSTRAINT IF EXISTS chk_account_name_no_colon;
-- Drop tables in reverse order of creation due to foreign key constraints
ALTER TABLE IF EXISTS transactions DROP CONSTRAINT IF EXISTS chk_transaction_balanced;
DROP TABLE IF EXISTS journal_entries;
DROP TABLE IF EXISTS transactions;
DROP TABLE IF EXISTS accounts;
-- Drop custom types
DROP TYPE IF EXISTS account_subtype;
DROP TYPE IF EXISTS account_type;
-- Drop extension
DROP EXTENSION IF EXISTS "uuid-ossp";