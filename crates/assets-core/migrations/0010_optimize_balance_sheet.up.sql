-- Update balance sheet function to use the stored full_path column
CREATE OR REPLACE FUNCTION balance_sheet_data(report_date DATE) RETURNS TABLE (
        account_type TEXT,
        name TEXT,
        balance DECIMAL(20, 2),
        full_path TEXT,
        level INTEGER
    ) AS $$ BEGIN RETURN QUERY
SELECT a.account_type::TEXT,
    a.name::TEXT,
    COALESCE(SUM(je.amount), 0) as balance,
    a.full_path::TEXT,
    -- Calculate level from the number of colons in full_path
    (
        LENGTH(a.full_path) - LENGTH(REPLACE(a.full_path, ':', ''))
    ) as level
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
    a.full_path
HAVING COALESCE(SUM(je.amount), 0) != 0
ORDER BY a.account_type,
    a.full_path;
END;
$$ LANGUAGE plpgsql;