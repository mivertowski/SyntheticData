//! FEC (Fichier des Écritures Comptables) export for French GAAP.
//!
//! Exports journal entries in the mandatory 18-column format required by
//! Article A47 A-1 of the Livre des Procédures Fiscales (LPF).
//! See: https://www.cegid.com/fr/glossaire/glossaire-fec-comptable/

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use datasynth_core::error::SynthResult;
use datasynth_core::models::{ChartOfAccounts, JournalEntry};

/// FEC 18 mandatory columns (Article A47 A-1 LPF), in order.
const FEC_HEADER: &str = "Code journal;Libellé journal;Numéro de l'écriture;Date de comptabilisation;Numéro de compte;Libellé de compte;Numéro de compte auxiliaire;Libellé de compte auxiliaire;Référence de la pièce justificative;Date d'émission de la pièce justificative;Libellé de l'écriture comptable;Montant au débit;Montant au crédit;Lettrage;Date de lettrage;Date de validation de l'écriture;Montant en devise;Identifiant de la devise";

fn escape_fec_field(s: &str) -> String {
    let t = s.replace(';', ",").replace(['\n', '\r'], " ");
    if t.contains('"') {
        format!("\"{}\"", t.replace('"', "\"\""))
    } else {
        t
    }
}

fn format_decimal(d: rust_decimal::Decimal) -> String {
    format!("{:.2}", d)
}

