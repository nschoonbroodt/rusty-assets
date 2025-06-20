use crate::error::{CoreError, Result};
use crate::models::{NewUser, User};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Struct for updating existing users - all fields are optional
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserUpdates {
    pub display_name: Option<String>,
    // Note: name is not updatable due to uniqueness constraints
}

impl UserUpdates {
    /// Check if any fields are set for update
    pub fn has_updates(&self) -> bool {
        self.display_name.is_some()
    }
}

pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get all users
    pub async fn get_all_users(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, name, display_name, is_active, created_at FROM users WHERE is_active = true ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    /// Get user by name
    pub async fn get_user_by_name(&self, name: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, name, display_name, is_active, created_at FROM users WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, name, display_name, is_active, created_at FROM users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Create a new user with validation
    pub async fn create_user(&self, new_user: NewUser) -> Result<User> {
        // Validate input
        Self::validate_user_input(&new_user.name, &new_user.display_name)?;

        // Create user with proper error handling
        match sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (name, display_name)
            VALUES ($1, $2)
            RETURNING id, name, display_name, is_active, created_at
            "#,
        )
        .bind(&new_user.name)
        .bind(&new_user.display_name)
        .fetch_one(&self.pool)
        .await
        {
            Ok(user) => Ok(user),
            Err(sqlx::Error::Database(db_err)) if Self::is_unique_violation(db_err.as_ref()) => {
                Err(CoreError::InvalidInput(format!(
                    "User with name '{}' already exists",
                    new_user.name
                )))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Update an existing user
    pub async fn update_user(&self, user_id: Uuid, updates: UserUpdates) -> Result<User> {
        if !updates.has_updates() {
            return Err(CoreError::InvalidInput("No updates provided".to_string()));
        }

        // Validate updates
        if let Some(ref display_name) = updates.display_name {
            Self::validate_display_name(display_name)?;
        }

        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET 
                display_name = COALESCE($2, display_name),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND is_active = true
            RETURNING id, name, display_name, is_active, created_at
            "#,
        )
        .bind(user_id)
        .bind(&updates.display_name)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            CoreError::NotFound(format!("User with id {} not found or inactive", user_id))
        })?;

        Ok(user)
    }

    /// Deactivate a user (soft delete)
    pub async fn deactivate_user(&self, user_id: Uuid) -> Result<()> {
        let rows_affected = sqlx::query(
            "UPDATE users SET is_active = false, updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(CoreError::NotFound(format!(
                "User with id {} not found or already inactive",
                user_id
            )));
        }

        Ok(())
    }

    /// Reactivate a user
    pub async fn reactivate_user(&self, user_id: Uuid) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET is_active = true, updated_at = CURRENT_TIMESTAMP 
            WHERE id = $1 AND is_active = false
            RETURNING id, name, display_name, is_active, created_at
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            CoreError::NotFound(format!(
                "User with id {} not found or already active",
                user_id
            ))
        })?;

        Ok(user)
    }

    /// Get the first user (ordered by creation date) - used as default owner for accounts
    pub async fn get_first_user(&self) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, name, display_name, is_active, created_at FROM users WHERE is_active = true ORDER BY created_at ASC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Get all users with optional filtering
    pub async fn get_all_users_with_filter(&self, include_inactive: bool) -> Result<Vec<User>> {
        let query = if include_inactive {
            "SELECT id, name, display_name, is_active, created_at FROM users ORDER BY name"
        } else {
            "SELECT id, name, display_name, is_active, created_at FROM users WHERE is_active = true ORDER BY name"
        };

        let users = sqlx::query_as::<_, User>(query)
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }

    // Validation helper functions
    fn validate_user_input(name: &str, display_name: &str) -> Result<()> {
        Self::validate_name(name)?;
        Self::validate_display_name(display_name)?;
        Ok(())
    }

    fn validate_name(name: &str) -> Result<()> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(CoreError::InvalidInput("Name cannot be empty".to_string()));
        }
        if trimmed.len() > 50 {
            return Err(CoreError::InvalidInput(format!(
                "Name too long: {} characters (max 50)",
                trimmed.len()
            )));
        }
        // Check for invalid characters
        if trimmed.chars().any(|c| c.is_control() || c == '\0') {
            return Err(CoreError::InvalidInput(
                "Name contains invalid characters".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_display_name(display_name: &str) -> Result<()> {
        let trimmed = display_name.trim();
        if trimmed.is_empty() {
            return Err(CoreError::InvalidInput(
                "Display name cannot be empty".to_string(),
            ));
        }
        if trimmed.len() > 100 {
            return Err(CoreError::InvalidInput(format!(
                "Display name too long: {} characters (max 100)",
                trimmed.len()
            )));
        }
        // Check for invalid characters
        if trimmed.chars().any(|c| c.is_control() || c == '\0') {
            return Err(CoreError::InvalidInput(
                "Display name contains invalid characters".to_string(),
            ));
        }
        Ok(())
    }

    fn is_unique_violation(db_err: &dyn sqlx::error::DatabaseError) -> bool {
        db_err.code().as_deref() == Some("23505") // PostgreSQL unique violation code
    }
}

#[cfg(test)]
mod tests;
