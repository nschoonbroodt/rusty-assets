-- Account ownership table for multi-user account sharing
CREATE TABLE account_ownership (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    ownership_percentage DECIMAL(5,4) NOT NULL DEFAULT 1.0, -- e.g., 0.5 for 50%, 1.0 for 100%
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    UNIQUE(user_id, account_id)
);

-- Indexes for account ownership
CREATE INDEX idx_account_ownership_user ON account_ownership(user_id);
CREATE INDEX idx_account_ownership_account ON account_ownership(account_id);
CREATE INDEX idx_account_ownership_percentage ON account_ownership(ownership_percentage);