use super::*;
use crate::importers::PayslipImporter;

fn init_test_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[tokio::test]
#[ignore = "Only run this test if you have the payslips available"]
async fn test_qt_payslip_importer_february() {
    init_test_logging();
    let importer = QtPayslipImporter::new();
    let result = importer
        .import_from_file("../../perso/Qt/2025/Bulletins 02_2025.pdf")
        .await
        .unwrap();
    debug!("{:#?}", result);
}
#[tokio::test]
#[ignore = "Only run this test if you have the payslips available"]
async fn test_qt_payslip_importer_march() {
    init_test_logging();
    let importer = QtPayslipImporter::new();
    let result = importer
        .import_from_file("../../perso/Qt/2025/Bulletins 03_2025.pdf")
        .await
        .unwrap();
    debug!("{:#?}", result);
}
#[tokio::test]
#[ignore = "Only run this test if you have the payslips available"]
async fn test_qt_payslip_importer_april() {
    init_test_logging();
    let importer = QtPayslipImporter::new();
    let result = importer
        .import_from_file("../../perso/Qt/2025/Bulletins 04_2025.pdf")
        .await
        .unwrap();
    debug!("{:#?}", result);
}
#[tokio::test]
#[ignore = "Only run this test if you have the payslips available"]
async fn test_qt_payslip_importer_may() {
    init_test_logging();
    let importer = QtPayslipImporter::new();
    let result = importer
        .import_from_file("../../perso/Qt/2025/Bulletins 05_2025.pdf")
        .await
        .unwrap();

    debug!("{:#?}", result);
}
