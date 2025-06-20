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
