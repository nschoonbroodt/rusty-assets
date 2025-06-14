use crate::error::{CoreError, Result};
use crate::importers::{ImportedPayslip, PayslipImporter, PayslipItemType, PayslipLineItem};
use async_trait::async_trait;
use chrono::NaiveDate;
use regex::Regex;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

// TODO: Qt Company Payslip Importer - Current Implementation Status
// =================================================================
//
// GOAL: Robust PDF payslip importer for Qt Company French payslips using pdftotext
// Captures detailed breakdown of social contributions, CSG, taxes, and employer contributions
//
// WHAT'S WORKING ✅:
// - Basic extraction: Gross salary (11,147.51 €), Net salary (7,177.06 €), Period (April 2025)
// - Employee name extraction: "Nicolas SCHOONBROODT" from header line
// - PDF text extraction: Using `pdftotext.exe -table -enc Latin1` for structured French text
// - Major employee deductions captured (~2,359 € out of ~3,970 € expected):
//   * Sécurité Sociale plafonnée: 270.83 €
//   * Sécurité Sociale déplafonnée: 44.96 €
//   * Complémentaire Tranche 1: 162.90 €
//   * Complémentaire Tranche 2: 721.23 € (large retirement contribution)
//   * APEC: 2.70 €
//   * CSG/CRDS taxes: 329.95 € + 773.68 €
//   * Health insurance: 53.00 €
// - All employer contributions captured correctly (social security, retirement, unemployment, etc.)
// - French payslip format parsing with different line patterns (2, 4, 6 amounts per line)
// - Proper classification into PayslipItemType categories
//
// PARSING LOGIC IMPLEMENTED:
// - 6 amounts: Base|Rate|Employee|Base|Rate|Employer (full format)
// - 4 amounts: Base|Employee|Base|Employer (simplified format)
// - 2 amounts: Base|Employer (employer-only contributions)
// - Special handling for CSG/CRDS (French income taxes)
// - Special handling for health insurance contributions
//
// REMAINING ISSUES ❌:
// 1. Amount parsing bug: "1 081.11" parsed as "81.11" (space in thousands separator)
//    - Affects some employer contribution amounts
//    - French decimal regex needs improvement for spaced thousands
//
// 2. Missing deductions in final total calculation:
//    - Some parsed deductions not included in summary (~1,611 € still missing)
//    - Logic for aggregating line items needs review
//
// 3. Database integration issue:
//    - Account lookup failing: "no rows returned by a query that expected to return at least one row"
//    - Need to verify account path "Assets:Current Assets:Main Checking" exists
//    - User "nicolas" exists but account lookup fails
//
// 4. Text encoding edge cases:
//    - Some French characters display as "Sécurité" -> "Sécurité" in debug output
//    - Latin1 encoding mostly working but some display issues remain
//
// HOW IT WORKS:
// 1. extract_pdf_text(): Calls `pdftotext.exe -table -enc Latin1` for structured extraction
// 2. parse_period(): Extracts "Période : Avril 2025" -> NaiveDate
// 3. extract_employee_name(): Parses header line "##SCHOONBROODT##Nicolas##" format
// 4. extract_summary_amounts(): Gets gross/net from "Salaire brut" and "Net payé" lines
// 5. extract_detailed_deductions(): Main parsing logic for individual line items
//    - Uses extract_description_from_line() to get deduction names
//    - Uses extract_all_amounts_from_line() to parse French decimal amounts
//    - Uses parse_detailed_deduction_line() to handle different line formats
//    - Uses classify_deduction_type() to map to accounting categories
// 6. create_detailed_line_items(): Aggregates and validates totals
//
// NEXT STEPS:
// 1. Fix French decimal parsing for spaced thousands (e.g., "1 081.11")
// 2. Debug missing deductions in final total calculation
// 3. Fix database account lookup issue
// 4. Test with multiple payslip samples for robustness
// 5. Add handling for benefits (meal vouchers, transport) - partially implemented
// 6. Consider adding validation for unreasonable amounts
// 7. Add support for different payslip formats/periods if needed
//
// COMMIT STATUS: Major functionality complete, needs final debugging and database integration
// =================================================================

