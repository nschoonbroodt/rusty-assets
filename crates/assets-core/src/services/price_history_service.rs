use crate::error::Result;
use crate::models::{Account, AccountWithMarketValue, NewPriceHistory, PriceHistory};
use sqlx::PgPool;

pub struct PriceHistoryService {
    pool: PgPool,
}

impl PriceHistoryService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Add or update a price entry for a symbol on a specific date
    pub async fn add_price(
        &self,
        new_price: NewPriceHistory,
    ) -> Result<PriceHistory> {
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
        .bind(&new_price.symbol.to_uppercase())
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
            ORDER BY price_date DESC
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
        let symbols: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT symbol FROM price_history ORDER BY symbol",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(symbols)
    }

    /// Calculate market value for an account with investments
    pub async fn get_account_with_market_value(
        &self,
        account: Account,
    ) -> Result<AccountWithMarketValue> {
        // Calculate book value from journal entries
        let book_value = account.calculate_balance(&self.pool).await
            .map_err(|e| crate::error::CoreError::Database(e))?;

        let mut market_value = None;
        let mut unrealized_gain_loss = None;
        let mut latest_price = None;

        // For investment accounts with symbol and quantity, calculate market value
        if let (Some(symbol), Some(quantity)) = (&account.symbol, account.quantity) {
            if let Ok(Some(price)) = self.get_latest_price(symbol).await {
                let current_market_value = quantity * price.price;
                market_value = Some(current_market_value);
                unrealized_gain_loss = Some(current_market_value - book_value);
                latest_price = Some(price);
            }
        }

        Ok(AccountWithMarketValue {
            account,
            book_value,
            market_value,
            unrealized_gain_loss,
            latest_price,
        })
    }
}
