DROP FUNCTION fn_unhide_duplicate_transaction(
    p_transaction_id UUID
);
DROP FUNCTION fn_hide_duplicate_transaction(
    p_duplicate_transaction_id UUID,
    p_primary_transaction_id UUID
);