/// Write journal entries to a FEC-compliant CSV file (semicolon-separated, UTF-8).
///
/// One row per journal line. Columns 1–18 follow the official order.
/// Uses `coa` for "Libellé de compte" (column 6). Blank fields for
/// auxiliaire, lettrage, and optional devise when not used.
pub fn write_fec_csv(
    path: &Path,
    entries: &[JournalEntry],
    coa: &ChartOfAccounts,
) -> SynthResult<()> {
    let file = File::create(path)?;
    let mut w = BufWriter::with_capacity(256 * 1024, file);

    writeln!(w, "{}", FEC_HEADER)?;

    let mut ecriture_num: u64 = 1;
    for je in entries {
        let code_journal = escape_fec_field(je.header.document_type.as_str());
        let libelle_journal = je
            .header
            .header_text
            .as_deref()
            .unwrap_or(je.header.document_type.as_str());
        let libelle_journal = escape_fec_field(libelle_journal);
        let date_compta = je.header.posting_date.format("%Y%m%d").to_string();
        let ref_piece = je.header.reference.as_deref().unwrap_or("").to_string();
        let ref_piece = escape_fec_field(&ref_piece);
        let date_piece = je.header.document_date.format("%Y%m%d").to_string();
        let date_validation = je.header.posting_date.format("%Y%m%d").to_string();
        let currency = escape_fec_field(je.header.currency.as_str());

        for line in &je.lines {
            let libelle_compte = coa
                .get_account(&line.gl_account)
                .map(|a| a.short_description.as_str())
                .unwrap_or(line.gl_account.as_str());
            let libelle_compte = escape_fec_field(libelle_compte);
            let libelle_ecriture = line
                .line_text
                .as_deref()
                .or(je.header.header_text.as_deref())
                .unwrap_or("")
                .to_string();
            let libelle_ecriture = escape_fec_field(&libelle_ecriture);

            let debit = format_decimal(line.debit_amount);
            let credit = format_decimal(line.credit_amount);
            let montant_devise = if line.debit_amount > rust_decimal::Decimal::ZERO {
                format_decimal(line.debit_amount)
            } else {
                format_decimal(line.credit_amount)
            };

            // Column 7: Numéro de compte auxiliaire
            let aux_num = line.auxiliary_account_number.as_deref().unwrap_or("");
            // Column 8: Libellé de compte auxiliaire
            let aux_label = line
                .auxiliary_account_label
                .as_deref()
                .map(escape_fec_field)
                .unwrap_or_default();
            // Column 14: Lettrage
            let lettrage = line.lettrage.as_deref().unwrap_or("");
            // Column 15: Date de lettrage
            let lettrage_date = line
                .lettrage_date
                .map(|d| d.format("%Y%m%d").to_string())
                .unwrap_or_default();

            writeln!(
                w,
                "{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{}",
                code_journal,
                libelle_journal,
                ecriture_num,
                date_compta,
                escape_fec_field(&line.gl_account),
                libelle_compte,
                escape_fec_field(aux_num),
                aux_label,
                ref_piece,
                date_piece,
                libelle_ecriture,
                debit,
                credit,
                lettrage,
                lettrage_date,
                date_validation,
                montant_devise,
                currency,
            )?;
        }
        ecriture_num += 1;
    }

    w.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_core::models::{
        AccountSubType, AccountType, CoAComplexity, GLAccount, IndustrySector, JournalEntryHeader,
        JournalEntryLine,
    };
    use rust_decimal_macros::dec;

    #[test]
    fn test_fec_header_has_18_columns() {
        let cols: Vec<&str> = FEC_HEADER.split(';').collect();
        assert_eq!(cols.len(), 18, "FEC must have 18 columns");
    }

    #[test]
    fn test_fec_data_row_round_trip() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        // Build a minimal CoA with one account
        let mut coa = ChartOfAccounts::new(
            "TEST".to_string(),
            "Test CoA".to_string(),
            "FR".to_string(),
            IndustrySector::Manufacturing,
            CoAComplexity::Small,
        );
        coa.add_account(GLAccount::new(
            "411000".to_string(),
            "Clients".to_string(),
            AccountType::Asset,
            AccountSubType::AccountsReceivable,
        ));
        coa.add_account(GLAccount::new(
            "701000".to_string(),
            "Ventes".to_string(),
            AccountType::Revenue,
            AccountSubType::ProductRevenue,
        ));

        // Build a journal entry with two lines
        let mut header = JournalEntryHeader::new("C001".to_string(), date);
        header.currency = "EUR".to_string();
        header.header_text = Some("Test sale".to_string());
        header.reference = Some("REF001".to_string());
        let mut je = JournalEntry::new(header);
        je.add_line(JournalEntryLine::debit(
            je.header.document_id,
            1,
            "411000".to_string(),
            dec!(1000.50),
        ));
        je.add_line(JournalEntryLine::credit(
            je.header.document_id,
            2,
            "701000".to_string(),
            dec!(1000.50),
        ));

        // Write to temp file and read back
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fec.csv");
        write_fec_csv(&path, &[je], &coa).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Header + 2 data rows
        assert_eq!(lines.len(), 3, "expected header + 2 data rows");

        // Every data row must have exactly 18 semicolon-separated columns
        for (i, line) in lines.iter().enumerate() {
            let cols: Vec<&str> = line.split(';').collect();
            assert_eq!(
                cols.len(),
                18,
                "row {} has {} columns, expected 18",
                i,
                cols.len()
            );
        }

        // Verify debit/credit amounts in data rows (columns 12 and 13, 1-indexed)
        let row1_cols: Vec<&str> = lines[1].split(';').collect();
        assert_eq!(row1_cols[11], "1000.50", "debit amount");
        assert_eq!(row1_cols[12], "0.00", "credit amount on debit line");

        let row2_cols: Vec<&str> = lines[2].split(';').collect();
        assert_eq!(row2_cols[11], "0.00", "debit amount on credit line");
        assert_eq!(row2_cols[12], "1000.50", "credit amount");
    }

    #[test]
    fn test_fec_auxiliary_and_lettrage_columns() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        let mut coa = ChartOfAccounts::new(
            "TEST".to_string(),
            "Test CoA".to_string(),
            "FR".to_string(),
            IndustrySector::Manufacturing,
            CoAComplexity::Small,
        );
        coa.add_account(GLAccount::new(
            "411000".to_string(),
            "Clients".to_string(),
            AccountType::Asset,
            AccountSubType::AccountsReceivable,
        ));
        coa.add_account(GLAccount::new(
            "701000".to_string(),
            "Ventes".to_string(),
            AccountType::Revenue,
            AccountSubType::ProductRevenue,
        ));

        // Build a JE where the AR line has auxiliary + lettrage fields set
        let mut header = JournalEntryHeader::new("C001".to_string(), date);
        header.currency = "EUR".to_string();
        header.header_text = Some("Test sale".to_string());
        header.reference = Some("REF002".to_string());
        let mut je = JournalEntry::new(header);

        // AR line with auxiliary and lettrage
        let mut ar_line = JournalEntryLine::debit(
            je.header.document_id,
            1,
            "411000".to_string(),
            dec!(2000.00),
        );
        ar_line.auxiliary_account_number = Some("CUST-001".to_string());
        ar_line.auxiliary_account_label = Some("Acme Corp".to_string());
        ar_line.lettrage = Some("LTR-SO00001".to_string());
        ar_line.lettrage_date = Some(date);
        je.add_line(ar_line);

        // Revenue line without auxiliary/lettrage
        je.add_line(JournalEntryLine::credit(
            je.header.document_id,
            2,
            "701000".to_string(),
            dec!(2000.00),
        ));

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fec_aux.csv");
        write_fec_csv(&path, &[je], &coa).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);

        // Row 1 (AR line): columns 7-8 populated, columns 14-15 populated
        let row1: Vec<&str> = lines[1].split(';').collect();
        assert_eq!(row1.len(), 18, "row must have 18 columns");
        assert_eq!(row1[6], "CUST-001", "column 7: auxiliary account number");
        assert_eq!(row1[7], "Acme Corp", "column 8: auxiliary account label");
        assert_eq!(row1[13], "LTR-SO00001", "column 14: lettrage");
        assert_eq!(row1[14], "20240615", "column 15: lettrage date");

        // Row 2 (Revenue line): columns 7-8 empty, columns 14-15 empty
        let row2: Vec<&str> = lines[2].split(';').collect();
        assert_eq!(row2[6], "", "column 7: empty for non-AR line");
        assert_eq!(row2[7], "", "column 8: empty for non-AR line");
        assert_eq!(row2[13], "", "column 14: empty for non-AR line");
        assert_eq!(row2[14], "", "column 15: empty for non-AR line");
    }
}
