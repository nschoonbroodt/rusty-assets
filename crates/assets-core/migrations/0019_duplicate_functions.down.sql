-- Rollback duplicate transaction management functions
DROP FUNCTION IF EXISTS fn_unhide_duplicate_transaction(UUID);
DROP FUNCTION IF EXISTS fn_hide_duplicate_transaction(UUID, UUID);
