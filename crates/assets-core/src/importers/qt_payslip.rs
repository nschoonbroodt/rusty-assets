use crate::error::{CoreError, Result};
use crate::importers::{ImportedPayslip, PayslipImporter, PayslipItemType, PayslipLineItem};
use async_trait::async_trait;
use chrono::NaiveDate;
use regex::Regex;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

/// Qt Company Payslip Importer - Simplified Version
/// Extracts only the essential 8 components as requested by the user
pub struct QtPayslipImporter {
    // Add configuration options later if needed
}

#[async_trait]
impl PayslipImporter for QtPayslipImporter {
    fn format_description(&self) -> &'static str {
        "Qt Company PDF payslip format (simplified, 8 key components)"
    }

    fn can_handle_file(&self, file_path: &str) -> Result<bool> {
        Ok(file_path.to_lowercase().ends_with(".pdf"))
    }

    async fn import_from_file(&self, file_path: &str) -> Result<ImportedPayslip> {
        println!("💰 Importing Qt Payslip from PDF (simplified extraction)...");

        let text = self.extract_text_from_pdf(file_path)?;

        // Extract basic information
        let employee_name = self.extract_employee_name(&text);
        let period = self.extract_period(&text)?;
        let (gross_salary, net_salary) = self.extract_summary_amounts(&text)?;

        // Extract the 8 key components only
        let base_salary = self.extract_base_salary(&text)?;
        let commission = self.extract_commission(&text)?;
        let social_contributions = self.extract_social_contributions_total(&text)?;
        let income_tax = self.extract_income_tax(&text)?;
        let (ticket_employee, ticket_employer) = self.extract_tickets_restaurant(&text)?;
        let navigo_reimbursement = self.extract_navigo_reimbursement(&text)?;

        // Calculate amounts for verification
        let income_tax_amount = income_tax
            .as_ref()
            .map(|item| item.amount)
            .unwrap_or(Decimal::ZERO);
        let ticket_employee_amount = ticket_employee
            .as_ref()
            .map(|item| item.amount)
            .unwrap_or(Decimal::ZERO);
        let navigo_amount = navigo_reimbursement
            .as_ref()
            .map(|item| item.amount)
            .unwrap_or(Decimal::ZERO);

        // Calculate expected net pay: Base + Commission - Social - Tax - TR_Employee + Navigo
        let expected_net = base_salary + commission
            - social_contributions
            - income_tax_amount
            - ticket_employee_amount
            + navigo_amount;

        println!("\n=== SIMPLIFIED PAYSLIP BREAKDOWN ===");
        println!("Base salary: {} €", base_salary);
        println!("Commissions: {} €", commission);
        println!("Social contributions: {} €", social_contributions);
        println!("Income tax (PAS): {} €", income_tax_amount);
        println!(
            "Tickets restaurant (employee): {} €",
            ticket_employee_amount
        );
        if let Some(ref ticket_emp) = ticket_employer {
            println!("Tickets restaurant (employer): {} €", ticket_emp.amount);
        }
        println!("Navigo reimbursement: {} €", navigo_amount);
        println!("Expected net pay: {} €", expected_net);
        println!("Actual net pay: {} €", net_salary);
        println!("Difference: {} €", (expected_net - net_salary).abs());
        println!("=== END BREAKDOWN ===\n");

        // Create simplified line items
        let mut line_items = Vec::new();

        // 1. Base salary
        line_items.push(PayslipLineItem {
            item_type: PayslipItemType::BaseSalary,
            description: "Base salary".to_string(),
            amount: base_salary,
            is_employer_contribution: false,
            account_mapping: Some("Income:Salary:Base".to_string()),
            raw_data: HashMap::new(),
        });

        // 2. Commissions
        line_items.push(PayslipLineItem {
            item_type: PayslipItemType::Commission,
            description: "Commissions".to_string(),
            amount: commission,
            is_employer_contribution: false,
            account_mapping: Some("Income:Salary:Variable".to_string()),
            raw_data: HashMap::new(),
        });

        // 3. Social contributions
        line_items.push(PayslipLineItem {
            item_type: PayslipItemType::SocialSecurity,
            description: "Social contributions".to_string(),
            amount: social_contributions,
            is_employer_contribution: false,
            account_mapping: Some("Expenses:Taxes:SocialContributions".to_string()),
            raw_data: HashMap::new(),
        });

        // 4. Income tax
        if let Some(tax) = income_tax {
            line_items.push(PayslipLineItem {
                item_type: PayslipItemType::IncomeTax,
                description: "Income tax (PAS)".to_string(),
                amount: tax.amount,
                is_employer_contribution: false,
                account_mapping: Some("Expenses:Taxes:IR".to_string()),
                raw_data: HashMap::new(),
            });
        }

        // 5. Employee contribution to meal vouchers
        if let Some(ticket_emp) = ticket_employee {
            line_items.push(PayslipLineItem {
                item_type: PayslipItemType::MealVouchers,
                description: "Meal vouchers (employee)".to_string(),
                amount: ticket_emp.amount,
                is_employer_contribution: false,
                account_mapping: Some("Assets:Meal Voucher".to_string()),
                raw_data: HashMap::new(),
            });
        }

        // 6. Employer contribution to meal vouchers (dual account)
        if let Some(ticket_empr) = ticket_employer {
            // Asset side
            line_items.push(PayslipLineItem {
                item_type: PayslipItemType::MealVouchers,
                description: "Meal vouchers (employer) - Asset".to_string(),
                amount: ticket_empr.amount,
                is_employer_contribution: true,
                account_mapping: Some("Assets:Meal Voucher".to_string()),
                raw_data: HashMap::new(),
            });
            // Income side
            line_items.push(PayslipLineItem {
                item_type: PayslipItemType::MealVouchers,
                description: "Meal vouchers (employer) - Benefit".to_string(),
                amount: ticket_empr.amount,
                is_employer_contribution: true,
                account_mapping: Some("Income:Benefits:Meal Voucher".to_string()),
                raw_data: HashMap::new(),
            });
        }

        // 7. Navigo transport reimbursement
        if let Some(navigo) = navigo_reimbursement {            line_items.push(PayslipLineItem {
                item_type: PayslipItemType::TransportReimbursement,
                description: "Transport reimbursement (Navigo)".to_string(),
                amount: navigo.amount,
                is_employer_contribution: false,
                account_mapping: Some("Income:Benefits:Transports".to_string()),
                raw_data: HashMap::new(),
            });        }

        // 8. Net à payer (configurable account - using default for now)
        line_items.push(PayslipLineItem {
            item_type: PayslipItemType::NetPay,
            description: "Net à payer".to_string(),
            amount: net_salary,
            is_employer_contribution: false,
            account_mapping: None, // Will be set by the service using the destination account
            raw_data: HashMap::new(),
        });

        // Create raw data map
        let mut raw_data = HashMap::new();
        raw_data.insert("pdf_file".to_string(), file_path.to_string());
        raw_data.insert(
            "text_sample".to_string(),
            text[..text.len().min(500)].to_string(),
        );
        raw_data.insert("base_salary".to_string(), base_salary.to_string());
        raw_data.insert("commission".to_string(), commission.to_string());
        raw_data.insert(
            "social_contributions".to_string(),
            social_contributions.to_string(),
        );
        raw_data.insert("income_tax".to_string(), income_tax_amount.to_string());
        raw_data.insert(
            "ticket_employee".to_string(),
            ticket_employee_amount.to_string(),
        );
        raw_data.insert(
            "navigo_reimbursement".to_string(),
            navigo_amount.to_string(),
        );
        raw_data.insert("expected_net".to_string(), expected_net.to_string());

        println!("✅ Successfully extracted simplified payslip data:");
        println!("   Employee: {}", employee_name);
        println!("   Period: {}", period);
        println!("   Gross: {} €", gross_salary);
        println!("   Net: {} €", net_salary);

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

    /// Extract employee name from the payslip text
    fn extract_employee_name(&self, _text: &str) -> String {
        // For simplicity, return a default name - could be enhanced later
        "Unknown Employee".to_string()
    }

    /// Extract payslip period from the text
    fn extract_period(&self, text: &str) -> Result<NaiveDate> {
        // Look for patterns like "Avril 2025" or "April 2025"
        for line in text.lines() {
            if let Some(captures) = Regex::new(r"(?i)(janvier|février|mars|avril|mai|juin|juillet|août|septembre|octobre|novembre|décembre|january|february|march|april|may|june|july|august|september|october|november|december)\s+(\d{4})")
                .unwrap()
                .captures(line) {
                let month_str = captures.get(1).unwrap().as_str().to_lowercase();
                let year_str = captures.get(2).unwrap().as_str();

                let french_months = HashMap::from([
                    ("janvier", 1), ("février", 2), ("mars", 3), ("avril", 4),
                    ("mai", 5), ("juin", 6), ("juillet", 7), ("août", 8),
                    ("septembre", 9), ("octobre", 10), ("novembre", 11), ("décembre", 12),
                    ("january", 1), ("february", 2), ("march", 3), ("april", 4),
                    ("may", 5), ("june", 6), ("july", 7), ("august", 8),
                    ("september", 9), ("october", 10), ("november", 11), ("december", 12),
                ]);

                if let (Some(&month_num), Ok(year)) = (
                    french_months.get(month_str.as_str()),
                    year_str.parse::<i32>(),
                ) {
                    return Ok(NaiveDate::from_ymd_opt(year, month_num, 1)
                        .ok_or_else(|| CoreError::ImportError("Invalid date".to_string()))?);
                }
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

    /// Extract commission from payslip
    fn extract_commission(&self, text: &str) -> Result<Decimal> {
        for line in text.lines() {
            let line = line.trim();
            if line.contains("Commissions") {
                if let Some(amount) = self.extract_amount_from_line(line) {
                    return Ok(amount);
                }
            }
        }
        Ok(Decimal::ZERO) // Return zero if no commission found
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

    /// Extract income tax (PAS - Prélèvement à la Source)
    fn extract_income_tax(&self, text: &str) -> Result<Option<PayslipLineItem>> {
        for line in text.lines() {
            let line = line.trim();

            // Handle encoding variations: "prélevé" might appear as "prΘlevΘ" or similar
            if line.contains("Imp⌠t sur le revenu pr")
                || line.contains("Impôt sur le revenu pr")
                || line.contains("Impot sur le revenu pr")
            {
                let amounts = self.extract_all_amounts_from_line(line);
                if let Some(&amount) = amounts.last() {
                    return Ok(Some(PayslipLineItem {
                        item_type: PayslipItemType::IncomeTax,
                        description: "Income tax (PAS)".to_string(),
                        amount,
                        is_employer_contribution: false,
                        account_mapping: Some("Expenses:Taxes:IR".to_string()),
                        raw_data: HashMap::new(),
                    }));
                }
            }
        }
        Ok(None)
    }
    /// Extract tickets restaurant (meal vouchers) employee and employer contributions
    fn extract_tickets_restaurant(
        &self,
        text: &str,
    ) -> Result<(Option<PayslipLineItem>, Option<PayslipLineItem>)> {
        let mut employee_amount = None;
        let mut employer_amount = None;

        for line in text.lines() {
            let line = line.trim();

            // Look for "Titres-restaurant" line (employee contribution)
            if line.contains("Titres-restaurant") {
                // Extract all amounts from this line
                let amounts = self.extract_all_amounts_from_line(line);
                println!("🎫 Titres-restaurant line: {}", line);
                println!("🎫 Found amounts: {:?}", amounts);

                // In the table format, amounts are structured:
                // Base, Taux, A déduire (employee), employer part, etc.
                // The employee deduction should be one of the larger amounts
                if let Some(&amount) = amounts.iter().find(|&&a| a > Decimal::new(50, 0)) {
                    employee_amount = Some(amount);
                    println!("🎫 Found tickets restaurant employee: {} €", amount);
                }

                // Look for employer amount (typically in the rightmost columns)
                if let Some(&emp_amount) = amounts.last() {
                    if emp_amount > Decimal::new(100, 0) {
                        employer_amount = Some(emp_amount);
                        println!("🎫 Found tickets restaurant employer: {} €", emp_amount);
                    }
                }
            }
        }

        let employee_item = if let Some(amount) = employee_amount {
            Some(PayslipLineItem {
                item_type: PayslipItemType::MealVouchers,
                description: "Meal vouchers (employee)".to_string(),
                amount,
                is_employer_contribution: false,
                account_mapping: Some("Assets:Meal Voucher".to_string()),
                raw_data: HashMap::new(),
            })
        } else {
            None
        };

        let employer_item = if let Some(amount) = employer_amount {
            Some(PayslipLineItem {
                item_type: PayslipItemType::MealVouchers,
                description: "Meal vouchers (employer)".to_string(),
                amount,
                is_employer_contribution: true,
                account_mapping: Some("Assets:Meal Voucher".to_string()),
                raw_data: HashMap::new(),
            })
        } else {
            None
        };

        Ok((employee_item, employer_item))
    }
    /// Extract Navigo transport reimbursement
    fn extract_navigo_reimbursement(&self, text: &str) -> Result<Option<PayslipLineItem>> {
        for line in text.lines() {
            let line = line.trim();

            // Look for "Forfait mensuel NAVIGO" line
            if line.contains("Forfait mensuel NAVIGO") {
                // Extract all amounts from this line
                let amounts = self.extract_all_amounts_from_line(line);
                println!("🚇 Navigo line: {}", line);
                println!("🚇 Found amounts: {:?}", amounts);

                if !amounts.is_empty() {
                    // In the table format, the Navigo reimbursement amount is typically
                    // the first significant amount (should be around 88.80)
                    let amount = amounts[0];
                    println!("🚇 Found Navigo reimbursement: {} €", amount);                    return Ok(Some(PayslipLineItem {
                        item_type: PayslipItemType::TransportReimbursement,
                        description: "Transport reimbursement (Navigo)".to_string(),
                        amount,
                        is_employer_contribution: false,
                        account_mapping: Some("Income:Benefits:Transports".to_string()),
                        raw_data: HashMap::new(),
                    }));
                }
            }
        }

        // If not found, return None (no Navigo reimbursement this month)
        Ok(None)
    }
}

impl Default for QtPayslipImporter {
    fn default() -> Self {
        Self::new()
    }
}
