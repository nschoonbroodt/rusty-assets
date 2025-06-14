-- Migration to update views and add functions for duplicate transaction handling

-- Recreate the balance calculation views to exclude duplicates
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
    LEFT JOIN transactions t ON je.transaction_id = t.id
    WHERE t.is_duplicate = FALSE OR t.is_duplicate IS NULL  -- Exclude confirmed duplicates
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

-- Recreate the balance sheet view to exclude duplicates
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
    LEFT JOIN transactions t ON je.transaction_id = t.id
    WHERE (t.is_duplicate = FALSE OR t.is_duplicate IS NULL)  -- Exclude confirmed duplicates
      AND a.account_type IN ('asset', 'liability', 'equity')
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

-- Recreate income statement view to exclude duplicates
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
    LEFT JOIN transactions t ON je.transaction_id = t.id
    WHERE (t.is_duplicate = FALSE OR t.is_duplicate IS NULL)  -- Exclude confirmed duplicates
      AND a.account_type IN ('income', 'expense')
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

-- Create a view that shows all transactions including duplicates (for admin purposes)
CREATE VIEW v_all_transactions_with_duplicate_status AS
SELECT 
    t.*,
    CASE 
        WHEN t.is_duplicate = TRUE THEN 'DUPLICATE'
        WHEN EXISTS (
            SELECT 1 FROM transaction_matches tm 
            WHERE (tm.primary_transaction_id = t.id OR tm.duplicate_transaction_id = t.id) 
              AND tm.status = 'CONFIRMED'
        ) THEN 'HAS_DUPLICATES'
        ELSE 'NORMAL'
    END as duplicate_status,
    mt.description as merged_into_description
FROM transactions t
LEFT JOIN transactions mt ON t.merged_into_transaction_id = mt.id
ORDER BY t.transaction_date DESC, t.created_at DESC;
