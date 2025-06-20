CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    description VARCHAR(500) NOT NULL,
    reference VARCHAR(100),
    -- Check number, transfer ID, etc.
    transaction_date TIMESTAMP WITH TIME ZONE NOT NULL,
    import_source VARCHAR(50),
    import_batch_id UUID,
    external_reference VARCHAR(255),
    is_duplicate BOOLEAN DEFAULT FALSE,
    merged_into_transaction_id UUID REFERENCES transactions(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_transactions_date ON transactions(transaction_date);
CREATE INDEX idx_transactions_import_source ON transactions(import_source);
CREATE INDEX idx_transactions_import_batch ON transactions(import_batch_id);
CREATE INDEX idx_transactions_external_ref ON transactions(external_reference);
CREATE INDEX idx_transactions_amount_date ON transactions(transaction_date, description);
CREATE INDEX idx_transactions_is_duplicate ON transactions(is_duplicate);
CREATE INDEX idx_transactions_merged_into ON transactions(merged_into_transaction_id);
