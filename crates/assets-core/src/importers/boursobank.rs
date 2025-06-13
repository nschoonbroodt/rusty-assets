use super::traits::{ImportedTransaction, TransactionImporter};
use crate::error::{CoreError, Result};
use async_trait::async_trait;
use chrono::NaiveDate;
use csv::ReaderBuilder;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

pub struct BoursoBankImporter {
    pub default_account_path: String,
}

impl BoursoBankImporter {
    pub fn new(default_account_path: String) -> Self {
        Self {
            default_account_path,
        }
    }
}

#[async_trait]
impl TransactionImporter for BoursoBankImporter {
    async fn import_from_file(&self, file_path: &str) -> Result<Vec<ImportedTransaction>> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_path(file_path)
            .map_err(|e| CoreError::ImportError(format!("Failed to open CSV: {}", e)))?;

        let mut transactions = Vec::new();

        for result in reader.deserialize() {
            let record: BoursoBankCsvRecord = result
                .map_err(|e| CoreError::ImportError(format!("Failed to parse CSV row: {}", e)))?;

            transactions.push(record.into_imported_transaction()?);
        }

        Ok(transactions)
    }

    fn format_description(&self) -> &'static str {
        "BoursoBank CSV format: dateOp;dateVal;label;category;categoryParent;supplierFound;amount;comment;accountNum;accountLabel;accountbalance"
    }

    fn can_handle_file(&self, file_path: &str) -> Result<bool> {
        if !file_path.ends_with(".csv") {
            return Ok(false);
        }

        // Check if file has BoursoBank headers
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_path(file_path)
            .map_err(|e| CoreError::ImportError(format!("Failed to read file: {}", e)))?;

        let headers = reader
            .headers()
            .map_err(|e| CoreError::ImportError(format!("Failed to read headers: {}", e)))?;

        // Check for BoursoBank specific headers
        let expected_headers = [
            "dateOp",
            "dateVal",
            "label",
            "category",
            "categoryParent",
            "supplierFound",
            "amount",
        ];
        let has_bourso_headers = expected_headers
            .iter()
            .all(|&header| headers.iter().any(|h| h == header));

        Ok(has_bourso_headers)
    }
}

#[derive(Debug, serde::Deserialize)]
struct BoursoBankCsvRecord {
    #[serde(rename = "dateOp")]
    date_op: String,
    #[serde(rename = "dateVal")]
    date_val: String,
    #[serde(rename = "label")]
    label: String,
    #[serde(rename = "category")]
    category: String,
    #[serde(rename = "categoryParent")]
    category_parent: String,
    #[serde(rename = "supplierFound")]
    supplier_found: String,
    #[serde(rename = "amount")]
    amount: String,
    #[serde(rename = "comment")]
    comment: String,
    #[serde(rename = "accountNum")]
    account_num: String,
    #[serde(rename = "accountLabel")]
    account_label: String,
    #[serde(rename = "accountbalance")]
    account_balance: String,
}

impl BoursoBankCsvRecord {
    fn into_imported_transaction(self) -> Result<ImportedTransaction> {
        let date_op = NaiveDate::parse_from_str(&self.date_op, "%Y-%m-%d").map_err(|e| {
            CoreError::ImportError(format!("Invalid dateOp format '{}': {}", self.date_op, e))
        })?;

        let date_val = NaiveDate::parse_from_str(&self.date_val, "%Y-%m-%d").map_err(|e| {
            CoreError::ImportError(format!("Invalid dateVal format '{}': {}", self.date_val, e))
        })?;

        // Parse amount - BoursoBank uses comma as decimal separator and quotes around negative values
        let amount_str = self
            .amount
            .trim_matches('"')
            .replace(',', ".")
            .replace(' ', "");

        let amount = Decimal::from_str(&amount_str).map_err(|e| {
            CoreError::ImportError(format!("Invalid amount '{}': {}", self.amount, e))
        })?;

        let mut raw_data = HashMap::new();
        raw_data.insert("original_date_op".to_string(), self.date_op);
        raw_data.insert("original_date_val".to_string(), self.date_val);
        raw_data.insert("original_amount".to_string(), self.amount);
        raw_data.insert("comment".to_string(), self.comment);
        raw_data.insert("account_balance".to_string(), self.account_balance);

        Ok(ImportedTransaction {
            date_op,
            date_val,
            description: self.label,
            amount,
            category: if self.category.is_empty() || self.category == "Non catégorisé" {
                None
            } else {
                Some(self.category)
            },
            category_parent: if self.category_parent.is_empty()
                || self.category_parent == "Non catégorisé"
            {
                None
            } else {
                Some(self.category_parent)
            },
            supplier: if self.supplier_found.is_empty() {
                None
            } else {
                Some(self.supplier_found)
            },
            account_number: self.account_num,
            account_label: self.account_label,
            raw_data,
        })
    }
}
