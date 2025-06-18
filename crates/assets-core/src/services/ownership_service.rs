use crate::error::Result;
use crate::models::{Account, AccountOwnership, AccountWithOwnership};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct OwnershipService {
    pool: PgPool,
}

impl OwnershipService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get ownership for a specific account
    pub async fn get_account_ownership(&self, account_id: Uuid) -> Result<Vec<AccountOwnership>> {
        let ownership = sqlx::query_as::<_, AccountOwnership>(
            r#"
            SELECT id, user_id, account_id, ownership_percentage, created_at
            FROM account_ownership
            WHERE account_id = $1
            ORDER BY ownership_percentage DESC
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(ownership)
    }

    /// Get all accounts owned by a user (with ownership percentage)
    pub async fn get_user_accounts(&self, user_id: Uuid) -> Result<Vec<AccountWithOwnership>> {
        // For now, we'll use a simpler approach that doesn't rely on compile-time query validation
        let ownership_records = sqlx::query_as::<_, AccountOwnership>(
            "SELECT id, user_id, account_id, ownership_percentage, created_at FROM account_ownership WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for ownership in ownership_records {
            // Get the account details
            if let Ok(Some(account)) = sqlx::query_as::<_, Account>(
                r#"
                SELECT 
                    id, code, name, account_type, account_subtype,
                    parent_id, symbol, quantity, average_cost, address,
                    purchase_date, purchase_price, currency, is_active,
                    notes, created_at, updated_at
                FROM accounts WHERE id = $1 AND is_active = true
                "#,
            )
            .bind(ownership.account_id)
            .fetch_optional(&self.pool)
            .await
            {
                // Get full ownership info for this account
                let all_ownership = self.get_account_ownership(ownership.account_id).await?;

                // Calculate user's balance
                let total_balance = account.calculate_balance(&self.pool).await?;
                let user_balance =
                    total_balance * ownership.ownership_percentage / Decimal::from(100);

                result.push(AccountWithOwnership {
                    account,
                    ownership: all_ownership,
                    user_balance: Some(user_balance),
                    user_percentage: Some(ownership.ownership_percentage),
                });
            }
        }

        Ok(result)
    }

    /// Set ownership for an account (replaces existing ownership)
    pub async fn set_account_ownership(
        &self,
        account_id: Uuid,
        ownership: Vec<(Uuid, Decimal)>,
    ) -> Result<()> {
        // Validate that percentages sum to 100 or less
        let total: Decimal = ownership.iter().map(|(_, pct)| pct).sum();
        if total > Decimal::from(100) {
            return Err(crate::error::CoreError::InvalidInput(
                "Total ownership percentage cannot exceed 100%".to_string(),
            ));
        }

        let mut tx = self.pool.begin().await?;

        // Delete existing ownership
        sqlx::query("DELETE FROM account_ownership WHERE account_id = $1")
            .bind(account_id)
            .execute(&mut *tx)
            .await?;

        // Insert new ownership
        for (user_id, percentage) in ownership {
            sqlx::query(
                "INSERT INTO account_ownership (user_id, account_id, ownership_percentage) VALUES ($1, $2, $3)"
            )
            .bind(user_id)
            .bind(account_id)
            .bind(percentage)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
