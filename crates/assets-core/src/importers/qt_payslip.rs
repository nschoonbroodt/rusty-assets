use crate::error::{CoreError, Result};
use crate::importers::{ImportedPayslip, PayslipImporter};
use async_trait::async_trait;
use chrono::NaiveDate;
use regex::Regex;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

/// Qt Company Payslip Importer
pub struct QtPayslipImporter {}

#[async_trait]
impl PayslipImporter for QtPayslipImporter {
    fn format_description(&self) -> &'static str {
        "Qt Company PDF payslip format"
    }

    fn can_handle_file(&self, file_path: &str) -> Result<bool> {
        Ok(file_path.to_lowercase().ends_with(".pdf"))
    }

    async fn import_from_file(&self, file_path: &str) -> Result<ImportedPayslip> {
        println!("💰 Importing Qt Payslip from PDF...");

        let text = self.extract_text_from_pdf(file_path)?;
        let pay_date = self.extract_period(&text)?;

        let (total_gross, net_paid_salary) = self.extract_summary_amounts(&text)?;
        let gross_fixed_salary = self.extract_base_salary(&text)?;
        let gross_variable_salary = self.extract_variable(&text)?;

        let total_social_contributions = self.extract_social_contributions_total(&text)?;
        let total_revenue_taxes = self.extract_revenue_taxes_total(&text)?;

        let additional_benefits = self.extract_additional_benefits(&text)?;
        let (meal_vouchers_employee_contribution, meal_vouchers_employer_contribution) =
            self.extract_tickets_restaurant(&text)?;

        println!("✅ Successfully extracted simplified payslip data:");
        println!("   Pay day: {}", pay_date);
        println!("   Total Gross Salary: {}", total_gross);
        println!("   Fixed Gross Salary: {}", gross_fixed_salary);
        println!("   Variable Gross Salary: {:?}", gross_variable_salary);

        println!(
            "   Total Social Contributions: {}",
            total_social_contributions
        );
        println!("   Total Revenue Taxes: {}", total_revenue_taxes);

        println!("   Additional benefits: {:?}", additional_benefits);
        println!(
            "   Meal Vouchers Contribution: Employee {}\tEmployer {}",
            meal_vouchers_employee_contribution, meal_vouchers_employer_contribution
        );

        println!("   Net Paid Salary: {}", net_paid_salary);

        // TODO: sanity check of amounts
        assert_eq!(
            total_gross,
            gross_fixed_salary
                + gross_variable_salary
                    .iter()
                    .map(|(_, v)| *v)
                    .sum::<Decimal>(),
            "Total gross salary should match fixed + variable"
        );
        assert_eq!(
            net_paid_salary,
            total_gross
                - total_social_contributions
                - total_revenue_taxes
                - meal_vouchers_employee_contribution
                + additional_benefits.iter().map(|(_, v)| *v).sum::<Decimal>(),
            "Net salary should match gross - contributions - taxes + benefits"
        );

        Ok(ImportedPayslip {
            pay_date: pay_date,
            employer_name: "The Qt Company".to_string(),
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

impl QtPayslipImporter {
    /// Create a new Qt payslip importer
    pub fn new() -> Self {
        Self {}
    }
    /// Extract text from PDF using pdftotext with Latin1 encoding
    fn extract_text_from_pdf(&self, file_path: &str) -> Result<String> {
        let output = Command::new("pdftotext.exe")
            .args(["-table", "-enc", "Latin1", file_path, "-"])
            .output()
            .map_err(|e| CoreError::ImportError(format!("Failed to run pdftotext: {}", e)))?;

        if !output.status.success() {
            return Err(CoreError::ImportError(format!(
                "pdftotext failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Handle Latin1 encoding properly
        let latin1_bytes = output.stdout;
        let text = latin1_bytes
            .iter()
            .map(|&byte| byte as char)
            .collect::<String>();

        Ok(text)
    }

    /// Extract payslip period from the text
    fn extract_period(&self, text: &str) -> Result<NaiveDate> {
        // Look for patterns like "Paiement le"
        for line in text.lines() {
            if line.contains("Paiement le") {
                // Extract the date from the line
                let date_regex = Regex::new(r"(\d{1,2}/\d{1,2}/\d{4})").unwrap();
                if let Some(capture) = date_regex.captures(line) {
                    let date =
                        NaiveDate::parse_from_str(capture.get(1).unwrap().as_str(), "%d/%m/%Y")
                            .map_err(|_| {
                                CoreError::ImportError(format!(
                                    "Failed to parse date from line: '{}'",
                                    line
                                ))
                            })?;
                    return Ok(date);
                }
            }
        }
        // Fallback to current date if period can't be parsed
        Err(CoreError::ImportError(
            "Could not extract payslip period".to_string(),
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

    /// Extract gross and net salary amounts from the structured text
    fn extract_summary_amounts(&self, text: &str) -> Result<(Decimal, Decimal)> {
        let mut gross_salary = None;
        let mut net_salary = None;

        for line in text.lines() {
            let line = line.trim();

            // Look for "Salaire brut" line followed by amount
            if line.contains("Salaire brut") {
                if let Some(amount_match) = Regex::new(r"(\d{1,2}\s\d{3}[.,]\d{2})(?:\s|$)")
                    .unwrap()
                    .find(line)
                {
                    if let Ok(amount) = self.parse_french_decimal(amount_match.as_str()) {
                        gross_salary = Some(amount);
                    }
                }
            }

            // Look for "Net payé" line
            if line.contains("Net payé") {
                if let Some(amount_match) = Regex::new(r"(\d{1,2}\s\d{3}[.,]\d{2})(?:\s|$)")
                    .unwrap()
                    .find(line)
                {
                    if let Ok(amount) = self.parse_french_decimal(amount_match.as_str()) {
                        net_salary = Some(amount);
                    }
                }
            }
        }

        match (gross_salary, net_salary) {
            (Some(gross), Some(net)) => Ok((gross, net)),
            _ => Err(CoreError::ImportError(
                "Could not extract gross and net salary amounts".to_string(),
            )),
        }
    }
    /// Extract amount from a line (looks for the last amount pattern)
    fn extract_amount_from_line(&self, line: &str) -> Option<Decimal> {
        // Try different patterns for amounts
        let patterns = [
            r"(\d{1,2}\s\d{3}[.,]\d{2})", // "12 345,67" or "12 345.67"
            r"(\d{1,3}[.,]\d{2})",        // "96.80" or "96,80"
            r"(\d{4,}[.,]\d{2})",         // "1234.56"
        ];

        for pattern in &patterns {
            let amount_regex = Regex::new(pattern).unwrap();
            if let Some(amount_match) = amount_regex.find(line) {
                if let Ok(amount) = self.parse_french_decimal(amount_match.as_str()) {
                    return Some(amount);
                }
            }
        }
        None
    }

    /// Extract base salary from payslip
    fn extract_base_salary(&self, text: &str) -> Result<Decimal> {
        for line in text.lines() {
            let line = line.trim();
            if line.contains("Salaire de base") {
                if let Some(amount) = self.extract_amount_from_line(line) {
                    return Ok(amount);
                }
            }
        }
        Err(CoreError::ImportError(
            "Could not extract base salary from payslip".to_string(),
        ))
    }

    /// Extract gross variable from payslip
    fn extract_variable(&self, text: &str) -> Result<HashMap<String, Decimal>> {
        let mut result = HashMap::<String, Decimal>::new();
        for line in text.lines() {
            let line = line.trim();
            if line.contains("Commissions") {
                if let Some(amount) = self.extract_amount_from_line(line) {
                    result.insert("Commissions".to_string(), amount);
                }
            }
        }
        Ok(result) // Return zero if no commission found
    }
    fn extract_tickets_restaurant(&self, text: &str) -> Result<(Decimal, Decimal)> {
        let mut employee_contribution = Decimal::ZERO;
        let mut employer_contribution = Decimal::ZERO;

        for line in text.lines() {
            let line = line.trim();

            // Look for "Titres-restaurant" or "Tickets restaurant" line
            if line.contains("Titres-restaurant") || line.contains("Tickets restaurant") {
                // Extract all amounts from this line
                let amounts = self.extract_all_amounts_from_line(line);
                println!("🎫 Titres-restaurant line: {}", line);
                println!("🎫 Found amounts: {:?}", amounts);

                // In the table format, amounts are structured:
                // Base, Taux, A déduire (employee), employer part, etc.
                // The employee deduction should be one of the larger amounts
                if let Some(&amount) = amounts.iter().find(|&&a| a > Decimal::new(50, 0)) {
                    employee_contribution = amount;
                    employer_contribution = employee_contribution
                        .checked_mul(Decimal::from_f64(7.26 / 4.84).unwrap())
                        .unwrap();
                    println!("🎫 Found tickets restaurant employee: {} €", amount);
                }

                // Look for employer amount (typically in the rightmost columns)
                if let Some(&emp_amount) = amounts.last() {
                    if emp_amount > Decimal::new(100, 0) {
                        employer_contribution = emp_amount;
                        println!("🎫 Found tickets restaurant employer: {} €", emp_amount);
                    }
                }
            }
        }

        Ok((employee_contribution, employer_contribution))
    }
    fn extract_additional_benefits(&self, text: &str) -> Result<HashMap<String, Decimal>> {
        let mut result = HashMap::<String, Decimal>::new();
        for line in text.lines() {
            let line = line.trim();
            if line.contains("Forfait mensuel NAVIGO") {
                if let Some(amount) = self.extract_amount_from_line(line) {
                    result.insert("Forfait mensuel NAVIGO".to_string(), amount);
                }
            }
        }
        Ok(result) // Return zero if no commission found
    }
    /// Extract total social contributions (from "Total des cotisations et contributions" line)
    fn extract_social_contributions_total(&self, text: &str) -> Result<Decimal> {
        for line in text.lines() {
            let line = line.trim();

            // Look for the total contributions line
            if line.contains("Total des cotisations et contributions") {
                // Extract the amount from this line
                if let Some(amount) = self.extract_amount_from_line(line) {
                    return Ok(amount);
                }
            }
        }

        Err(CoreError::ImportError(
            "Could not find 'Total des cotisations et contributions' line".to_string(),
        ))
    }
    fn extract_revenue_taxes_total(&self, text: &str) -> Result<Decimal> {
        for line in text.lines() {
            let line = line.trim();

            if line.contains("sur le revenu prélevé") {
                // Extract the amount from this line
                if let Some(amount) = self.extract_amount_from_line(line) {
                    return Ok(amount);
                }
            }
        }

        Err(CoreError::ImportError(
            "Could not find 'Impot sur le revenu' line".to_string(),
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

impl Default for QtPayslipImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::importers::PayslipImporter;

    #[tokio::test]
    #[ignore = "Only run this test if you have the payslips available"]
    async fn test_qt_payslip_importer_february() {
        let importer = QtPayslipImporter::new();
        let result = importer
            .import_from_file("../../perso/Qt/2025/Bulletins 02_2025.pdf")
            .await
            .unwrap();
        println!("{:#?}", result);
    }
    #[tokio::test]
    #[ignore = "Only run this test if you have the payslips available"]
    async fn test_qt_payslip_importer_march() {
        let importer = QtPayslipImporter::new();
        let result = importer
            .import_from_file("../../perso/Qt/2025/Bulletins 03_2025.pdf")
            .await
            .unwrap();
        println!("{:#?}", result);
    }
    #[tokio::test]
    #[ignore = "Only run this test if you have the payslips available"]
    async fn test_qt_payslip_importer_april() {
        let importer = QtPayslipImporter::new();
        let result = importer
            .import_from_file("../../perso/Qt/2025/Bulletins 04_2025.pdf")
            .await
            .unwrap();
        println!("{:#?}", result);
    }
    #[tokio::test]
    #[ignore = "Only run this test if you have the payslips available"]
    async fn test_qt_payslip_importer_may() {
        let importer = QtPayslipImporter::new();
        let result = importer
            .import_from_file("../../perso/Qt/2025/Bulletins 05_2025.pdf")
            .await
            .unwrap();
        println!("{:#?}", result);
    }
}
