-- Migration: 000007_reporting_views.up.sql
-- Description: Creates views for monthly sampled balances and market values of assets.
-- Assumes account_running_balances view from 000006_balance_views.up.sql exists.
CREATE VIEW account_monthly_sampled_balances AS WITH relevant_accounts AS (
    -- Select all accounts you want to track
    SELECT id AS account_id,
        name AS account_name,
        account_type,
        account_subtype
    FROM accounts
),
month_series AS (
    -- Generate a series of the 1st day of each month
    SELECT date_trunc('month', dd)::date AS month_start_date
    FROM generate_series(
            date_trunc(
                'month',
                (
                    SELECT MIN(transaction_date)
                    FROM transactions
                )
            )::date,
            -- Start from the 1st of the month of the first transaction
            (
                SELECT MAX(transaction_date)
                FROM transactions
            )::date + INTERVAL '1 month',
            -- Go one month past last transaction date
            '1 month'::interval
        ) dd
),
accounts_months AS (
    -- Create a combination of every account and every month start date
    SELECT ra.account_id,
        ra.account_name,
        ra.account_type,
        ra.account_subtype,
        ms.month_start_date
    FROM relevant_accounts ra
        CROSS JOIN month_series ms
)
SELECT am.account_id,
    am.account_name,
    am.account_type,
    am.account_subtype,
    am.month_start_date,
    COALESCE(lb.running_balance, 0.00) AS balance_at_month_start
FROM accounts_months am
    LEFT JOIN LATERAL (
        SELECT arb.running_balance
        FROM account_running_balances arb
        WHERE arb.account_id = am.account_id
            AND arb.balance_day <= am.month_start_date
        ORDER BY arb.balance_day DESC
        LIMIT 1
    ) lb ON true
ORDER BY am.account_id,
    am.month_start_date;
-- View for daily market values based on available price history
CREATE VIEW daily_account_market_values AS
SELECT a.id AS account_id,
    a.name AS account_name,
    a.account_type,
    a.account_subtype,
    a.symbol AS asset_symbol,
    -- This is from the accounts table
    a.quantity,
    ph.price_date::date AS value_date,
    ph.price AS price_per_unit,
    ph.currency AS value_currency,
    (a.quantity * ph.price) AS market_value
FROM accounts a
    JOIN price_history ph ON a.symbol = ph.symbol -- Corrected: ph.symbol instead of ph.asset_symbol
WHERE a.symbol IS NOT NULL
    AND a.quantity IS NOT NULL
    AND a.quantity != 0
    AND a.account_subtype IN (
        'stocks',
        'bonds',
        'etf',
        'mutual_fund',
        'crypto',
        'investment_account'
    );
-- View for the latest market value of each asset-holding account
CREATE VIEW latest_account_market_values AS WITH ranked_prices AS (
    SELECT a.id AS account_id,
        a.name AS account_name,
        a.account_type,
        a.account_subtype,
        a.symbol AS asset_symbol,
        a.quantity,
        ph.price_date,
        -- This is the original date from price_history
        ph.price AS price_per_unit,
        ph.currency AS value_currency,
        (a.quantity * ph.price) AS market_value,
        ph.id AS price_history_id,
        -- Added for constructing PriceHistory struct
        ph.source AS price_source,
        -- Added
        ph.created_at AS price_created_at,
        -- Added
        ROW_NUMBER() OVER (
            PARTITION BY a.id
            ORDER BY ph.price_date DESC,
                ph.created_at DESC -- Added created_at for tie-breaking
        ) as rn
    FROM accounts a
        JOIN price_history ph ON a.symbol = ph.symbol
    WHERE a.symbol IS NOT NULL
        AND a.quantity IS NOT NULL
        AND a.quantity != 0
        AND a.account_subtype IN (
            'stocks',
            'bonds',
            'etf',
            'mutual_fund',
            'crypto',
            'investment_account'
        )
)
SELECT account_id,
    account_name,
    account_type,
    account_subtype,
    asset_symbol,
    quantity,
    price_date::date AS value_date,
    -- Cast to date for consistency if needed, ph.price_date is already DATE
    price_per_unit,
    value_currency,
    market_value,
    price_history_id,
    -- Added
    price_source,
    -- Added
    price_created_at -- Added
FROM ranked_prices
WHERE rn = 1;