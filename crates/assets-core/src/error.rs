use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Environment variable error: {0}")]
    Environment(#[from] std::env::VarError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Transaction does not balance: expected {expected}, got {actual}")]
    UnbalancedTransaction { expected: Decimal, actual: Decimal },

    #[error("Import error: {0}")]
    ImportError(String),

    #[error("Empty account name")]
    EmptyAccountName,

    #[error("Generic error: {0}")]
    Generic(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error(transparent)]
    PdfImportError(#[from] pdf_extract::OutputError),
}

pub type Result<T> = std::result::Result<T, CoreError>;
