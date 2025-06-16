use super::traits::{ImportedTransaction, TransactionImporter};
use crate::error::{CoreError, Result};
use async_trait::async_trait;
use chrono::NaiveDate;
use csv::ReaderBuilder;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Default)]
pub struct SocietegeneraleImporter {}

#[async_trait]
impl TransactionImporter for SocietegeneraleImporter {
    async fn import_from_file(&self, file_path: &str) -> Result<Vec<ImportedTransaction>> {
        // Read file with proper encoding handling for French characters
        let content = std::fs::read(file_path)
            .map_err(|e| CoreError::ImportError(format!("Failed to read CSV: {}", e)))?;

        // Try UTF-8 first, then fallback to Windows-1252 for French characters
        let content_str = match String::from_utf8(content.clone()) {
            Ok(s) => s,
            Err(_) => {
                // Try to decode as Windows-1252 (common for French CSVs)
                encoding_rs::WINDOWS_1252.decode(&content).0.to_string()
            }
        };

        // Skip the first line which contains account info and the empty line
        let lines: Vec<&str> = content_str.lines().collect();
        if lines.len() < 3 {
            return Err(CoreError::ImportError("File too short".to_string()));
        }

        // Skip first line (account info), keep headers and data
        let csv_content = lines[2..].join("\n");

        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_reader(csv_content.as_bytes());

        let mut transactions = Vec::new();

        for result in reader.deserialize() {
            let record: SocietegeneraleCsvRecord = result
                .map_err(|e| CoreError::ImportError(format!("Failed to parse CSV row: {}", e)))?;

            transactions.push(record.into_imported_transaction()?);
        }

        Ok(transactions)
    }

    fn format_description(&self) -> &'static str {
        "Société Générale CSV format: Date de l'opération;Libellé;Détail de l'écriture;Montant de l'opération;Devise"
    }
    fn can_handle_file(&self, file_path: &str) -> Result<bool> {
        if !file_path.ends_with(".csv") {
            return Ok(false);
        }

        // Check if file contains SG-specific patterns
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| CoreError::ImportError(format!("Failed to read file: {}", e)))?;

        // Look for SG-specific patterns in the file
        Ok(content.contains("Date de l'opération") && content.contains("Détail de l'écriture"))
    }
}

#[derive(Debug, serde::Deserialize)]
struct SocietegeneraleCsvRecord {
    #[serde(rename = "Date de l'opération")]
    date_operation: String,
    #[serde(rename = "Libellé")]
    libelle: String,
    #[serde(rename = "Détail de l'écriture")]
    detail: String,
    #[serde(rename = "Montant de l'opération")]
    montant: String,
    #[serde(rename = "Devise")]
    devise: String,
}

impl SocietegeneraleCsvRecord {
    fn into_imported_transaction(self) -> Result<ImportedTransaction> {
        // Parse date (format: DD/MM/YYYY)
        let date = NaiveDate::parse_from_str(&self.date_operation, "%d/%m/%Y")
            .map_err(|e| CoreError::ImportError(format!("Invalid date format: {}", e)))?;

        // Parse amount (format: -1000,00 or 1000,00)
        let amount_str = self.montant.replace(" EUR", "").replace(",", ".");
        let amount = Decimal::from_str(&amount_str)
            .map_err(|e| CoreError::ImportError(format!("Invalid amount format: {}", e)))?;

        // Create description combining libellé and detail
        let description = if self.detail.trim().is_empty() || self.detail == self.libelle {
            self.libelle.clone()
        } else {
            format!("{} - {}", self.libelle, self.detail)
        };

        // Categorize based on transaction patterns
        let category = categorize_sg_transaction(&self.libelle, &self.detail);

        // Create raw data map
        let mut raw_data = HashMap::new();
        raw_data.insert("date_operation".to_string(), self.date_operation.clone());
        raw_data.insert("libelle".to_string(), self.libelle.clone());
        raw_data.insert("detail".to_string(), self.detail.clone());
        raw_data.insert("montant".to_string(), self.montant.clone());
        raw_data.insert("devise".to_string(), self.devise.clone());

        Ok(ImportedTransaction {
            date_op: date,
            date_val: date, // SG doesn't distinguish between operation and value date
            description,
            amount,
            category: Some(category),
            category_parent: None, // SG doesn't provide parent categories
            supplier: None,        // SG doesn't provide clear supplier info
            account_number: "".to_string(), // Will be filled by import service
            account_label: "".to_string(), // Will be filled by import service
            raw_data,
        })
    }
}

