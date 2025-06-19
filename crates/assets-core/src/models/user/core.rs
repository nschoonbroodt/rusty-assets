use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use bon::Builder;

/// User entity for multi-user support
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// New user data for creation
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct NewUser {
    #[builder(into)]
    pub name: String,
    #[builder(into)]
    pub display_name: String,
}
