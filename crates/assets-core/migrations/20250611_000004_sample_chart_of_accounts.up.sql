-- Sample Chart of Accounts following standard accounting principles
-- Account codes: 1xxx=Assets, 2xxx=Liabilities, 3xxx=Equity, 4xxx=Income, 5xxx=Expenses
-- ASSETS (1000-1999)
INSERT INTO accounts (code, name, account_type, account_subtype)
VALUES -- Cash and Bank Accounts
    ('1001', 'Checking Account', 'asset', 'checking'),
    ('1002', 'Savings Account', 'asset', 'savings'),
    ('1003', 'Cash', 'asset', 'cash'),
    -- Investment Accounts
    (
        '1100',
        'Brokerage Account',
        'asset',
        'investment_account'
    ),
    ('1101', 'Apple Inc. (AAPL)', 'asset', 'stocks'),
    ('1102', 'S&P 500 ETF (SPY)', 'asset', 'etf'),
    ('1103', 'Bitcoin', 'asset', 'crypto'),
    -- Real Estate and Property
    (
        '1200',
        'Primary Residence',
        'asset',
        'real_estate'
    ),
    (
        '1201',
        'Rental Property',
        'asset',
        'real_estate'
    );
-- LIABILITIES (2000-2999)
INSERT INTO accounts (code, name, account_type, account_subtype)
VALUES (
        '2001',
        'Credit Card',
        'liability',
        'credit_card'
    ),
    ('2002', 'Home Mortgage', 'liability', 'mortgage'),
    ('2003', 'Car Loan', 'liability', 'loan');
-- EQUITY (3000-3999)
INSERT INTO accounts (code, name, account_type, account_subtype)
VALUES (
        '3001',
        'Opening Balance Equity',
        'equity',
        'opening_balance'
    ),
    (
        '3002',
        'Retained Earnings',
        'equity',
        'retained_earnings'
    );
-- INCOME (4000-4999)
INSERT INTO accounts (code, name, account_type, account_subtype)
VALUES ('4001', 'Salary', 'income', 'salary'),
    ('4002', 'Dividend Income', 'income', 'dividend'),
    ('4003', 'Interest Income', 'income', 'interest'),
    (
        '4004',
        'Capital Gains',
        'income',
        'capital_gains'
    );
-- EXPENSES (5000-5999)
INSERT INTO accounts (code, name, account_type, account_subtype)
VALUES ('5001', 'Groceries', 'expense', 'food'),
    (
        '5002',
        'Rent/Mortgage Payment',
        'expense',
        'housing'
    ),
    ('5003', 'Utilities', 'expense', 'utilities'),
    ('5004', 'Gas', 'expense', 'transportation'),
    ('5005', 'Restaurants', 'expense', 'food'),
    (
        '5006',
        'Entertainment',
        'expense',
        'entertainment'
    ),
    ('5007', 'Investment Fees', 'expense', 'fees');