use crate::error::{CoreError, Result};
use crate::importers::{ImportedPayslip, PayslipImporter};
use async_trait::async_trait;
use chrono::NaiveDate;
use log::{debug, info, warn};
use regex::Regex;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

/// MathWorks Payslip Importer
pub struct MathWorksPayslipImporter {}

#[async_trait]
impl PayslipImporter for MathWorksPayslipImporter {
    fn format_description(&self) -> &'static str {
        "MathWorks PDF payslip format"
    }

    fn can_handle_file(&self, file_path: &str) -> Result<bool> {
        Ok(file_path.to_lowercase().ends_with(".pdf"))
    }

    async fn import_from_file(&self, file_path: &str) -> Result<ImportedPayslip> {
        info!("Importing MathWorks Payslip from PDF...");

        let text = self.extract_text_from_pdf(file_path)?;
        let pay_date = self.extract_pay_date(&text)?;

        let gross_fixed_salary = self.extract_base_salary(&text)?;
        let mut gross_variable_salary = self.extract_variable_salary(&text)?;
        let imported_total_gross_salary = self.extract_total_gross_salary(&text)?;
        let total_gross = gross_fixed_salary + gross_variable_salary.values().sum::<Decimal>();

        if (imported_total_gross_salary - total_gross).abs() > Decimal::new(1, 0) {
            warn!(
                "Total gross salary ({}) doesn't match reported total gross ({})",
                total_gross, imported_total_gross_salary
            );
            gross_variable_salary.insert(
                "Total Gross Salary Discrepancy".to_string(),
                imported_total_gross_salary - total_gross,
            );
        }

        let total_social_contributions = self.extract_social_contributions_total(&text)?;
        let total_revenue_taxes = self.extract_revenue_taxes_total(&text)?;

        let additional_benefits = self.extract_additional_benefits(&text)?;
        let (meal_vouchers_employee_contribution, meal_vouchers_employer_contribution) =
            self.extract_meal_vouchers(&text)?;

        let net_paid_salary = self.extract_net_paid_salary(&text)?;

        info!("✅ Successfully extracted MathWorks payslip data:");
        debug!("   Pay date: {}", pay_date);
        debug!("   Base Salary: {}", gross_fixed_salary);
        debug!("   Variable Salary: {:?}", gross_variable_salary);
        info!("   Total Gross Salary: {}", imported_total_gross_salary);
        debug!(
            "   Total Social Contributions: {}",
            total_social_contributions
        );
        debug!("   Total Revenue Taxes: {}", total_revenue_taxes);
        debug!("   Additional Benefits: {:?}", additional_benefits);
        debug!(
            "   Meal Vouchers: Employee {} | Employer {}",
            meal_vouchers_employee_contribution, meal_vouchers_employer_contribution
        );
        info!("   Net Paid Salary: {}", net_paid_salary); // Sanity check - verify the calculation

        let calculated_net =
            imported_total_gross_salary - total_social_contributions - total_revenue_taxes
                + additional_benefits.values().sum::<Decimal>()
                - meal_vouchers_employee_contribution;

        if (calculated_net - net_paid_salary).abs() > Decimal::new(1, 0) {
            warn!(
                "Calculated net ({}) doesn't match reported net ({})",
                calculated_net, net_paid_salary
            );
        }

        Ok(ImportedPayslip {
            pay_date,
            employer_name: "The MathWorks".to_string(),
            gross_fixed_salary,
            gross_variable_salary,
            total_social_contributions,
            total_revenue_taxes,
            additional_benefits,
            meal_vouchers_employee_contribution,
            meal_vouchers_employer_contribution,
            net_paid_salary,
        })
    }
}

impl MathWorksPayslipImporter {
    /// Create a new MathWorks payslip importer
    pub fn new() -> Self {
        Self {}
    }

