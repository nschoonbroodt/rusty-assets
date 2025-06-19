use chrono::{DateTime, Utc};
use uuid::Uuid;

// File import tracking models
#[derive(Debug, sqlx::FromRow)]
pub struct ImportedFile {
    pub id: Uuid,
    pub file_path: String,
    pub file_name: String,
    pub file_hash: String,
    pub file_size: i64,
    pub import_source: String,
    pub import_batch_id: Uuid,
    pub imported_by: Uuid,
    pub imported_at: DateTime<Utc>,
    pub transaction_count: i32,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct NewImportedFile {
    pub file_path: String,
    pub file_name: String,
    pub file_hash: String,
    pub file_size: i64,
    pub import_source: String,
    pub import_batch_id: Uuid,
    pub imported_by: Uuid,
    pub transaction_count: i32,
    pub notes: Option<String>,
}
