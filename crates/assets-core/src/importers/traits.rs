use crate::error::Result;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

#[async_trait]
pub trait TransactionImporter {
    /// Import transactions from a file path
    async fn import_from_file(&self, file_path: &str) -> Result<Vec<ImportedTransaction>>;

    /// Get the expected file format description
    fn format_description(&self) -> &'static str;

    /// Validate if this importer can handle the given file
    fn can_handle_file(&self, file_path: &str) -> Result<bool>;
}

/// Represents a transaction before it's converted to our internal format
#[derive(Debug, Clone)]
pub struct ImportedTransaction {
    pub date_op: NaiveDate,
    pub date_val: NaiveDate,
    pub description: String,
    pub amount: Decimal,
    pub category: Option<String>,
    pub category_parent: Option<String>,
    pub supplier: Option<String>,
    pub account_number: String,
    pub account_label: String,
    pub raw_data: HashMap<String, String>,
}
