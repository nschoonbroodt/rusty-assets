-- Double-entry bookkeeping redesign
-- Drop old tables (we'll recreate with proper structure)
DROP TABLE IF EXISTS transactions CASCADE;
DROP TABLE IF EXISTS investments CASCADE;
DROP TABLE IF EXISTS real_estate CASCADE;
DROP TABLE IF EXISTS accounts CASCADE;
DROP TABLE IF EXISTS categories CASCADE;
-- Drop old types
DROP TYPE IF EXISTS account_type CASCADE;
DROP TYPE IF EXISTS investment_type CASCADE;
-- Create new account types following double-entry principles
CREATE TYPE account_type AS ENUM (
    'asset',
    -- Cash, investments, real estate, equipment
    'liability',
    -- Credit cards, loans, mortgages
    'equity',
    -- Owner's equity, retained earnings
    'income',
    -- Salary, dividends, capital gains
    'expense' -- Food, utilities, taxes, fees
);
CREATE TYPE account_subtype AS ENUM (
    -- Asset subtypes
    'cash',
    'checking',
    'savings',
    'investment_account',
    'stocks',
    'etf',
    'bonds',
    'crypto',
    'real_estate',
    'equipment',
    'other_asset',
    -- Liability subtypes  
    'credit_card',
    'loan',
    'mortgage',
    'other_liability',
    -- Equity subtypes
    'opening_balance',
    'retained_earnings',
    'owner_equity',
    -- Income subtypes
    'salary',
    'dividend',
    'interest',
    'capital_gains',
    'other_income',
    -- Expense subtypes
    'food',
    'housing',
    'transportation',
    'utilities',
    'entertainment',
    'healthcare',
    'taxes',
    'fees',
    'other_expense'
);
-- Chart of Accounts - the foundation of double-entry
CREATE TABLE accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(20) UNIQUE NOT NULL,
    -- e.g., "1001", "2001", "4001"
    name VARCHAR(255) NOT NULL,
    account_type account_type NOT NULL,
    account_subtype account_subtype NOT NULL,
    parent_id UUID REFERENCES accounts(id) ON DELETE
    SET NULL,
        -- Asset-specific fields (null for non-assets)
        symbol VARCHAR(20),
        -- Stock/ETF symbol
        quantity DECIMAL(15, 6),
        -- Shares/units owned
        average_cost DECIMAL(15, 2),
        -- Average cost basis
        -- Real estate specific
        address TEXT,
        purchase_date TIMESTAMPTZ,
        purchase_price DECIMAL(15, 2),
        -- General
        currency VARCHAR(3) NOT NULL DEFAULT 'USD',
        is_active BOOLEAN NOT NULL DEFAULT true,
        notes TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
-- Transactions table (header for each transaction)
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    description TEXT NOT NULL,
    reference VARCHAR(100),
    -- Check number, transfer ID, etc.
    transaction_date TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
-- Journal entries (the actual debits and credits)
CREATE TABLE journal_entries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transaction_id UUID NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    -- Amount is positive for debits, negative for credits
    -- Or we could use separate debit/credit columns
    amount DECIMAL(15, 2) NOT NULL,
    memo TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
-- Ensure transactions balance (sum of journal entries = 0)
CREATE OR REPLACE FUNCTION check_transaction_balance() RETURNS TRIGGER AS $$ BEGIN IF (
        SELECT SUM(amount)
        FROM journal_entries
        WHERE transaction_id = NEW.transaction_id
    ) != 0 THEN RAISE EXCEPTION 'Transaction does not balance: sum of journal entries must equal zero';
END IF;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
-- Trigger to check balance after insert/update/delete
CREATE TRIGGER check_journal_entry_balance
AFTER
INSERT
    OR
UPDATE
    OR DELETE ON journal_entries FOR EACH ROW EXECUTE FUNCTION check_transaction_balance();
-- Indexes for performance
CREATE INDEX idx_journal_entries_transaction_id ON journal_entries(transaction_id);
CREATE INDEX idx_journal_entries_account_id ON journal_entries(account_id);
CREATE INDEX idx_transactions_date ON transactions(transaction_date);
CREATE INDEX idx_accounts_type ON accounts(account_type);
CREATE INDEX idx_accounts_code ON accounts(code);
CREATE INDEX idx_accounts_parent_id ON accounts(parent_id);
-- Update trigger for accounts
CREATE TRIGGER update_accounts_updated_at BEFORE
UPDATE ON accounts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();