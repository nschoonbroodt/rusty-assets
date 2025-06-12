-- Balance sheet function that accepts a date parameter
CREATE OR REPLACE FUNCTION balance_sheet_data(report_date DATE) RETURNS TABLE (
        account_type TEXT,
        name TEXT,
        balance DECIMAL(20, 2),
        full_path TEXT,
        level INTEGER
    ) AS $$ BEGIN RETURN QUERY WITH account_balances AS (
        SELECT a.id,
            a.name,
            a.account_type,
            a.parent_id,
            COALESCE(SUM(je.amount), 0) as account_balance
        FROM accounts a
            LEFT JOIN journal_entries je ON a.id = je.account_id
            LEFT JOIN transactions t ON je.transaction_id = t.id
        WHERE a.account_type IN ('asset', 'liability', 'equity')
            AND a.is_active = true
            AND (
                t.transaction_date IS NULL
                OR t.transaction_date <= report_date
            )
        GROUP BY a.id,
            a.name,
            a.account_type,
            a.parent_id
    ),
    account_hierarchy AS (
        WITH RECURSIVE account_path AS (
            SELECT ab.id,
                ab.name,
                ab.account_type,
                ab.parent_id,
                ab.account_balance,
                ab.name as full_path,
                0 as level
            FROM account_balances ab
            WHERE ab.parent_id IS NULL
            UNION ALL
            SELECT ab.id,
                ab.name,
                ab.account_type,
                ab.parent_id,
                ab.account_balance,
                ap.full_path || ':' || ab.name as full_path,
                ap.level + 1
            FROM account_balances ab
                JOIN account_path ap ON ab.parent_id = ap.id
        )
        SELECT *
        FROM account_path
    )
SELECT ah.account_type::TEXT,
    ah.name::TEXT,
    ah.account_balance,
    ah.full_path::TEXT,
    ah.level
FROM account_hierarchy ah
WHERE ah.account_balance != 0
ORDER BY ah.account_type,
    ah.full_path;
END;
$$ LANGUAGE plpgsql;