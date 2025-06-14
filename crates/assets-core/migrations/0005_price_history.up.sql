-- Price history table for tracking asset prices over time
CREATE TABLE price_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol VARCHAR(20) NOT NULL,
    price DECIMAL(20, 8) NOT NULL,
    price_date DATE NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'EUR',
    source VARCHAR(50),
    -- e.g., 'manual', 'yahoo_finance', 'alpha_vantage'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(symbol, price_date)
);
-- Indexes for price history
CREATE INDEX idx_price_history_symbol ON price_history(symbol);
CREATE INDEX idx_price_history_date ON price_history(price_date);
CREATE INDEX idx_price_history_symbol_date ON price_history(symbol, price_date);
-- Index for latest price queries (most recent price per symbol)
CREATE INDEX idx_price_history_latest ON price_history(symbol, price_date DESC);