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
