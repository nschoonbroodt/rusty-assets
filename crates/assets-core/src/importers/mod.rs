pub mod boursobank;
pub mod generic_payslip;
pub mod payslip_traits;
pub mod societegenerale;
pub mod traits;

pub use boursobank::BoursoBankImporter;
pub use generic_payslip::GenericPayslipImporter;
pub use payslip_traits::{ImportedPayslip, PayslipImporter, PayslipItemType, PayslipLineItem};
pub use societegenerale::SocietegeneraleImporter;
pub use traits::{ImportedTransaction, TransactionImporter};
