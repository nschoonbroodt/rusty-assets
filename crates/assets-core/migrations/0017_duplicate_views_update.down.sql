-- Rollback views update

-- Drop the admin view
DROP VIEW IF EXISTS v_all_transactions_with_duplicate_status;

-- Recreate original views (without duplicate filtering)
DROP VIEW IF EXISTS v_income_statement_accounts;
CREATE VIEW v_income_statement_accounts AS
WITH account_balances AS (
    SELECT 
        a.id,
        a.full_path,
        a.name,
        a.account_type,
        a.account_subtype,
        a.parent_id,
        COALESCE(SUM(
            CASE 
                WHEN a.account_type = 'income' THEN -je.amount
                WHEN a.account_type = 'expense' THEN je.amount
                ELSE 0
            END
        ), 0) as balance
    FROM accounts a
    LEFT JOIN journal_entries je ON a.id = je.account_id  
    WHERE a.account_type IN ('income', 'expense')
    GROUP BY a.id, a.full_path, a.name, a.account_type, a.account_subtype, a.parent_id
)
SELECT 
    id,
    full_path,
    name,
    account_type,
    account_subtype,
    parent_id,
    balance,
    ABS(balance) as abs_balance
FROM account_balances
WHERE balance != 0
ORDER BY account_type, full_path;

DROP VIEW IF EXISTS v_balance_sheet_accounts;
CREATE VIEW v_balance_sheet_accounts AS
WITH account_balances AS (
    SELECT 
        a.id,
        a.full_path,
        a.name,
        a.account_type,
        a.account_subtype,
        a.parent_id,
        COALESCE(SUM(
            CASE 
                WHEN a.account_type = 'asset' THEN je.amount
                WHEN a.account_type = 'liability' THEN -je.amount
                WHEN a.account_type = 'equity' THEN -je.amount
                ELSE 0
            END
        ), 0) as balance
    FROM accounts a
    LEFT JOIN journal_entries je ON a.id = je.account_id
    WHERE a.account_type IN ('asset', 'liability', 'equity')
    GROUP BY a.id, a.full_path, a.name, a.account_type, a.account_subtype, a.parent_id
)
SELECT 
    id,
    full_path,
    name,
    account_type,
    account_subtype,
    parent_id,
    balance,
    ABS(balance) as abs_balance
FROM account_balances
WHERE balance != 0
ORDER BY account_type, full_path;

DROP VIEW IF EXISTS v_account_balances;
CREATE VIEW v_account_balances AS
WITH account_movements AS (
    SELECT 
        a.id as account_id,
        a.full_path as account_path,
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
    GROUP BY a.id, a.full_path, a.name, a.account_type
)
SELECT 
    account_id,
    account_path,
    account_name,
    account_type,
    balance,
    ABS(balance) as abs_balance
FROM account_movements
ORDER BY account_path;