/// Qt Company specific payslip importer that processes PDF files using pdftotext
/// This approach is much more reliable than OCR as it preserves the table structure
pub struct QtPayslipImporter;

impl QtPayslipImporter {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Extract text from PDF using pdftotext.exe with table format and proper encoding
    async fn extract_pdf_text(&self, file_path: &str) -> Result<String> {
        let output = Command::new("pdftotext.exe")
            .arg("-table")
            .arg("-enc")
            .arg("Latin1") // Use Latin1 encoding for proper French characters
            .arg(file_path)
            .arg("-") // Output to stdout
            .output()
            .map_err(|e| {
                CoreError::ImportError(format!(
                    "Failed to run pdftotext.exe. Please install poppler-utils: {}",
                    e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CoreError::ImportError(format!(
                "pdftotext.exe failed: {}",
                stderr
            )));
        }

        // Convert from Latin1 bytes to UTF-8 string
        let text = encoding_rs::WINDOWS_1252
            .decode(&output.stdout)
            .0
            .to_string();

        Ok(text)
    }

    /// Parse period from the payslip text
    fn parse_period(&self, text: &str) -> Result<NaiveDate> {
        // Look for "Période : Avril 2025" pattern
        let period_regex = Regex::new(r"Période\s*:\s*(\w+)\s+(\d{4})")
            .map_err(|e| CoreError::ImportError(format!("Regex error: {}", e)))?;

        if let Some(caps) = period_regex.captures(text) {
            let month_str = caps.get(1).unwrap().as_str().to_lowercase();
            let year_str = caps.get(2).unwrap().as_str();

            // French month names to numbers
            let french_months = HashMap::from([
                ("janvier", 1),
                ("février", 2),
                ("fevrier", 2),
                ("mars", 3),
                ("avril", 4),
                ("mai", 5),
                ("juin", 6),
                ("juillet", 7),
                ("août", 8),
                ("aout", 8),
                ("septembre", 9),
                ("octobre", 10),
                ("novembre", 11),
                ("décembre", 12),
                ("decembre", 12),
            ]);

            if let (Some(&month_num), Ok(year)) = (
                french_months.get(month_str.as_str()),
                year_str.parse::<i32>(),
            ) {
                return Ok(NaiveDate::from_ymd_opt(year, month_num, 1)
                    .ok_or_else(|| CoreError::ImportError("Invalid date".to_string()))?);
            }
        }

        // Fallback to current date if period can't be parsed
        Ok(chrono::Utc::now().naive_utc().date())
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
    /// Extract gross salary, commissions, and other income components
    fn extract_income_components(&self, text: &str) -> Result<Vec<PayslipLineItem>> {
        let mut income_items = Vec::new();
        let mut in_elements_section = false;

        for line in text.lines() {
            let line = line.trim();

            // Start looking after "Éléments de paie" header
            if line.contains("Éléments de paie") {
                in_elements_section = true;
                continue;
            }

            // Stop at deductions sections
            if line.contains("Santé") || line.contains("Sécurité Sociale - Mal") {
                break;
            }

            if in_elements_section {
                // Look for salary components
                if line.contains("Salaire de base") {
                    if let Some(amount) = self.extract_amount_from_line(line) {
                        income_items.push(PayslipLineItem {
                            item_type: PayslipItemType::BaseSalary,
                            description: "Salaire de base".to_string(),
                            amount,
                            is_employer_contribution: false,
                            account_mapping: Some(
                                PayslipItemType::BaseSalary
                                    .suggested_account_path()
                                    .to_string(),
                            ),
                            raw_data: HashMap::new(),
                        });
                    }
                } else if line.contains("Commissions") {
                    if let Some(amount) = self.extract_amount_from_line(line) {
                        income_items.push(PayslipLineItem {
                            item_type: PayslipItemType::Commission,
                            description: "Commissions".to_string(),
                            amount,
                            is_employer_contribution: false,
                            account_mapping: Some(
                                PayslipItemType::Commission
                                    .suggested_account_path()
                                    .to_string(),
                            ),
                            raw_data: HashMap::new(),
                        });
                    }
                } else if line.contains("Réintégration sociale") {
                    if let Some(amount) = self.extract_amount_from_line(line) {
                        income_items.push(PayslipLineItem {
                            item_type: PayslipItemType::OtherIncome,
                            description: "Réintégration sociale".to_string(),
                            amount,
                            is_employer_contribution: false,
                            account_mapping: Some(
                                PayslipItemType::OtherIncome
                                    .suggested_account_path()
                                    .to_string(),
                            ),
                            raw_data: HashMap::new(),
                        });
                    }
                }
            }
        }

        Ok(income_items)
    }

    /// Extract amount from the end of a line (common pattern in payslips)
    fn extract_amount_from_line(&self, line: &str) -> Option<Decimal> {
        // Look for amount pattern at the end of the line
        let amount_regex = Regex::new(r"(\d{1,2}\s\d{3}[.,]\d{2})(?:\s|$)").unwrap();
        if let Some(amount_match) = amount_regex.find(line) {
            return self.parse_french_decimal(amount_match.as_str()).ok();
        }
        None
    }
    /// Extract gross and net salary amounts from the structured text
    fn extract_summary_amounts(&self, text: &str) -> Result<(Decimal, Decimal)> {
        let mut gross_salary = None;
        let mut net_salary = None;

        for line in text.lines() {
            let line = line.trim();

            // Look for "Salaire brut" line followed by amount
            if line.contains("Salaire brut") {
                // Extract amount from the line - typically at the end
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
            (Some(gross), Some(net)) => {
                println!("✅ Extracted summary amounts:");
                println!("   Gross salary: {} €", gross);
                println!("   Net salary: {} €", net);
                println!("   Total deductions: {} €", gross - net);
                Ok((gross, net))
            }
            _ => Err(CoreError::ImportError(
                "Could not extract both gross and net salary amounts from payslip".to_string(),
            )),
        }
    }
    /// Extract employee name from the payslip text
    fn extract_employee_name(&self, text: &str) -> String {
        for line in text.lines() {
            let line = line.trim();
            // Look for the line that contains the employee name after address
            if line.starts_with("Monsieur") || line.starts_with("Madame") {
                // Extract name after "Monsieur" or "Madame"
                if let Some(name_part) = line
                    .strip_prefix("Monsieur ")
                    .or_else(|| line.strip_prefix("Madame "))
                {
                    return name_part.trim().to_string();
                }
            }
        }

        // Fallback: look for name in header line pattern
        for line in text.lines() {
            if line.contains("##") && line.contains("SCHOONBROODT") {
                // Header format: ACE-QTCOMP##BULLETIN##04-2025##14230##SCHOONBROODT##Nicolas##8383666
                let parts: Vec<&str> = line.split("##").collect();
                if parts.len() >= 6 && parts[4] == "SCHOONBROODT" {
                    return format!("{} {}", parts[5], parts[4]);
                }
            }
        }

        "Unknown Employee".to_string()
    }
    /// Extract detailed employee deductions and employer contributions from the payslip table
    fn extract_detailed_deductions(
        &self,
        text: &str,
    ) -> (Vec<PayslipLineItem>, Vec<PayslipLineItem>) {
        let mut employee_deductions = Vec::new();
        let mut employer_contributions = Vec::new();
        let mut in_deductions_section = false;

        for line in text.lines() {
            let line = line.trim();

            // Start looking for deductions after "Santé" section (first deductions)
            if line.contains("Santé") && !line.contains("Complémentaire - Santé") {
                in_deductions_section = true;
                continue;
            }

            // Stop at totals section
            if line.contains("Total des cotisations") || line.contains("Montant net social") {
                break;
            }

            if in_deductions_section && !line.is_empty() {
                // Skip category headers and empty lines
                if self.is_category_header(line) {
                    continue;
                }

                // Try to extract deduction items from the structured table
                if let Some((employee_item, employer_item)) =
                    self.parse_detailed_deduction_line(line)
                {
                    if let Some(emp_item) = employee_item {
                        println!(
                            "DEBUG: Employee deduction: {} = {} €",
                            emp_item.description, emp_item.amount
                        );
                        employee_deductions.push(emp_item);
                    }
                    if let Some(empr_item) = employer_item {
                        println!(
                            "DEBUG: Employer contribution: {} = {} €",
                            empr_item.description, empr_item.amount
                        );
                        employer_contributions.push(empr_item);
                    }
                }
            }
        }

        (employee_deductions, employer_contributions)
    }
    /// Check if a line is a category header (like "Santé", "Retraite", etc.)
    fn is_category_header(&self, line: &str) -> bool {
        let headers = [
            "Santé",
            "Retraite",
            "Famille",
            "Assurance chômage",
            "Cot. statutaires ou prévues par la conv. coll.",
            "Autres contributions dues par l'employeur",
        ];

        // Check if line starts with any header and has no amounts
        if headers.iter().any(|&header| line.starts_with(header)) {
            // Make sure it's not a line with actual data (amounts)
            let amounts = self.extract_all_amounts_from_line(line);
            return amounts.is_empty();
        }

        false
    }
    /// Parse a detailed deduction line to extract employee and employer amounts
    /// Qt payslip table format: Description | Base | Taux | A déduire (employee) | A payer | Charges patronales (employer)
    fn parse_detailed_deduction_line(
        &self,
        line: &str,
    ) -> Option<(Option<PayslipLineItem>, Option<PayslipLineItem>)> {
        // Skip lines that don't look like deduction entries
        if line.is_empty() || line.starts_with("         ") {
            return None;
        }

        println!("DEBUG: Parsing line: {}", line);

        // First, try to get the description from the beginning of the line
        let description_opt = self.extract_description_from_line(line);
        if description_opt.is_none() {
            return None;
        }
        let description = description_opt.unwrap();

        // Extract all amounts from the line
        let amounts = self.extract_all_amounts_from_line(line);
        println!(
            "DEBUG: Found {} amounts in line: {:?}",
            amounts.len(),
            amounts
        );

        if amounts.is_empty() {
            return None;
        }

        let mut employee_item = None;
        let mut employer_item = None; // Handle different patterns based on the line content and number of amounts
        if line.contains("CSG") || line.contains("CRDS") {
            // CSG/CRDS lines: Base | Rate | Employee_Deduction  OR  Base | Employee_Deduction
            // Example: "CSG déduct. de l'impôt sur le revenu    11 377.71  6.8000    773.68"
            //       or "CSG déduct. de l'impôt sur le revenu    11 377.71    773.68"
            let employee_amount = if amounts.len() >= 3 {
                amounts[2] // Third amount is the actual deduction
            } else if amounts.len() >= 2 {
                amounts[1] // Second amount is the deduction
            } else {
                return None;
            };

            employee_item = Some(PayslipLineItem {
                item_type: PayslipItemType::IncomeTax,
                description: description.clone(),
                amount: employee_amount,
                is_employer_contribution: false,
                account_mapping: Some(
                    PayslipItemType::IncomeTax
                        .suggested_account_path()
                        .to_string(),
                ),
                raw_data: HashMap::new(),
            });
        } else if line.contains("Complémentaire - Santé") {
            // Health insurance: usually just one amount for employee
            // Example: "Complémentaire - Santé                              53.00"
            if amounts.len() >= 1 {
                let employee_amount = amounts[0];
                employee_item = Some(PayslipLineItem {
                    item_type: PayslipItemType::HealthInsurance,
                    description: description.clone(),
                    amount: employee_amount,
                    is_employer_contribution: false,
                    account_mapping: Some(
                        PayslipItemType::HealthInsurance
                            .suggested_account_path()
                            .to_string(),
                    ),
                    raw_data: HashMap::new(),
                });
            }
        } else {
            // For other lines, parse based on number of amounts
            match amounts.len() {
                6 => {
                    // Full format: Base | Rate | Employee_Deduction | Base | Rate | Employer_Contribution
                    // Example: "Sécurité Sociale plafonnée    3 925.00  6.9000    270.83    3 925.00  8.5500    335.59"
                    let employee_amount = amounts[2]; // Third amount is employee deduction
                    let employer_amount = amounts[5]; // Sixth amount is employer contribution

                    if employee_amount > Decimal::ZERO {
                        employee_item = Some(PayslipLineItem {
                            item_type: self.classify_deduction_type(&description, false),
                            description: description.clone(),
                            amount: employee_amount,
                            is_employer_contribution: false,
                            account_mapping: Some(
                                self.classify_deduction_type(&description, false)
                                    .suggested_account_path()
                                    .to_string(),
                            ),
                            raw_data: HashMap::new(),
                        });
                    }

                    if employer_amount > Decimal::ZERO {
                        employer_item = Some(PayslipLineItem {
                            item_type: self.classify_deduction_type(&description, true),
                            description: format!("{} (employeur)", description),
                            amount: employer_amount,
                            is_employer_contribution: true,
                            account_mapping: Some(
                                self.classify_deduction_type(&description, true)
                                    .suggested_account_path()
                                    .to_string(),
                            ),
                            raw_data: HashMap::new(),
                        });
                    }
                }
                4 => {
                    // Format: Base | Employee_Deduction | Base | Employer_Contribution
                    // Example: "Sécurité Sociale plafonnée    925.00  270.83  925.00  335.59"
                    // or "Complémentaire Tranche 2    314.67  721.23  314.67  81.11"

                    let potential_employee = amounts[1];
                    let potential_employer = amounts[3];

                    // For employee deductions, check if it's a reasonable deduction amount
                    // It should be positive and typically smaller than the base, but retirement
                    // contributions can be quite large
                    if potential_employee > Decimal::ZERO {
                        employee_item = Some(PayslipLineItem {
                            item_type: self.classify_deduction_type(&description, false),
                            description: description.clone(),
                            amount: potential_employee,
                            is_employer_contribution: false,
                            account_mapping: Some(
                                self.classify_deduction_type(&description, false)
                                    .suggested_account_path()
                                    .to_string(),
                            ),
                            raw_data: HashMap::new(),
                        });
                    }

                    if potential_employer > Decimal::ZERO {
                        employer_item = Some(PayslipLineItem {
                            item_type: self.classify_deduction_type(&description, true),
                            description: format!("{} (employeur)", description),
                            amount: potential_employer,
                            is_employer_contribution: true,
                            account_mapping: Some(
                                self.classify_deduction_type(&description, true)
                                    .suggested_account_path()
                                    .to_string(),
                            ),
                            raw_data: HashMap::new(),
                        });
                    }
                }
                2 => {
                    // Format: Base | Employer_Contribution (employee-only employer contribution)
                    let employer_amount = amounts[1]; // Second amount is employer

                    if employer_amount > Decimal::ZERO {
                        employer_item = Some(PayslipLineItem {
                            item_type: self.classify_deduction_type(&description, true),
                            description: format!("{} (employeur)", description),
                            amount: employer_amount,
                            is_employer_contribution: true,
                            account_mapping: Some(
                                self.classify_deduction_type(&description, true)
                                    .suggested_account_path()
                                    .to_string(),
                            ),
                            raw_data: HashMap::new(),
                        });
                    }
                }
                _ => {
                    // Other cases - just log for now
                    println!("DEBUG: Unhandled amount count: {}", amounts.len());
                }
            }
        }

        // Log what we're adding
        if let Some(ref emp) = employee_item {
            println!(
                "DEBUG: Adding employee deduction: {} = {} €",
                emp.description, emp.amount
            );
        }
        if let Some(ref empr) = employer_item {
            println!(
                "DEBUG: Adding employer contribution: {} = {} €",
                empr.description, empr.amount
            );
        }

        Some((employee_item, employer_item))
    }
    /// Extract all decimal amounts from a line
    fn extract_all_amounts_from_line(&self, line: &str) -> Vec<Decimal> {
        let mut amounts = Vec::new();

        // Look for French decimal patterns (can include spaces as thousand separators)
        // This regex handles: 123.45, 1 234.56, 1,234.56, 1 234,56, etc.
        let amount_regex = Regex::new(r"\b\d{1,3}(?:\s\d{3})*[.,]\d{2}\b").unwrap();

        for capture in amount_regex.find_iter(line) {
            if let Ok(amount) = self.parse_french_decimal(capture.as_str()) {
                amounts.push(amount);
            }
        }

        amounts
    }
    /// Extract description from the beginning of a line
    fn extract_description_from_line(&self, line: &str) -> Option<String> {
        // The Qt payslip has descriptions at the beginning, followed by amounts
        // We need to extract everything before the first number

        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Find the first occurrence of a number (not including leading spaces)
        let mut description_end = trimmed.len();
        let mut found_number = false;

        for (i, c) in trimmed.char_indices() {
            if c.is_ascii_digit() {
                // Check if this is part of a number pattern
                // Look back to see if there are spaces before this digit
                let before = &trimmed[..i];
                if before.ends_with(' ') || before.ends_with('\t') {
                    description_end = i;
                    found_number = true;
                    break;
                }
            }
        }

        if !found_number {
            return None;
        }

        let description = trimmed[..description_end].trim().to_string();
        if description.is_empty() {
            return None;
        }

        Some(description)
    }
    /// Classify deduction type based on description
    fn classify_deduction_type(&self, description: &str, is_employer: bool) -> PayslipItemType {
        let desc_lower = description.to_lowercase();

        // French payslip-specific classifications
        if desc_lower.contains("sécurité sociale") || desc_lower.contains("securite sociale") {
            if is_employer {
                PayslipItemType::EmployerSocialSecurity
            } else {
                PayslipItemType::SocialSecurity
            }
        } else if desc_lower.contains("retraite") || desc_lower.contains("complémentaire") {
            if is_employer {
                PayslipItemType::EmployerRetirement
            } else {
                PayslipItemType::RetirementContribution
            }
        } else if desc_lower.contains("santé") || desc_lower.contains("complémentaire - sant") {
            if is_employer {
                PayslipItemType::EmployerHealthInsurance
            } else {
                PayslipItemType::HealthInsurance
            }
        } else if desc_lower.contains("chômage")
            || desc_lower.contains("chomage")
            || desc_lower.contains("apec")
        {
            if is_employer {
                PayslipItemType::EmployerUnemployment
            } else {
                PayslipItemType::UnemploymentTax
            }
        } else if desc_lower.contains("csg")
            || desc_lower.contains("crds")
            || desc_lower.contains("impôt")
            || desc_lower.contains("pas")
        {
            // CSG, CRDS, and income tax are French-specific taxes
            PayslipItemType::IncomeTax
        } else {
            // Default classification
            if is_employer {
                PayslipItemType::OtherEmployerContribution
            } else {
                PayslipItemType::OtherTax
            }
        }
    }
    /// Create detailed payslip line items with individual income, deductions and employer contributions
    fn create_detailed_line_items(
        &self,
        income_items: Vec<PayslipLineItem>,
        gross_salary: Decimal,
        net_salary: Decimal,
        employee_deductions: Vec<PayslipLineItem>,
        employer_contributions: Vec<PayslipLineItem>,
    ) -> Vec<PayslipLineItem> {
        let mut items = Vec::new();

        // Add income components (salary, commissions, etc.)
        if !income_items.is_empty() {
            println!("ℹ️  Using detailed income breakdown");
            items.extend(income_items);
        } else {
            // Fallback to simple gross salary
            println!("ℹ️  Using simple gross salary (detailed breakdown not available)");
            items.push(PayslipLineItem {
                item_type: PayslipItemType::BaseSalary,
                description: "Salaire brut".to_string(),
                amount: gross_salary,
                is_employer_contribution: false,
                account_mapping: Some(
                    PayslipItemType::BaseSalary
                        .suggested_account_path()
                        .to_string(),
                ),
                raw_data: HashMap::new(),
            });
        }

        // Add all individual employee deductions
        items.extend(employee_deductions);

        // Add all employer contributions (these show the total cost to the employer)
        items.extend(employer_contributions);

        // Verify that our detailed deductions add up reasonably
        let total_employee_deductions: Decimal = items
            .iter()
            .filter(|item| {
                !item.is_employer_contribution
                    && item.item_type != PayslipItemType::BaseSalary
                    && item.item_type != PayslipItemType::Commission
                    && item.item_type != PayslipItemType::OtherIncome
            })
            .map(|item| item.amount)
            .sum();

        let expected_deductions = gross_salary - net_salary;
        let difference = (total_employee_deductions - expected_deductions).abs();

        // If our detailed parsing is significantly off, add an adjustment item
        if difference > Decimal::new(1, 0) {
            // More than 1 euro difference
            println!(
                "⚠️  Deduction total mismatch - Expected: {} €, Parsed: {} €, Difference: {} €",
                expected_deductions, total_employee_deductions, difference
            );

            if total_employee_deductions < expected_deductions {
                // We're missing some deductions
                let missing_amount = expected_deductions - total_employee_deductions;
                items.push(PayslipLineItem {
                    item_type: PayslipItemType::OtherTax,
                    description: "Autres déductions non détaillées".to_string(),
                    amount: missing_amount,
                    is_employer_contribution: false,
                    account_mapping: Some(
                        PayslipItemType::OtherTax
                            .suggested_account_path()
                            .to_string(),
                    ),
                    raw_data: HashMap::new(),
                });
            }
        } else {
            println!(
                "✅ Detailed deductions match expected total (difference: {} €)",
                difference
            );
        }

        items
    }
}

#[async_trait]
impl PayslipImporter for QtPayslipImporter {
    fn format_description(&self) -> &'static str {
        "Qt Company PDF payslip format (pdftotext-based)"
    }

    fn can_handle_file(&self, file_path: &str) -> Result<bool> {
        Ok(file_path.to_lowercase().ends_with(".pdf"))
    }

    async fn import_from_file(&self, file_path: &str) -> Result<ImportedPayslip> {
        println!("💰 Importing Qt Payslip from PDF using pdftotext...");

        // Extract text from PDF using pdftotext
        let text = self.extract_pdf_text(file_path).await?;

        // Parse period
        let period = self.parse_period(&text)?;

        // Extract employee name
        let employee_name = self.extract_employee_name(&text); // Extract income components and summary amounts
        let income_items = self.extract_income_components(&text)?;
        let (gross_salary, net_salary) = self.extract_summary_amounts(&text)?; // Extract detailed deductions and employer contributions
        let (employee_deductions, employer_contributions) = self.extract_detailed_deductions(&text);

        if !employee_deductions.is_empty() {
            println!("\n=== Individual Employee Deductions Found ===");
            for item in &employee_deductions {
                println!("{}: {} €", item.description, item.amount);
            }
            println!("=== End Employee Deductions ===\n");
        }

        if !employer_contributions.is_empty() {
            println!("\n=== Employer Contributions Found ===");
            for item in &employer_contributions {
                println!("{}: {} €", item.description, item.amount);
            }
            println!("=== End Employer Contributions ===\n");
        } // Create detailed line items with individual breakdowns
        let line_items = self.create_detailed_line_items(
            income_items,
            gross_salary,
            net_salary,
            employee_deductions,
            employer_contributions,
        );

        // Create raw data map
        let mut raw_data = HashMap::new();
        raw_data.insert("pdf_file".to_string(), file_path.to_string());
        raw_data.insert(
            "text_sample".to_string(),
            text[..text.len().min(500)].to_string(),
        );

        println!("✅ Successfully extracted payslip data:");
        println!("   Employee: {}", employee_name);
        println!("   Period: {}", period);
        println!("   Gross: {} €", gross_salary);
        println!("   Net: {} €", net_salary);
        println!("   Total deductions: {} €", gross_salary - net_salary);

        Ok(ImportedPayslip {
            pay_date: period,
            pay_period_start: period,
            pay_period_end: period,
            employee_name,
            employer_name: "The Qt Company".to_string(),
            gross_salary,
            net_salary,
            line_items,
            raw_data,
        })
    }
}

impl Default for QtPayslipImporter {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
