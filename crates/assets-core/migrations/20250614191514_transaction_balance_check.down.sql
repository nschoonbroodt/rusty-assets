-- Drop the constraint trigger first
DROP TRIGGER IF EXISTS trigger_validate_transaction_balance ON journal_entries;
-- Drop the function
DROP FUNCTION IF EXISTS validate_transaction_balance();