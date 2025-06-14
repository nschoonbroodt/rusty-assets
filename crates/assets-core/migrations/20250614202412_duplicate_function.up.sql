-- Function to hide duplicate transaction and update references
CREATE FUNCTION fn_hide_duplicate_transaction(
    p_duplicate_transaction_id UUID,
    p_primary_transaction_id UUID
) RETURNS VOID AS $$
BEGIN
    -- Mark the duplicate transaction as hidden
    UPDATE transactions 
    SET is_duplicate = TRUE,
        merged_into_transaction_id = p_primary_transaction_id
    WHERE id = p_duplicate_transaction_id;
    
    -- Update any transaction matches to reflect the merge
    UPDATE transaction_matches 
    SET status = 'CONFIRMED'
    WHERE (primary_transaction_id = p_primary_transaction_id AND duplicate_transaction_id = p_duplicate_transaction_id)
       OR (primary_transaction_id = p_duplicate_transaction_id AND duplicate_transaction_id = p_primary_transaction_id);
       
END;
$$ LANGUAGE plpgsql;

-- Function to unhide a duplicate transaction (undo merge)
CREATE FUNCTION fn_unhide_duplicate_transaction(
    p_transaction_id UUID
) RETURNS VOID AS $$
BEGIN
    -- Unhide the transaction
    UPDATE transactions 
    SET is_duplicate = FALSE,
        merged_into_transaction_id = NULL
    WHERE id = p_transaction_id;
    
    -- Update transaction matches back to pending
    UPDATE transaction_matches 
    SET status = 'PENDING'
    WHERE primary_transaction_id = p_transaction_id 
       OR duplicate_transaction_id = p_transaction_id;
       
END;
$$ LANGUAGE plpgsql;
