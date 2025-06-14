-- Create custom types for account classification
CREATE TYPE account_type AS ENUM (
    'asset',
    'liability',
    'equity',
    'income',
    'expense'
);
CREATE TYPE account_subtype AS ENUM (
    -- Asset subtypes
    'checking',
    'savings',
    'cash',
    'investment_account',
    'stocks',
    'bonds',
    'etf',
    'mutual_fund',
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
    'rental',
    'bonus',
    'investment',
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
    'communication',
    'personal',
    'other_expense',
    'category' -- Added category subtype
);