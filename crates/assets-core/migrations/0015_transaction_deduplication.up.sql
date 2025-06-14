-- Migration to add transaction deduplication and matching support

-- Add source information to track where transactions come from
ALTER TABLE transactions ADD COLUMN import_source VARCHAR(50);
ALTER TABLE transactions ADD COLUMN import_batch_id UUID;
ALTER TABLE transactions ADD COLUMN external_reference VARCHAR(255);

-- Create table to track transaction matches/duplicates
CREATE TABLE transaction_matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    primary_transaction_id UUID NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    duplicate_transaction_id UUID NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    match_confidence DECIMAL(3,2) NOT NULL CHECK (match_confidence >= 0 AND match_confidence <= 1),
    match_criteria JSONB NOT NULL, -- Store what criteria matched (amount, date, description patterns)
    match_type VARCHAR(50) NOT NULL, -- 'EXACT', 'PROBABLE', 'POSSIBLE'
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING', -- 'PENDING', 'CONFIRMED', 'REJECTED'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT uq_transaction_match UNIQUE (primary_transaction_id, duplicate_transaction_id),
    CONSTRAINT chk_different_transactions CHECK (primary_transaction_id != duplicate_transaction_id)
);

-- Create indexes for efficient matching queries
CREATE INDEX idx_transactions_import_source ON transactions(import_source);
CREATE INDEX idx_transactions_import_batch ON transactions(import_batch_id);
CREATE INDEX idx_transactions_external_ref ON transactions(external_reference);
CREATE INDEX idx_transactions_amount_date ON transactions(transaction_date, description);
CREATE INDEX idx_transaction_matches_status ON transaction_matches(status);

-- Function to find potential duplicate transactions
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

-- Enable pg_trgm extension for text similarity if not already enabled
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- View to show transactions with their potential duplicates
CREATE VIEW v_transactions_with_duplicates AS
SELECT 
    t.id,
    t.description,
    t.transaction_date,
    t.import_source,
    t.import_batch_id,
    COALESCE(SUM(ABS(je.amount)), 0) / 2 as amount,
    COUNT(tm1.duplicate_transaction_id) + COUNT(tm2.primary_transaction_id) as duplicate_count,
    CASE 
        WHEN COUNT(tm1.duplicate_transaction_id) + COUNT(tm2.primary_transaction_id) > 0 
        THEN TRUE 
        ELSE FALSE 
    END as has_duplicates
FROM transactions t
LEFT JOIN journal_entries je ON t.id = je.transaction_id
LEFT JOIN transaction_matches tm1 ON t.id = tm1.primary_transaction_id
LEFT JOIN transaction_matches tm2 ON t.id = tm2.duplicate_transaction_id
GROUP BY t.id, t.description, t.transaction_date, t.import_source, t.import_batch_id
ORDER BY t.transaction_date DESC;
