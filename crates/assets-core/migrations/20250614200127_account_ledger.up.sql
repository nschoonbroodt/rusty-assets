-- Create account ledger function to show transaction history for a specific account
CREATE FUNCTION fn_account_ledger(
    p_account_id UUID,
    p_start_date DATE,
    p_end_date DATE
)
RETURNS TABLE (
    transaction_date DATE,
    transaction_id UUID,
    description TEXT,
    reference TEXT,
    memo TEXT,
    debit_amount DECIMAL(19, 4),
    credit_amount DECIMAL(19, 4),
    running_balance DECIMAL(19, 4)
)
AS $$
DECLARE
    opening_balance DECIMAL(19, 4) := 0.0;
BEGIN
    -- Calculate opening balance (all transactions before start date)
    SELECT COALESCE(SUM(je.amount), 0.0) INTO opening_balance
    FROM journal_entries je
    INNER JOIN transactions t ON t.id = je.transaction_id
    WHERE je.account_id = p_account_id
      AND t.transaction_date < p_start_date;

    -- Return the ledger entries with running balance
    RETURN QUERY
    SELECT
        t.transaction_date::DATE as transaction_date,
        t.id as transaction_id,
        t.description::TEXT as description,
        COALESCE(t.reference, '')::TEXT as reference,
        COALESCE(je.memo, '')::TEXT as memo,
        CASE 
            WHEN je.amount > 0 THEN je.amount
            ELSE 0.0
        END as debit_amount,
        CASE 
            WHEN je.amount < 0 THEN ABS(je.amount)
            ELSE 0.0
        END as credit_amount,
        (opening_balance + SUM(je.amount) OVER (ORDER BY t.transaction_date, t.created_at, je.created_at))::DECIMAL(19, 4) as running_balance
    FROM 
        journal_entries je
    INNER JOIN 
        transactions t ON t.id = je.transaction_id
    WHERE 
        je.account_id = p_account_id
        AND t.transaction_date >= p_start_date
        AND t.transaction_date <= p_end_date
    ORDER BY 
        t.transaction_date, t.created_at, je.created_at;
END;
$$ LANGUAGE plpgsql;
