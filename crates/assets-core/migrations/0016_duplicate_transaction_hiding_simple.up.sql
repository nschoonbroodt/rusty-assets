-- Migration to add duplicate transaction hiding functionality

-- Add field to track if a transaction is hidden due to being a confirmed duplicate
ALTER TABLE transactions ADD COLUMN is_duplicate BOOLEAN DEFAULT FALSE;
ALTER TABLE transactions ADD COLUMN merged_into_transaction_id UUID REFERENCES transactions(id);

-- Create index for efficient filtering of non-duplicate transactions
CREATE INDEX idx_transactions_is_duplicate ON transactions(is_duplicate);
CREATE INDEX idx_transactions_merged_into ON transactions(merged_into_transaction_id);
