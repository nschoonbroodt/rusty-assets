use super::*;
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[test]
fn test_sg_record_parsing() {
    let record = SocietegeneraleCsvRecord {
        date_operation: "27/05/2025".to_string(),
        libelle: "000001 VIR EUROPEE".to_string(),
        detail: "000001 VIR EUROPEEN EMIS   LOGITEL POUR: M. NICOLAS SCHOONBROODT 27 05 SG 00485 CPT 00030470377 REF: 9514775507581".to_string(),
        montant: "-1000,00".to_string(),
        devise: "EUR".to_string(),
    };

    let transaction = record.into_imported_transaction().unwrap();

    assert_eq!(
        transaction.date_op,
        NaiveDate::from_ymd_opt(2025, 5, 27).unwrap()
    );
    assert_eq!(transaction.amount, Decimal::from_str("-1000.00").unwrap());
    assert_eq!(transaction.category, Some("Transfers:Wire".to_string()));
}

#[test]
fn test_sg_categorization() {
    assert_eq!(
        categorize_sg_transaction("000001 VIR EUROPEE", "VIR EUROPEEN EMIS"),
        "Transfers:Wire"
    );
    assert_eq!(
        categorize_sg_transaction("VIR INST RE 564770", "VIR INST"),
        "Transfers:Internal"
    );
    assert_eq!(
        categorize_sg_transaction("VIR RECU 951356780", "VIR RECU DE: M. NICOLAS"),
        "Income:Transfers"
    );
}
