-- Add full_path column to accounts table for efficient querying
ALTER TABLE accounts
ADD COLUMN full_path TEXT;
-- Create function to build account path
CREATE OR REPLACE FUNCTION build_account_path(account_id UUID) RETURNS TEXT AS $$
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
CREATE OR REPLACE FUNCTION update_account_full_path() RETURNS TRIGGER AS $$
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