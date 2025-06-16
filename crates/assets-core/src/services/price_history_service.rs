use crate::error::Result;
use crate::models::{Account, AccountWithMarketValue, NewPriceHistory, PriceHistory};
use rust_decimal::Decimal;
use sqlx::PgPool;

pub struct PriceHistoryService {
    pool: PgPool,
}

impl PriceHistoryService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Add or update a price entry for a symbol on a specific date
    pub async fn add_price(&self, new_price: NewPriceHistory) -> Result<PriceHistory> {
        let price = sqlx::query_as::<_, PriceHistory>(
            r#"
            INSERT INTO price_history (symbol, price, price_date, currency, source)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (symbol, price_date) 
            DO UPDATE SET 
                price = EXCLUDED.price,
                currency = EXCLUDED.currency,
                source = EXCLUDED.source,
                created_at = NOW()
            RETURNING id, symbol, price, price_date, currency, source, created_at
            "#,
        )
        .bind(new_price.symbol.to_uppercase())
        .bind(new_price.price)
        .bind(new_price.price_date)
        .bind(&new_price.currency)
        .bind(&new_price.source)
        .fetch_one(&self.pool)
        .await?;

        Ok(price)
    }

    /// Get the latest price for a symbol
    pub async fn get_latest_price(&self, symbol: &str) -> Result<Option<PriceHistory>> {
        let price = sqlx::query_as::<_, PriceHistory>(
            r#"
            SELECT id, symbol, price, price_date, currency, source, created_at
            FROM price_history
            WHERE symbol = $1
            ORDER BY price_date DESC, created_at DESC -- Ensure consistent ordering
            LIMIT 1
            "#,
        )
        .bind(symbol.to_uppercase())
        .fetch_optional(&self.pool)
        .await?;

        Ok(price)
    }

    /// Get price history for a symbol over a date range
    pub async fn get_price_history(
        &self,
        symbol: &str,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
    ) -> Result<Vec<PriceHistory>> {
        let mut query = "SELECT id, symbol, price, price_date, currency, source, created_at FROM price_history WHERE symbol = $1".to_string();
        let mut params = vec![symbol.to_uppercase()];
        let mut param_count = 1;

        if let Some(start) = start_date {
            param_count += 1;
            query.push_str(&format!(" AND price_date >= ${}", param_count));
            params.push(start.to_string());
        }

        if let Some(end) = end_date {
            param_count += 1;
            query.push_str(&format!(" AND price_date <= ${}", param_count));
            params.push(end.to_string());
        }

        query.push_str(" ORDER BY price_date ASC");

        let mut query_builder = sqlx::query_as::<_, PriceHistory>(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let prices = query_builder.fetch_all(&self.pool).await?;
        Ok(prices)
    }

    /// Get all symbols that have price history
    pub async fn get_tracked_symbols(&self) -> Result<Vec<String>> {
        let symbols: Vec<String> =
            sqlx::query_scalar("SELECT DISTINCT symbol FROM price_history ORDER BY symbol")
                .fetch_all(&self.pool)
                .await?;

        Ok(symbols)
    }

    /// Calculate market value for an account with investments
    /// This function is now simplified by using the `latest_account_market_values` view.
    pub async fn get_account_with_market_value(
        &self,
        account: Account,
    ) -> Result<AccountWithMarketValue> {
        // Book value still needs to be calculated from journal entries,
        // as the view `latest_account_market_values` only provides market value.
        let book_value_result: Option<Decimal> = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0) FROM journal_entries WHERE account_id = $1",
        )
        .bind(account.id)
        .fetch_one(&self.pool)
        .await?;
        let book_value = book_value_result.unwrap_or_default();

        // Fetch market data from the new view
        let market_data_row = sqlx::query!(
            r#"
            SELECT
                asset_symbol,
                quantity,
                value_date,
                price_per_unit,
                value_currency,
                market_value,
                price_history_id,
                price_source,
                price_created_at
            FROM latest_account_market_values
            WHERE account_id = $1
            "#,
            account.id
        )
        .fetch_optional(&self.pool)
        .await?;

        let (market_value, unrealized_gain_loss, latest_price) = if let Some(row) = market_data_row
        {
            let price_history = PriceHistory {
                id: row.price_history_id.unwrap_or_else(uuid::Uuid::new_v4), // Handle potential NULL if view is not populated
                symbol: row.asset_symbol.unwrap_or_default(), // Handle potential NULL
                price: row.price_per_unit.unwrap_or_default(), // Handle potential NULL
                price_date: row
                    .value_date
                    .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()), // Handle potential NULL
                currency: row.value_currency.unwrap_or_default(), // Handle potential NULL
                source: row.price_source,
                created_at: row.price_created_at.unwrap_or_else(chrono::Utc::now), // Handle potential NULL
            };
            let mv = row.market_value;
            let ugl = mv.map(|m| m - book_value);
            (mv, ugl, Some(price_history))
        } else {
            // If no market data found (e.g., no price history for the symbol)
            (None, None, None)
        };

        Ok(AccountWithMarketValue {
            account,
            book_value,
            market_value,
            unrealized_gain_loss,
            latest_price,
        })
    }
}
