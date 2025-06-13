pub mod boursobank;
pub mod societegenerale;
pub mod traits;

pub use boursobank::BoursoBankImporter;
pub use societegenerale::SocietegeneraleImporter;
pub use traits::{ImportedTransaction, TransactionImporter};
