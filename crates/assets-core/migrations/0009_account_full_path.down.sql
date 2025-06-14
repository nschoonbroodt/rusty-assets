-- Remove the full_path functionality
-- First drop views that depend on full_path column
DROP VIEW IF EXISTS v_account_balances;
DROP VIEW IF EXISTS v_balance_sheet_accounts;
DROP VIEW IF EXISTS v_income_statement_accounts;
DROP VIEW IF EXISTS v_all_transactions_with_duplicate_status;

-- Then remove the full_path infrastructure
DROP TRIGGER IF EXISTS trigger_update_account_full_path ON accounts;
DROP FUNCTION IF EXISTS update_account_full_path();
DROP FUNCTION IF EXISTS build_account_path(UUID);
DROP INDEX IF EXISTS idx_accounts_full_path;
ALTER TABLE accounts DROP COLUMN IF EXISTS full_path;

-- Recreate the original views without full_path
CREATE VIEW v_account_balances AS
WITH account_movements AS (
    SELECT 
        a.id as account_id,
        a.name as account_name,
        a.account_type,
        COALESCE(SUM(
            CASE 
                WHEN a.account_type IN ('asset', 'expense') THEN je.amount
                ELSE -je.amount
            END
        ), 0) as balance
    FROM accounts a
    LEFT JOIN journal_entries je ON a.id = je.account_id
    GROUP BY a.id, a.name, a.account_type
)
SELECT 
    account_id,
    account_name,
    account_type,
    balance,
    ABS(balance) as abs_balance
FROM account_movements
ORDER BY account_name;