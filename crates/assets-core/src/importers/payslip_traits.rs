use crate::error::Result;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

#[async_trait]
pub trait PayslipImporter {
    /// Import payslip data from a file path
    async fn import_from_file(&self, file_path: &str) -> Result<ImportedPayslip>;

    /// Get the expected file format description
    fn format_description(&self) -> &'static str;

    /// Validate if this importer can handle the given file
    fn can_handle_file(&self, file_path: &str) -> Result<bool>;
}

/// Represents a complete payslip with all line items
#[derive(Debug, Clone)]
pub struct ImportedPayslip {
    pub pay_date: NaiveDate,
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub employee_name: String,
    pub employer_name: String,
    pub gross_salary: Decimal,
    pub net_salary: Decimal,
    pub line_items: Vec<PayslipLineItem>,
    pub raw_data: HashMap<String, String>,
}

/// Individual line item on a payslip (salary, tax, deduction, benefit, etc.)
#[derive(Debug, Clone)]
pub struct PayslipLineItem {
    pub item_type: PayslipItemType,
    pub description: String,
    pub amount: Decimal,
    pub is_employer_contribution: bool, // For benefits paid by employer
    pub account_mapping: Option<String>, // Suggested account path
    pub raw_data: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PayslipItemType {    // Income items (credits to the employee)
    BaseSalary,
    Overtime,
    Bonus,
    Commission,
    Allowance,
    TransportReimbursement,
    OtherIncome,
    
    // Tax deductions (debits from employee)
    IncomeTax,
    SocialSecurity,
    Medicare,
    UnemploymentTax,
    OtherTax,
      // Benefit deductions (debits from employee for benefits)
    HealthInsurance,
    RetirementContribution,
    LifeInsurance,
    UnionDues,
    MealVouchers,
    OtherDeduction,
    
    // Employer contributions (not affecting employee net pay)
    EmployerHealthInsurance,
    EmployerRetirement,
    EmployerSocialSecurity,
    EmployerUnemployment,
    OtherEmployerContribution,
    
    // Net pay (final amount paid to employee)
    NetPay,
}

impl PayslipItemType {
    /// Get the suggested account path for this item type
    pub fn suggested_account_path(&self) -> &'static str {
        match self {
            // Income accounts
            PayslipItemType::BaseSalary => "Income:Salary:Base Salary",
            PayslipItemType::Overtime => "Income:Salary:Overtime",
            PayslipItemType::Bonus => "Income:Salary:Bonus",            PayslipItemType::Commission => "Income:Salary:Commission",
            PayslipItemType::Allowance => "Income:Salary:Allowance",
            PayslipItemType::TransportReimbursement => "Income:Benefits:Transport",
            PayslipItemType::OtherIncome => "Income:Salary:Other",
            
            // Tax expense accounts
            PayslipItemType::IncomeTax => "Expenses:Taxes:Income Tax",
            PayslipItemType::SocialSecurity => "Expenses:Taxes:Social Security",
            PayslipItemType::Medicare => "Expenses:Taxes:Medicare",
            PayslipItemType::UnemploymentTax => "Expenses:Taxes:Unemployment",
            PayslipItemType::OtherTax => "Expenses:Taxes:Other",
            
            // Benefit expense accounts
            PayslipItemType::HealthInsurance => "Expenses:Benefits:Health Insurance",            PayslipItemType::RetirementContribution => "Expenses:Benefits:Retirement",            PayslipItemType::LifeInsurance => "Expenses:Benefits:Life Insurance",
            PayslipItemType::UnionDues => "Expenses:Benefits:Union Dues",
            PayslipItemType::MealVouchers => "Expenses:Benefits:Meal Vouchers",
            PayslipItemType::OtherDeduction => "Expenses:Benefits:Other",
            
            // Employer contribution accounts (these are "income" to show total compensation)
            PayslipItemType::EmployerHealthInsurance => "Income:Benefits:Health Insurance",
            PayslipItemType::EmployerRetirement => "Income:Benefits:Retirement",
            PayslipItemType::EmployerSocialSecurity => "Income:Benefits:Social Security",
            PayslipItemType::EmployerUnemployment => "Income:Benefits:Unemployment",
            PayslipItemType::OtherEmployerContribution => "Income:Benefits:Other",
            
            // Net pay - configurable account
            PayslipItemType::NetPay => "Assets:Current Assets:Checking",
        }
    }
      /// Check if this item affects the employee's net pay
    pub fn affects_net_pay(&self) -> bool {
        !matches!(self, 
            PayslipItemType::EmployerHealthInsurance |
            PayslipItemType::EmployerRetirement |
            PayslipItemType::EmployerSocialSecurity |
            PayslipItemType::EmployerUnemployment |
            PayslipItemType::OtherEmployerContribution |
            PayslipItemType::NetPay // NetPay is the result, not a contributor
        )
    }
}
