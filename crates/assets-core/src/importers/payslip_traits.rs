use crate::error::Result;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[async_trait]
pub trait PayslipImporter {
    /// Import payslip data from a file path
    async fn import_from_file(&self, file_path: &str) -> Result<ImportedPayslip>;

    /// Get the expected file format description
    fn format_description(&self) -> &'static str;

    /// Validate if this importer can handle the given file
    fn can_handle_file(&self, file_path: &str) -> Result<bool>;
}

/// Represents payslip with the following data: (this is French oriented ;-) )
/// - pay date and period
/// - employee and employer names
/// - gross fixed and variable salary (anything irregular goes to variable: Commission, Bonus, Vacation, etc.)
/// - Social Contributions to be deducted
/// - Revenue taxes to be deducted
/// - additional benefits (untaxed) such as transport reimbursement,
/// - meal vouchers: deduced part from the employee and part paid by the employer
/// - Net salary to be paid
#[derive(Debug, Clone)]
pub struct ImportedPayslip {
    pub pay_date: NaiveDate,
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub employee_name: String,
    pub employer_name: String,
    pub gross_fixed_salary: Decimal,
    pub gross_variable_salary: Vec<(String, Decimal)>, // e.g. vec![(String::from("Bonus"), Decimal::new(5000, 2))],
    pub total_social_contributions: Decimal,
    pub total_revenue_taxes: Decimal,
    pub additional_benefits: Vec<(String, Decimal)>, // e.g. vec![(String::from("Transport Reimbursement"), Decimal::new(2000, 2))],
    pub meal_vouchers_employee_contribution: Decimal, // Employee's part of meal vouchers
    pub meal_vouchers_employer_contribution: Decimal, // Employer's part of meal vouchers
    pub net_paid_salary: Decimal,
}
