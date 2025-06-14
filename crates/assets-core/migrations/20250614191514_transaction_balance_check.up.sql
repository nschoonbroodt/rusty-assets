CREATE FUNCTION validate_transaction_balance() RETURNS TRIGGER AS $$
DECLARE balance DECIMAL(20, 2);
trans_id UUID;
BEGIN -- Get the transaction ID from the affected row
trans_id := COALESCE(NEW.transaction_id, OLD.transaction_id);
-- Calculate the balance for this transaction
SELECT COALESCE(SUM(amount), 0) INTO balance
FROM journal_entries
WHERE transaction_id = trans_id;
--
-- Only check balance if transaction is not empty
IF balance != 0
AND EXISTS (
    SELECT 1
    FROM journal_entries
    WHERE transaction_id = trans_id
) THEN RAISE EXCEPTION 'Transaction % is not balanced. Sum of entries: %',
trans_id,
balance;
END IF;
RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;
-- Create a constraint trigger that fires at transaction commit
CREATE CONSTRAINT TRIGGER trigger_validate_transaction_balance
AFTER
INSERT
    OR
UPDATE
    OR DELETE ON journal_entries DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION validate_transaction_balance();