use crate::error::Result;
use crate::models::{User, NewUser};
use sqlx::PgPool;

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

    /// Create a new user
    pub async fn create_user(&self, new_user: NewUser) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (name, display_name)
            VALUES ($1, $2)
            RETURNING id, name, display_name, is_active, created_at
            "#,
        )
        .bind(&new_user.name)
        .bind(&new_user.display_name)
        .fetch_one(&self.pool)
        .await?;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::helpers::*;

    #[tokio::test]
    async fn test_create_user() {
        let (pool, _container) = setup_test_db().await;
        let service = UserService::new(pool);

        let new_user = NewUser::builder()
            .name("testuser")
            .display_name("Test User")
            .build();
        let result = service.create_user(new_user).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "testuser");
        assert_eq!(user.display_name, "Test User");
    }

    #[tokio::test]
    async fn test_get_all_users() {
        let (pool, _container) = setup_test_db().await;
        
        let user1 = create_test_user_with_names(&pool.clone(), "user1", "User One").await;
        let user2 = create_test_user_with_names(&pool.clone(), "user2", "User Two").await;
        
        let service = UserService::new(pool);
        let result = service.get_all_users().await;
        assert!(result.is_ok());
        let users = result.unwrap();
        assert_eq!(users.len(), 2);
        assert!(users.iter().any(|u| u.id == user1.id));
        assert!(users.iter().any(|u| u.id == user2.id));
    }

    #[tokio::test]
    async fn test_get_user_by_name() {
        let (pool, _container) = setup_test_db().await;
        
        let user1 = create_test_user_with_names(&pool.clone(), "user1", "User One").await;
        let _user2 = create_test_user_with_names(&pool.clone(), "user2", "User Two").await;
        
        let service = UserService::new(pool);
        let user = service.get_user_by_name("user1").await;
        assert!(user.is_ok());
        let user = user.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().id, user1.id);
    }

    #[tokio::test]
    async fn test_get_first_user() {
        let (pool, _container) = setup_test_db().await;
        let _user1 = create_test_user_with_names(&pool.clone(), "user1", "User One").await;
        let _user2 = create_test_user_with_names(&pool.clone(), "user2", "User Two").await;

        let service = UserService::new(pool);

        let user = service.get_first_user().await;
        assert!(user.is_ok());
        let user = user.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().name, "user1"); // Assuming "user1" is the first created user in setup
    }
}