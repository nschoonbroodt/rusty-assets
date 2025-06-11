-- Initial database schema for RustyAssets
-- Create UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create custom types for account classification
CREATE TYPE account_type AS ENUM ('asset', 'liability', 'equity', 'income', 'expense');

CREATE TYPE account_subtype AS ENUM (
    -- Asset subtypes
    'checking', 'savings', 'cash', 'investment_account', 'stocks', 'bonds', 'etf', 'mutual_fund', 'crypto', 'real_estate', 'equipment', 'other_asset',
    -- Liability subtypes  
    'credit_card', 'loan', 'mortgage', 'other_liability',
    -- Equity subtypes
    'opening_balance', 'retained_earnings', 'owner_equity',
    -- Income subtypes
    'salary', 'dividend', 'interest', 'capital_gains', 'rental', 'bonus', 'investment', 'other_income',
    -- Expense subtypes
    'food', 'housing', 'transportation', 'utilities', 'entertainment', 'healthcare', 'taxes', 'fees', 'communication', 'personal', 'other_expense'
);

-- Chart of Accounts table
CREATE TABLE accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    account_type account_type NOT NULL,
    account_subtype account_subtype NOT NULL,
    parent_id UUID REFERENCES accounts(id),
    
    -- Asset-specific fields (null for non-assets)
    symbol VARCHAR(20),        -- Stock/ETF symbol
    quantity DECIMAL(20,8),    -- Shares/units owned
    average_cost DECIMAL(20,8), -- Average cost basis
    
    -- Real estate specific
    address TEXT,
    purchase_date TIMESTAMP WITH TIME ZONE,
    purchase_price DECIMAL(20,2),
    
    -- General fields
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    is_active BOOLEAN NOT NULL DEFAULT true,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Transactions table (header for journal entries)
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    description VARCHAR(500) NOT NULL,
    reference VARCHAR(100), -- Check number, transfer ID, etc.
    transaction_date TIMESTAMP WITH TIME ZONE NOT NULL,
    created_by UUID, -- Will reference users table when created
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Journal entries table (actual debits and credits)
CREATE TABLE journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id UUID NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts(id),
    amount DECIMAL(20,2) NOT NULL, -- Positive for debits, negative for credits
    memo VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for better performance
CREATE INDEX idx_accounts_code ON accounts(code);
CREATE INDEX idx_accounts_type ON accounts(account_type);
CREATE INDEX idx_accounts_parent ON accounts(parent_id);
CREATE INDEX idx_transactions_date ON transactions(transaction_date);
CREATE INDEX idx_transactions_created_by ON transactions(created_by);
CREATE INDEX idx_journal_entries_transaction ON journal_entries(transaction_id);
CREATE INDEX idx_journal_entries_account ON journal_entries(account_id);
CREATE INDEX idx_journal_entries_amount ON journal_entries(amount);
