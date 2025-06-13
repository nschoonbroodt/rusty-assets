-- Revert income statement function to original version
-- First drop the modified function
DROP FUNCTION IF EXISTS fn_income_statement(UUID[], DATE, DATE);

-- Then recreate the original function
CREATE OR REPLACE FUNCTION fn_income_statement(
    p_user_ids UUID[],
    p_start_date DATE,
    p_end_date DATE
)
RETURNS TABLE (
    category_name TEXT,
    account_name TEXT,
    total_amount DECIMAL(19, 4)
)
AS $$
BEGIN
    RETURN QUERY
    SELECT
        COALESCE(parent_acc.name, 'Uncategorized')::TEXT AS category_name,
        a.name::TEXT AS account_name,
        COALESCE(
            SUM(
                CASE
                    WHEN a.account_type = 'income' THEN (je.amount * -1.0 * ao.ownership_percentage / 100.0)
                    WHEN a.account_type = 'expense' THEN (je.amount * ao.ownership_percentage / 100.0)
                    ELSE 0.0
                END
            ), 0.0
        )::DECIMAL(19, 4) AS total_amount
    FROM
        accounts a
    INNER JOIN
        account_ownership ao ON a.id = ao.account_id
    INNER JOIN
        journal_entries je ON je.account_id = a.id
    INNER JOIN
        transactions t ON t.id = je.transaction_id
    LEFT JOIN
        accounts parent_acc ON a.parent_id = parent_acc.id
    WHERE
        ao.user_id = ANY(p_user_ids)
        AND a.account_type IN ('income', 'expense')
        AND t.transaction_date >= p_start_date
        AND t.transaction_date <= p_end_date
    GROUP BY
        parent_acc.name, a.id, a.name, a.account_type
    HAVING
        ABS(COALESCE(
            SUM(
                CASE
                    WHEN a.account_type = 'income' THEN (je.amount * ao.ownership_percentage / 100.0)
                    WHEN a.account_type = 'expense' THEN (je.amount * ao.ownership_percentage / 100.0)
                    ELSE 0.0
                END
            ), 0.0
        )) > 0.01;
END;
$$ LANGUAGE plpgsql;
