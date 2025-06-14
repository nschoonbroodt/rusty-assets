-- Fix date arithmetic in the duplicate detection function

CREATE OR REPLACE FUNCTION fn_find_potential_duplicates(
    p_transaction_id UUID,
    p_amount_tolerance DECIMAL DEFAULT 0.01,
    p_date_tolerance_days INTEGER DEFAULT 3
)
RETURNS TABLE (
    potential_duplicate_id UUID,
    match_confidence DECIMAL,
    match_criteria JSONB
) AS $$
DECLARE
    v_transaction RECORD;
    v_amount_range_min DECIMAL;
    v_amount_range_max DECIMAL;
    v_date_range_start DATE;
    v_date_range_end DATE;
BEGIN
    -- Get the reference transaction
    SELECT t.transaction_date, t.description, 
           COALESCE(SUM(ABS(je.amount)), 0) / 2 as total_amount
    INTO v_transaction
    FROM transactions t
    LEFT JOIN journal_entries je ON t.id = je.transaction_id
    WHERE t.id = p_transaction_id
    GROUP BY t.id, t.transaction_date, t.description;
    
    IF NOT FOUND THEN
        RETURN;
    END IF;
    
    -- Calculate search ranges (fix date arithmetic)
    v_amount_range_min := v_transaction.total_amount - p_amount_tolerance;
    v_amount_range_max := v_transaction.total_amount + p_amount_tolerance;
    v_date_range_start := v_transaction.transaction_date - INTERVAL '1 day' * p_date_tolerance_days;
    v_date_range_end := v_transaction.transaction_date + INTERVAL '1 day' * p_date_tolerance_days;
    
    -- Find potential matches
    RETURN QUERY
    SELECT 
        t2.id as potential_duplicate_id,
        -- Calculate confidence score based on multiple factors
        CASE 
            WHEN ABS(COALESCE(SUM(ABS(je2.amount)), 0) / 2 - v_transaction.total_amount) < 0.01 
                 AND t2.transaction_date = v_transaction.transaction_date
                 AND similarity(t2.description, v_transaction.description) > 0.8
            THEN 0.95
            WHEN ABS(COALESCE(SUM(ABS(je2.amount)), 0) / 2 - v_transaction.total_amount) < p_amount_tolerance
                 AND t2.transaction_date BETWEEN v_date_range_start AND v_date_range_end
                 AND similarity(t2.description, v_transaction.description) > 0.6
            THEN 0.80
            WHEN ABS(COALESCE(SUM(ABS(je2.amount)), 0) / 2 - v_transaction.total_amount) < p_amount_tolerance
                 AND t2.transaction_date BETWEEN v_date_range_start AND v_date_range_end
            THEN 0.60
            ELSE 0.30
        END as match_confidence,
        -- Store match criteria as JSON
        jsonb_build_object(
            'amount_diff', ABS(COALESCE(SUM(ABS(je2.amount)), 0) / 2 - v_transaction.total_amount),
            'date_diff_days', ABS(EXTRACT(days FROM t2.transaction_date - v_transaction.transaction_date)),
            'description_similarity', similarity(t2.description, v_transaction.description),
            'same_date', t2.transaction_date = v_transaction.transaction_date,
            'same_amount', ABS(COALESCE(SUM(ABS(je2.amount)), 0) / 2 - v_transaction.total_amount) < 0.01
        ) as match_criteria
    FROM transactions t2
    LEFT JOIN journal_entries je2 ON t2.id = je2.transaction_id
    WHERE t2.id != p_transaction_id
      AND t2.transaction_date BETWEEN v_date_range_start AND v_date_range_end
      AND t2.import_source IS DISTINCT FROM (
          SELECT import_source FROM transactions WHERE id = p_transaction_id
      )
    GROUP BY t2.id, t2.transaction_date, t2.description
    HAVING ABS(COALESCE(SUM(ABS(je2.amount)), 0) / 2 - v_transaction.total_amount) <= p_amount_tolerance
    ORDER BY match_confidence DESC;
END;
$$ LANGUAGE plpgsql;
