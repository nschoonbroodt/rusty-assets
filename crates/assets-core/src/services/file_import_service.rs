use crate::error::Result;
use crate::models::{ImportedFile, NewImportedFile};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub struct FileImportService {
    pool: PgPool,
}

impl FileImportService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Calculate SHA-256 hash of a file
    pub fn calculate_file_hash<P: AsRef<Path>>(file_path: P) -> Result<String> {
        let content = fs::read(file_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Get file size in bytes
    pub fn get_file_size<P: AsRef<Path>>(file_path: P) -> Result<i64> {
        let metadata = fs::metadata(file_path)?;
        Ok(metadata.len() as i64)
    }    /// Check if a file has already been imported (by hash)
    pub async fn is_file_already_imported(&self, file_hash: &str) -> Result<bool> {
        let count: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM imported_files WHERE file_hash = $1"
        )
        .bind(file_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.unwrap_or(0) > 0)
    }    /// Check if a file path has already been imported for a specific source
    pub async fn is_file_path_already_imported(
        &self,
        file_path: &str,
        import_source: &str,
    ) -> Result<bool> {
        let count: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM imported_files WHERE file_path = $1 AND import_source = $2"
        )
        .bind(file_path)
        .bind(import_source)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.unwrap_or(0) > 0)
    }
    /// Get previously imported file by hash
    pub async fn get_imported_file_by_hash(&self, file_hash: &str) -> Result<Option<ImportedFile>> {
        let file =
            sqlx::query_as::<_, ImportedFile>("SELECT * FROM imported_files WHERE file_hash = $1")
                .bind(file_hash)
                .fetch_optional(&self.pool)
                .await?;

        Ok(file)
    }
    /// Record a file import
    pub async fn record_file_import(&self, new_file: NewImportedFile) -> Result<ImportedFile> {
        let file = sqlx::query_as::<_, ImportedFile>(
            r#"
            INSERT INTO imported_files (
                file_path, file_name, file_hash, file_size, import_source,
                import_batch_id, imported_by, transaction_count, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(&new_file.file_path)
        .bind(&new_file.file_name)
        .bind(&new_file.file_hash)
        .bind(new_file.file_size)
        .bind(&new_file.import_source)
        .bind(new_file.import_batch_id)
        .bind(new_file.imported_by)
        .bind(new_file.transaction_count)
        .bind(&new_file.notes)
        .fetch_one(&self.pool)
        .await?;

        Ok(file)
    }

    /// List all imported files with optional filtering
    pub async fn list_imported_files(
        &self,
        import_source: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<ImportedFile>> {
        let limit = limit.unwrap_or(50);
        let files = match import_source {
            Some(source) => {
                sqlx::query_as::<_, ImportedFile>(
                    "SELECT * FROM imported_files WHERE import_source = $1 ORDER BY imported_at DESC LIMIT $2"
                )
                .bind(source)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, ImportedFile>(
                    "SELECT * FROM imported_files ORDER BY imported_at DESC LIMIT $1"
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(files)
    }

    /// Prepare file metadata for import
    pub fn prepare_file_metadata<P: AsRef<Path>>(
        &self,
        file_path: P,
        import_source: &str,
        import_batch_id: Uuid,
        imported_by: Uuid,
        transaction_count: i32,
        notes: Option<String>,
    ) -> Result<NewImportedFile> {
        let file_path_ref = file_path.as_ref();
        let file_hash = Self::calculate_file_hash(&file_path)?;
        let file_size = Self::get_file_size(&file_path)?;
        let file_name = file_path_ref
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(NewImportedFile {
            file_path: file_path_ref.to_string_lossy().to_string(),
            file_name,
            file_hash,
            file_size,
            import_source: import_source.to_string(),
            import_batch_id,
            imported_by,
            transaction_count,
            notes,
        })
    }
}
