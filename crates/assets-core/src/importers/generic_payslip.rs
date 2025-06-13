use crate::error::Result;
use crate::importers::payslip_traits::{
    ImportedPayslip, PayslipImporter, PayslipItemType, PayslipLineItem,
};
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

/// Generic CSV payslip importer
/// 
/// Expected CSV format:
/// pay_date,pay_period_start,pay_period_end,employee_name,employer_name,item_type,description,amount,is_employer_contribution
/// 2025-01-31,2025-01-01,2025-01-31,John Doe,Acme Corp,BaseSalary,Base Salary,5000.00,false
/// 2025-01-31,2025-01-01,2025-01-31,John Doe,Acme Corp,IncomeTax,Federal Income Tax,-800.00,false
/// 2025-01-31,2025-01-01,2025-01-31,John Doe,Acme Corp,EmployerHealthInsurance,Health Insurance,300.00,true
/// 
/// Or you can customize this importer for your specific payslip format
pub struct GenericPayslipImporter;

#[async_trait]
impl PayslipImporter for GenericPayslipImporter {
    async fn import_from_file(&self, file_path: &str) -> Result<ImportedPayslip> {
        let content = tokio::fs::read_to_string(file_path).await?;
        self.parse_csv_content(&content)
    }

    fn format_description(&self) -> &'static str {
        "Generic CSV payslip format with columns: pay_date,pay_period_start,pay_period_end,employee_name,employer_name,item_type,description,amount,is_employer_contribution"
    }

    fn can_handle_file(&self, file_path: &str) -> Result<bool> {
        Ok(file_path.to_lowercase().ends_with(".csv"))
    }
}

impl GenericPayslipImporter {
    pub fn new() -> Self {
        Self
    }    fn parse_csv_content(&self, content: &str) -> Result<ImportedPayslip> {
        let mut lines = content.lines();
        
        // Skip header
        let _header = lines.next().ok_or_else(|| {
            crate::error::CoreError::ValidationError("Empty CSV file".to_string())
        })?;

        let mut line_items = Vec::new();
        let mut payslip_info: Option<(NaiveDate, NaiveDate, NaiveDate, String, String)> = None;
        let mut gross_total = Decimal::ZERO;
        let mut net_total = Decimal::ZERO;

        for (line_num, line) in lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() < 9 {
                return Err(crate::error::CoreError::ValidationError(format!(
                    "Invalid CSV format at line {}. Expected 9 fields, got {}",
                    line_num + 2,
                    fields.len()
                )));
            }

            // Parse dates and basic info (should be same for all lines in a payslip)
            let pay_date = NaiveDate::parse_from_str(fields[0], "%Y-%m-%d")
                .map_err(|e| crate::error::CoreError::ValidationError(format!("Invalid pay_date: {}", e)))?;
            let pay_period_start = NaiveDate::parse_from_str(fields[1], "%Y-%m-%d")
                .map_err(|e| crate::error::CoreError::ValidationError(format!("Invalid pay_period_start: {}", e)))?;
            let pay_period_end = NaiveDate::parse_from_str(fields[2], "%Y-%m-%d")
                .map_err(|e| crate::error::CoreError::ValidationError(format!("Invalid pay_period_end: {}", e)))?;
            let employee_name = fields[3].to_string();
            let employer_name = fields[4].to_string();

            // Store payslip info from first line
            if payslip_info.is_none() {
                payslip_info = Some((pay_date, pay_period_start, pay_period_end, employee_name.clone(), employer_name.clone()));
            }

            // Parse line item
            let item_type = self.parse_item_type(fields[5])?;
            let description = fields[6].to_string();
            let amount = Decimal::from_str(fields[7])
                .map_err(|e| crate::error::CoreError::ValidationError(format!("Invalid amount: {}", e)))?;
            let is_employer_contribution = fields[8].to_lowercase() == "true";

            // Calculate totals
            if item_type.affects_net_pay() {
                if amount > Decimal::ZERO {
                    gross_total += amount;
                    net_total += amount;
                } else {
                    net_total += amount; // Negative amount reduces net
                }
            }

            let mut raw_data = HashMap::new();
            for (i, field) in fields.iter().enumerate() {
                raw_data.insert(format!("field_{}", i), field.to_string());
            }

            line_items.push(PayslipLineItem {
                item_type,
                description,
                amount,
                is_employer_contribution,
                account_mapping: None, // Will use default mapping
                raw_data,
            });
        }

