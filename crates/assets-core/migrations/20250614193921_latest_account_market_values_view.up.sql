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