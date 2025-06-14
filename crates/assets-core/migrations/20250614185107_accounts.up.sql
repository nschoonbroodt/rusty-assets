CREATE TABLE accounts (
    -- each account can hold one type of asset or liability
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    account_type account_type NOT NULL,
    account_subtype account_subtype NOT NULL,
    is_category BOOLEAN NOT NULL DEFAULT FALSE,  -- does not contains transactions, used for grouping
    parent_id UUID REFERENCES accounts(id), -- Self-referential foreign key for hierarchical structure
    symbol VARCHAR(20),
    quantity DECIMAL(20, 8),
    average_cost DECIMAL(20, 8),
     -- Real estate specific
    address TEXT,
    purchase_date TIMESTAMP WITH TIME ZONE,
    purchase_price DECIMAL(20, 2),
    -- General fields
    currency VARCHAR(3) NOT NULL DEFAULT 'EUR',
    is_active BOOLEAN NOT NULL DEFAULT true,
    notes TEXT,
    full_path TEXT, -- Full path for hierarchical queries -- will be populated by a trigger
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT chk_account_name_no_colon CHECK (name NOT LIKE '%:%'),
    CONSTRAINT uq_account_name_parent UNIQUE (name, parent_id) -- Ensures name is unique under a given parent
);

CREATE INDEX idx_accounts_type ON accounts(account_type);
CREATE INDEX idx_accounts_parent ON accounts(parent_id);

-- Partial unique index for root account names (parent_id IS NULL)
-- This ensures that root accounts (those without a parent) have unique names
CREATE UNIQUE INDEX idx_accounts_unique_root_name ON accounts (name)
WHERE parent_id IS NULL;

-- Create function to build account path
CREATE FUNCTION build_account_path(account_id UUID) RETURNS TEXT AS $$
DECLARE result_path TEXT;
BEGIN WITH RECURSIVE account_path AS (
    -- Start with the target account
    SELECT id,
        name,
        parent_id,
        name::TEXT as path,
        1 as depth
    FROM accounts
    WHERE id = account_id
    UNION ALL
    -- Recursively get parent accounts
    SELECT a.id,
        a.name,
        a.parent_id,
        a.name || ':' || ap.path as path,
        ap.depth + 1
    FROM accounts a
        INNER JOIN account_path ap ON a.id = ap.parent_id
)
SELECT path INTO result_path
FROM account_path
WHERE parent_id IS NULL;
RETURN COALESCE(
    result_path,
    (
        SELECT name
        FROM accounts
        WHERE id = account_id
    )
);
END;
$$ LANGUAGE plpgsql;
-- Create trigger function to update full_path
CREATE FUNCTION update_account_full_path() RETURNS TRIGGER AS $$
DECLARE affected_account_id UUID;
BEGIN -- Handle the current account
affected_account_id := COALESCE(NEW.id, OLD.id);
-- Update the full_path for the affected account and all its children
WITH RECURSIVE account_tree AS (
    -- Start with the affected account
    SELECT id
    FROM accounts
    WHERE id = affected_account_id
    UNION ALL
    -- Include all descendant accounts
    SELECT a.id
    FROM accounts a
        INNER JOIN account_tree at ON a.parent_id = at.id
)
UPDATE accounts
SET full_path = build_account_path(id)
WHERE id IN (
        SELECT id
        FROM account_tree
    );
RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;
-- Create trigger to maintain full_path
CREATE TRIGGER trigger_update_account_full_path
AFTER
INSERT
    OR
UPDATE OF name,
    parent_id ON accounts FOR EACH ROW EXECUTE FUNCTION update_account_full_path();
-- Populate existing accounts with full_path
UPDATE accounts
SET full_path = build_account_path(id);
-- Create index for efficient querying
CREATE INDEX idx_accounts_full_path ON accounts(full_path);