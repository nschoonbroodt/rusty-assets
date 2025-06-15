pub mod boursobank;
pub mod mathworks_payslip;
pub mod payslip_traits;
pub mod qt_payslip;
pub mod societegenerale;
pub mod traits;

pub use boursobank::BoursoBankImporter;
pub use mathworks_payslip::MathWorksPayslipImporter;
pub use payslip_traits::{ImportedPayslip, PayslipImporter};
pub use qt_payslip::QtPayslipImporter;
pub use societegenerale::SocietegeneraleImporter;
pub use traits::{ImportedTransaction, TransactionImporter};
