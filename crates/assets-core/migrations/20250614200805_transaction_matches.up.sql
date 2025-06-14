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

CREATE INDEX idx_transaction_matches_status ON transaction_matches(status);