    /// Extract text from PDF using pdftotext with UTF-8 encoding
    fn extract_text_from_pdf(&self, file_path: &str) -> Result<String> {
        let output = Command::new("pdftotext.exe")
            .args(["-table", "-enc", "UTF-8", file_path, "-"])
            .output()
            .map_err(|e| CoreError::ImportError(format!("Failed to run pdftotext: {}", e)))?;

        if !output.status.success() {
            return Err(CoreError::ImportError(format!(
                "pdftotext failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        String::from_utf8(output.stdout)
            .map_err(|e| CoreError::ImportError(format!("Failed to decode PDF text: {}", e)))
    }

    /// Extract pay date from the text
    fn extract_pay_date(&self, text: &str) -> Result<NaiveDate> {
        let date_regex = Regex::new(r"(\d{2}/\d{2}/\d{4})").unwrap();

        // Look for "Date de paiement" followed by a date
        for line in text.lines() {
            if line.contains("Date de paiement") {
                // Extract date pattern dd/mm/yyyy
                if let Some(capture) = date_regex.captures(line) {
                    let date_str = capture.get(1).unwrap().as_str();
                    return NaiveDate::parse_from_str(date_str, "%d/%m/%Y").map_err(|_| {
                        CoreError::ImportError(format!("Failed to parse pay date from: '{}'", line))
                    });
                }
            }
        }

        Err(CoreError::ImportError(
            "Could not extract pay date from payslip".to_string(),
        ))
    }

    /// Parse French decimal format (handles spaces and commas)
    fn parse_french_decimal(&self, text: &str) -> Result<Decimal> {
        let cleaned = text
            .trim()
            .replace(" ", "") // Remove spaces (thousands separator)
            .replace(",", "."); // Replace comma with dot for decimal

        Decimal::from_str(&cleaned)
            .map_err(|_| CoreError::ImportError(format!("Failed to parse amount: {}", text)))
    }

    /// Extract base salary from payslip
    fn extract_base_salary(&self, text: &str) -> Result<Decimal> {
        let amount_regex = Regex::new(r"(\d{1,2}\s\d{3}[.,]\d{2})").unwrap();
        for line in text.lines() {
            let line = line.trim();
            if line.contains("Salaire de base") {
                // Look for amount patterns in the line
                if let Some(amount_match) = amount_regex.find(line) {
                    return self.parse_french_decimal(amount_match.as_str());
                }
            }
        }
        Err(CoreError::ImportError(
            "Could not extract base salary from payslip".to_string(),
        ))
    }

    /// Extract total gross salary from payslip
    fn extract_total_gross_salary(&self, text: &str) -> Result<Decimal> {
        let amount_regex = Regex::new(r"(\d{1,2}\s\d{3}[.,]\d{2})").unwrap();
        for line in text.lines() {
            let line = line.trim();
            if line.contains("Rémunération brute") {
                // Look for amount patterns in the line
                if let Some(amount_match) = amount_regex.find(line) {
                    return self.parse_french_decimal(amount_match.as_str());
                }
            }
        }
        Err(CoreError::ImportError(
            "Could not extract total gross salary from payslip".to_string(),
        ))
    }

    /// Extract variable salary components (bonuses, commissions, etc.)
    fn extract_variable_salary(&self, text: &str) -> Result<HashMap<String, Decimal>> {
        // TODO: refactor this to avoid code duplication
        let amount_regex1 = Regex::new(r"(\d{1,2}\s\d{3}[.,]\d{2})").unwrap();
        let amount_regex2 = Regex::new(r"(\d{1,3}[.,]\d{2})").unwrap();
        let mut result = HashMap::new(); // Look for Stakeholder bonus
        for line in text.lines() {
            let line = line.trim();
            if line.contains("Stakeholder") {
                // First try regex that handles thousands separators: "2 425,68"
                if let Some(amount_match) = amount_regex1.find(line) {
                    if let Ok(amount) = self.parse_french_decimal(amount_match.as_str()) {
                        result.insert("Stakeholder bonus".to_string(), amount);
                        continue;
                    }
                }
                // Fallback for amounts without thousands separator: "813,71"
                if let Some(amount_match) = amount_regex2.find(line) {
                    if let Ok(amount) = self.parse_french_decimal(amount_match.as_str()) {
                        result.insert("Stakeholder bonus".to_string(), amount);
                    }
                }
            }
            // Look for vacation prime
            if line.contains("Prime de vacances") {
                if let Some(amount_match) = amount_regex2.find(line) {
                    if let Ok(amount) = self.parse_french_decimal(amount_match.as_str()) {
                        result.insert("Prime de vacances".to_string(), amount);
                    }
                }
            }
            if line.contains("Indemnité compensatrice de Congés Payés") {
                // First try regex that handles thousands separators: "2 425,68"
                if let Some(amount_match) = amount_regex1.find(line) {
                    if let Ok(amount) = self.parse_french_decimal(amount_match.as_str()) {
                        result.insert(
                            "Indemnité compensatrice de Congés Payés".to_string(),
                            amount,
                        );
                        continue;
                    }
                }
                // Fallback for amounts without thousands separator: "813,71"
                if let Some(amount_match) = amount_regex2.find(line) {
                    if let Ok(amount) = self.parse_french_decimal(amount_match.as_str()) {
                        result.insert(
                            "Indemnité compensatrice de Congés Payés".to_string(),
                            amount,
                        );
                    }
                }
            }
            if line.contains("Indemnité compensatrice RTT") {
                // First try regex that handles thousands separators: "2 425,68"
                if let Some(amount_match) = amount_regex1.find(line) {
                    if let Ok(amount) = self.parse_french_decimal(amount_match.as_str()) {
                        result.insert("Indemnité compensatrice RTT".to_string(), amount);
                        continue;
                    }
                }
                // Fallback for amounts without thousands separator: "813,71"
                if let Some(amount_match) = amount_regex2.find(line) {
                    if let Ok(amount) = self.parse_french_decimal(amount_match.as_str()) {
                        result.insert("Indemnité compensatrice RTT".to_string(), amount);
                    }
                }
            }
        }

        Ok(result)
    }

    /// Extract total social contributions
    fn extract_social_contributions_total(&self, text: &str) -> Result<Decimal> {
        let amount_regex = Regex::new(r"(\d{1,2}\s\d{3}[.,]\d{2})").unwrap();
        for line in text.lines() {
            let line = line.trim();
            if line.contains("TOTAL COTISATIONS & CONTRIBUTIONS SALARIALES") {
                // Look for the amount at the end of the line
                if let Some(amount_match) = amount_regex.find(line) {
                    return self.parse_french_decimal(amount_match.as_str());
                }
            }
        }

        Err(CoreError::ImportError(
            "Could not find total social contributions".to_string(),
        ))
    }
    /// Extract revenue taxes (withholding tax)
    fn extract_revenue_taxes_total(&self, text: &str) -> Result<Decimal> {
        // Look for "Prélèvement à la source" in the calculation section with amount
        let amount_regex1 = Regex::new(r"(\d{1,2}\s\d{3}[.,]\d{2})").unwrap();
        let amount_regex2 = Regex::new(r"(\d{1,3}[.,]\d{2})").unwrap();
        for line in text.lines() {
            let line = line.trim();
            if line.contains("Prélèvement à la source") && line.contains("-") {
                // Look for amounts with thousands separators: "- 1 270,44"
                if let Some(amount_match) = amount_regex1.find(line) {
                    return self.parse_french_decimal(amount_match.as_str());
                }
                // Fallback for smaller amounts without thousands separator
                if let Some(amount_match) = amount_regex2.find(line) {
                    return self.parse_french_decimal(amount_match.as_str());
                }
            }
        }

        // Alternative: look for the detailed tax line at the bottom
        let amount_regex1 = Regex::new(r"(\d{1,2}\s\d{3}[.,]\d{2})").unwrap();
        let amount_regex2 = Regex::new(r"(\d{1,3}[.,]\d{2})").unwrap();
        for line in text.lines() {
            let line = line.trim();
            if line.contains("Impôt sur le revenu prélevé à la source") && !line.contains("%") {
                // Look for amounts with thousands separators
                if let Some(amount_match) = amount_regex1.find(line) {
                    return self.parse_french_decimal(amount_match.as_str());
                }
                // Fallback for smaller amounts
                if let Some(amount_match) = amount_regex2.find(line) {
                    return self.parse_french_decimal(amount_match.as_str());
                }
            }
        }

        Err(CoreError::ImportError(
            "Could not find withholding tax amount".to_string(),
        ))
    }

    /// Extract additional benefits (transport, telework allowance, etc.)
    fn extract_additional_benefits(&self, text: &str) -> Result<HashMap<String, Decimal>> {
        let mut result = HashMap::new();
        let amount_regex = Regex::new(r"(\d{1,3}[.,]\d{2})").unwrap();

        for line in text.lines() {
            let line = line.trim(); // Transport benefits - extract the benefit amount (last amount in the line)
            if line.contains("Frais transport public") {
                let amounts = self.extract_all_amounts_from_line(line);
                if let Some(&benefit_amount) = amounts.last() {
                    result.insert("Transport Allowance".to_string(), benefit_amount);
                }
            }

            // Telework allowance
            if line.contains("Frais de télétravail") {
                if let Some(amount_match) = amount_regex.find(line) {
                    if let Ok(amount) = self.parse_french_decimal(amount_match.as_str()) {
                        result.insert("Telework Allowance".to_string(), amount);
                    }
                }
            }
            // CSG participation
            if line.contains("Déduction CSG/CRDS participation placée") {
                let amounts = self.extract_all_amounts_from_line(line);
                if let Some(&benefit_amount) = amounts.last() {
                    result.insert("Déduction CSG Placée".to_string(), benefit_amount);
                }
            }
        }

        Ok(result)
    }

    /// Extract meal vouchers (employee and employer contributions)
    fn extract_meal_vouchers(&self, text: &str) -> Result<(Decimal, Decimal)> {
        let mut employee_contribution = Decimal::ZERO;
        let mut employer_contribution = Decimal::ZERO;

        let amount_regex = Regex::new(r"(\d{1,3}[.,]\d{2})").unwrap();
        for line in text.lines() {
            let line = line.trim();
            if line.contains("Titres-restaurant") {
                // Extract all amounts from the line
                let amounts: Vec<Decimal> = amount_regex
                    .find_iter(line)
                    .filter_map(|m| self.parse_french_decimal(m.as_str()).ok())
                    .collect();

                info!("🎫 Meal vouchers line: {}", line);
                info!("🎫 Found amounts: {:?}", amounts);

                // In MathWorks format, typically we see:
                // Titres-restaurant    19,00    2,9400    55,86    83,60
                // Where 55,86 is employee contribution and 83,60 is employer contribution
                if amounts.len() >= 2 {
                    employee_contribution = amounts[amounts.len() - 2]; // Second to last
                    employer_contribution = amounts[amounts.len() - 1]; // Last
                }
            }
        }

        Ok((employee_contribution, employer_contribution))
    }

    /// Extract net paid salary
    fn extract_net_paid_salary(&self, text: &str) -> Result<Decimal> {
        let amount_regex = Regex::new(r"(\d{1,2}\s\d{3}[.,]\d{2})").unwrap();
        for line in text.lines() {
            let line = line.trim();
            if line.contains("Net payé en euros") {
                // Look for the amount at the end of the line
                if let Some(amount_match) = amount_regex.find(line) {
                    return self.parse_french_decimal(amount_match.as_str());
                }
            }
        }

        Err(CoreError::ImportError(
            "Could not find net paid salary".to_string(),
        ))
    }

    /// Extract all amounts from a line using regex
    fn extract_all_amounts_from_line(&self, line: &str) -> Vec<Decimal> {
        let amount_regex = Regex::new(r"(\d{1,3}(?:\s\d{3})*[.,]\d{2})").unwrap();
        let mut amounts = Vec::new();

        for capture in amount_regex.find_iter(line) {
            if let Ok(amount) = self.parse_french_decimal(capture.as_str()) {
                amounts.push(amount);
            }
        }

        amounts
    }
}

impl Default for MathWorksPayslipImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::importers::PayslipImporter;

    fn init_test_logging() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    #[ignore = "Only run this test if you have the payslips available"]
    async fn test_mathworks_payslip_importer() {
        // TODO: is this test actually failing?
        init_test_logging();
        let importer = MathWorksPayslipImporter::new();

        let value = 1;
        let file_path = format!(
            "../../perso/MathWorks/2025/2025_{:02}_schoonbroodt_nicolas.pdf",
            value
        );
        if importer.can_handle_file(&file_path).unwrap() {
            let result = importer.import_from_file(&file_path).await;
            match result {
                Ok(payslip) => debug!("Payslip {}: {:#?}", value, payslip),
                Err(e) => debug!("Failed to import payslip {}: {}", value, e),
            }
        } else {
            debug!("Cannot handle file: {}", file_path);
        }
    }
}
