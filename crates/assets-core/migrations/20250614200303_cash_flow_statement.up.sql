-- Create cash flow statement function
-- Categorizes cash flows into Operating, Investing, and Financing activities
CREATE FUNCTION fn_cash_flow_statement(
    p_user_ids UUID[],
    p_start_date DATE,
    p_end_date DATE
)
RETURNS TABLE (
    activity_type TEXT,
    category_name TEXT,
    account_name TEXT,
    account_path TEXT,
    cash_flow DECIMAL(19, 4)
)
AS $$
BEGIN
    RETURN QUERY
    SELECT
        CASE
            -- Operating Activities: Daily life cash flows (income and routine expenses)
            WHEN a.account_type = 'income' THEN 'Operating'
            WHEN a.account_type = 'expense' AND a.account_subtype IN (
                'food', 'housing', 'transportation', 'communication', 'utilities', 
                'healthcare', 'personal', 'entertainment', 'fees', 'taxes'
            ) THEN 'Operating'
            
            -- Investing Activities: Investment and savings-related cash flows
            WHEN a.account_type = 'asset' AND a.account_subtype IN (
                'stocks', 'etf', 'bonds', 'mutual_fund', 'crypto', 'investment_account'
            ) THEN 'Investing'
            WHEN a.account_type = 'asset' AND a.account_subtype = 'savings' THEN 'Investing'
            WHEN a.account_type = 'expense' AND a.account_subtype = 'investment' THEN 'Investing'
            
            -- Financing Activities: Debt and equity-related cash flows
            WHEN a.account_type = 'liability' THEN 'Financing'
            WHEN a.account_type = 'equity' THEN 'Financing'
            WHEN a.account_type = 'asset' AND a.account_subtype IN ('loan', 'mortgage') THEN 'Financing'
            
            -- Default to Operating for uncategorized items
            ELSE 'Operating'
        END::TEXT AS activity_type,
        
        COALESCE(parent_acc.name, 'Uncategorized')::TEXT AS category_name,
        a.name::TEXT AS account_name,
        COALESCE(a.full_path, a.name)::TEXT AS account_path,
        
        -- Calculate cash flow (positive = cash inflow, negative = cash outflow)
        COALESCE(
            SUM(
                CASE
                    -- For income accounts: positive amounts are cash inflows
                    WHEN a.account_type = 'income' THEN (je.amount * -1.0 * ao.ownership_percentage)
                    -- For asset accounts: positive amounts are cash outflows (money leaving to buy assets)
                    WHEN a.account_type = 'asset' THEN (je.amount * -1.0 * ao.ownership_percentage)
                    -- For expense accounts: positive amounts are cash outflows  
                    WHEN a.account_type = 'expense' THEN (je.amount * -1.0 * ao.ownership_percentage)
                    -- For liability accounts: positive amounts are cash inflows (borrowing)
                    WHEN a.account_type = 'liability' THEN (je.amount * ao.ownership_percentage)
                    -- For equity accounts: positive amounts are cash inflows
                    WHEN a.account_type = 'equity' THEN (je.amount * ao.ownership_percentage)
                    ELSE 0.0
                END
            ), 0.0
        )::DECIMAL(19, 4) AS cash_flow
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
        AND t.transaction_date >= p_start_date
        AND t.transaction_date <= p_end_date
        -- Exclude opening balance transactions to focus on actual cash flows
        AND COALESCE(t.reference, '') != 'OPENING'
    GROUP BY
        a.account_type, a.account_subtype, parent_acc.name, a.id, a.name, a.full_path
    HAVING
        -- Only include accounts with meaningful cash flows
        ABS(COALESCE(
            SUM(
                CASE
                    WHEN a.account_type = 'income' THEN (je.amount * -1.0 * ao.ownership_percentage)
                    WHEN a.account_type = 'asset' THEN (je.amount * -1.0 * ao.ownership_percentage)
                    WHEN a.account_type = 'expense' THEN (je.amount * -1.0 * ao.ownership_percentage)
                    WHEN a.account_type = 'liability' THEN (je.amount * ao.ownership_percentage)
                    WHEN a.account_type = 'equity' THEN (je.amount * ao.ownership_percentage)
                    ELSE 0.0
                END
            ), 0.0
        )) > 0.01
    ORDER BY
        activity_type, category_name, a.name;
END;
$$ LANGUAGE plpgsql;