fn categorize_sg_transaction(libelle: &str, detail: &str) -> String {
    // TODO: Improve categorization
    let text = format!("{} {}", libelle, detail).to_lowercase();

    // Bank transfers and internal operations
    if text.contains("vir") && (text.contains("europee") || text.contains("europeen")) {
        return "Transfers:Wire".to_string();
    }
    if text.contains("vir inst") || text.contains("virement") {
        return "Transfers:Internal".to_string();
    }
    if text.contains("vir recu") {
        return "Income:Transfers".to_string();
    }

    // Card payments
    if text.contains("carte") {
        if text.contains("restaurant") || text.contains("resto") {
            return "Expenses:Food:Restaurants".to_string();
        }
        if text.contains("carrefour") || text.contains("monoprix") || text.contains("auchan") {
            return "Expenses:Food:Groceries".to_string();
        }
        if text.contains("essence") || text.contains("total") || text.contains("bp") {
            return "Expenses:Transportation:Fuel".to_string();
        }
        return "Expenses:Miscellaneous".to_string();
    }

    // Direct debits
    if text.contains("prlv") || text.contains("prelevement") {
        if text.contains("edf") || text.contains("gdf") || text.contains("engie") {
            return "Expenses:Utilities:Energy".to_string();
        }
        if text.contains("orange")
            || text.contains("sfr")
            || text.contains("free")
            || text.contains("bouygues")
        {
            return "Expenses:Utilities:Phone".to_string();
        }
        if text.contains("assurance") {
            return "Expenses:Insurance".to_string();
        }
        return "Expenses:Bills".to_string();
    }

    // Withdrawals
    if text.contains("retrait") || text.contains("dab") {
        return "Expenses:Cash".to_string();
    }

    // Fees
    if text.contains("commission") || text.contains("frais") || text.contains("cotisation") {
        return "Expenses:Bank:Fees".to_string();
    }

    // Default based on amount sign
    if text.starts_with('-') {
        "Expenses:Miscellaneous".to_string()
    } else {
        "Income:Miscellaneous".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    #[test]
    fn test_sg_record_parsing() {
        let record = SocietegeneraleCsvRecord {
            date_operation: "27/05/2025".to_string(),
            libelle: "000001 VIR EUROPEE".to_string(),
            detail: "000001 VIR EUROPEEN EMIS   LOGITEL POUR: M. NICOLAS SCHOONBROODT 27 05 SG 00485 CPT 00030470377 REF: 9514775507581".to_string(),
            montant: "-1000,00".to_string(),
            devise: "EUR".to_string(),
        };

        let transaction = record.into_imported_transaction().unwrap();

        assert_eq!(
            transaction.date_op,
            NaiveDate::from_ymd_opt(2025, 5, 27).unwrap()
        );
        assert_eq!(transaction.amount, Decimal::from_str("-1000.00").unwrap());
        assert_eq!(transaction.category, Some("Transfers:Wire".to_string()));
    }

    #[test]
    fn test_sg_categorization() {
        assert_eq!(
            categorize_sg_transaction("000001 VIR EUROPEE", "VIR EUROPEEN EMIS"),
            "Transfers:Wire"
        );
        assert_eq!(
            categorize_sg_transaction("VIR INST RE 564770", "VIR INST"),
            "Transfers:Internal"
        );
        assert_eq!(
            categorize_sg_transaction("VIR RECU 951356780", "VIR RECU DE: M. NICOLAS"),
            "Income:Transfers"
        );
    }
}