        let (pay_date, pay_period_start, pay_period_end, employee_name, employer_name) = 
            payslip_info.ok_or_else(|| {
                crate::error::CoreError::ValidationError("No valid payslip data found".to_string())
            })?;

        Ok(ImportedPayslip {
            pay_date,
            pay_period_start,
            pay_period_end,
            employee_name,
            employer_name,
            gross_salary: gross_total,
            net_salary: net_total,
            line_items,
            raw_data: HashMap::new(),
        })
    }

    fn parse_item_type(&self, type_str: &str) -> Result<PayslipItemType> {
        match type_str {
            "BaseSalary" => Ok(PayslipItemType::BaseSalary),
            "Overtime" => Ok(PayslipItemType::Overtime),
            "Bonus" => Ok(PayslipItemType::Bonus),
            "Commission" => Ok(PayslipItemType::Commission),
            "Allowance" => Ok(PayslipItemType::Allowance),
            "OtherIncome" => Ok(PayslipItemType::OtherIncome),
            "IncomeTax" => Ok(PayslipItemType::IncomeTax),
            "SocialSecurity" => Ok(PayslipItemType::SocialSecurity),
            "Medicare" => Ok(PayslipItemType::Medicare),
            "UnemploymentTax" => Ok(PayslipItemType::UnemploymentTax),
            "OtherTax" => Ok(PayslipItemType::OtherTax),
            "HealthInsurance" => Ok(PayslipItemType::HealthInsurance),
            "RetirementContribution" => Ok(PayslipItemType::RetirementContribution),
            "LifeInsurance" => Ok(PayslipItemType::LifeInsurance),
            "UnionDues" => Ok(PayslipItemType::UnionDues),
            "OtherDeduction" => Ok(PayslipItemType::OtherDeduction),
            "EmployerHealthInsurance" => Ok(PayslipItemType::EmployerHealthInsurance),
            "EmployerRetirement" => Ok(PayslipItemType::EmployerRetirement),
            "EmployerSocialSecurity" => Ok(PayslipItemType::EmployerSocialSecurity),
            "EmployerUnemployment" => Ok(PayslipItemType::EmployerUnemployment),
            "OtherEmployerContribution" => Ok(PayslipItemType::OtherEmployerContribution),
            _ => Err(crate::error::CoreError::ValidationError(format!(
                "Unknown payslip item type: {}",
                type_str
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_sample_payslip() {
        let csv_content = r#"pay_date,pay_period_start,pay_period_end,employee_name,employer_name,item_type,description,amount,is_employer_contribution
2025-01-31,2025-01-01,2025-01-31,John Doe,Acme Corp,BaseSalary,Base Salary,5000.00,false
2025-01-31,2025-01-01,2025-01-31,John Doe,Acme Corp,IncomeTax,Federal Income Tax,-800.00,false
2025-01-31,2025-01-01,2025-01-31,John Doe,Acme Corp,SocialSecurity,Social Security,-310.00,false
2025-01-31,2025-01-01,2025-01-31,John Doe,Acme Corp,HealthInsurance,Health Insurance,-150.00,false
2025-01-31,2025-01-01,2025-01-31,John Doe,Acme Corp,EmployerHealthInsurance,Employer Health Insurance,300.00,true"#;

        let importer = GenericPayslipImporter::new();
        let result = importer.parse_csv_content(csv_content).unwrap();

        assert_eq!(result.employee_name, "John Doe");
        assert_eq!(result.employer_name, "Acme Corp");
        assert_eq!(result.gross_salary, Decimal::from_str("5000.00").unwrap());
        assert_eq!(result.net_salary, Decimal::from_str("3740.00").unwrap()); // 5000 - 800 - 310 - 150
        assert_eq!(result.line_items.len(), 5);
    }
}
