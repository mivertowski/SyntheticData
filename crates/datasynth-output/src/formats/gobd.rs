//! GoBD (Grundsätze zur ordnungsmäßigen Führung und Aufbewahrung von Büchern,
//! Aufzeichnungen und Unterlagen in elektronischer Form) export for German GAAP.
//!
//! Exports three files:
//! 1. `gobd_journal.csv` — semicolon-separated journal entries (13 columns)
//! 2. `gobd_accounts.csv` — chart of accounts listing
//! 3. `index.xml` — GoBD-compliant table schema index

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use datasynth_core::error::SynthResult;
use datasynth_core::models::{ChartOfAccounts, JournalEntry};

/// GoBD journal CSV header (13 mandatory columns).
const GOBD_JOURNAL_HEADER: &str = "Belegdatum;Buchungsdatum;Belegnummer;Buchungstext;Kontonummer;Gegenkontonummer;Sollbetrag;Habenbetrag;Steuerschlüssel;Steuerbetrag;Währung;Kostenstelle;Belegnummernkreis";

/// GoBD accounts CSV header.
const GOBD_ACCOUNTS_HEADER: &str = "Kontonummer;Kontobezeichnung;Kontotyp;Saldo";

fn escape_gobd_field(s: &str) -> String {
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

/// Write journal entries to a GoBD-compliant CSV file (semicolon-separated, UTF-8).
///
/// 13 columns per row:
/// Belegdatum, Buchungsdatum, Belegnummer, Buchungstext, Kontonummer,
/// Gegenkontonummer, Sollbetrag, Habenbetrag, Steuerschlüssel, Steuerbetrag,
/// Währung, Kostenstelle, Belegnummernkreis
pub fn write_gobd_journal_csv(
    path: &Path,
    entries: &[JournalEntry],
    _coa: &ChartOfAccounts,
) -> SynthResult<()> {
    let file = File::create(path)?;
    let mut w = BufWriter::with_capacity(256 * 1024, file);

    writeln!(w, "{}", GOBD_JOURNAL_HEADER)?;

    for je in entries {
        let beleg_datum = je.header.document_date.format("%Y%m%d").to_string();
        let buchungs_datum = je.header.posting_date.format("%Y%m%d").to_string();
        let beleg_nummer = escape_gobd_field(&je.header.document_id.to_string()[..8]);
        let buchungstext = escape_gobd_field(
            je.header
                .header_text
                .as_deref()
                .unwrap_or(je.header.document_type.as_str()),
        );
        let waehrung = escape_gobd_field(&je.header.currency);
        let beleg_kreis = escape_gobd_field(je.header.document_type.as_str());

        // Determine contra account: if 2 lines, use the other line's account
        let contra_for = |idx: usize| -> String {
            if je.lines.len() == 2 {
                let other = if idx == 0 { 1 } else { 0 };
                je.lines[other].gl_account.clone()
            } else {
                String::new()
            }
        };

        for (idx, line) in je.lines.iter().enumerate() {
            let konto = escape_gobd_field(&line.gl_account);
            let gegen_konto = escape_gobd_field(&contra_for(idx));
            let soll = format_decimal(line.debit_amount);
            let haben = format_decimal(line.credit_amount);
            let steuer_schluessel = line.tax_code.as_deref().unwrap_or("");
            let steuer_betrag = "0.00"; // Tax amount not tracked separately in JE lines
            let kostenstelle = line.cost_center.as_deref().unwrap_or("");

            writeln!(
                w,
                "{};{};{};{};{};{};{};{};{};{};{};{};{}",
                beleg_datum,
                buchungs_datum,
                beleg_nummer,
                buchungstext,
                konto,
                gegen_konto,
                soll,
                haben,
                steuer_schluessel,
                steuer_betrag,
                waehrung,
                kostenstelle,
                beleg_kreis,
            )?;
        }
    }

    w.flush()?;
    Ok(())
}

/// Write chart of accounts to a GoBD-compliant CSV file.
pub fn write_gobd_accounts_csv(path: &Path, coa: &ChartOfAccounts) -> SynthResult<()> {
    let file = File::create(path)?;
    let mut w = BufWriter::with_capacity(64 * 1024, file);

    writeln!(w, "{}", GOBD_ACCOUNTS_HEADER)?;

    for account in &coa.accounts {
        writeln!(
            w,
            "{};{};{:?};0.00",
            escape_gobd_field(&account.account_number),
            escape_gobd_field(&account.short_description),
            account.account_type,
        )?;
    }

    w.flush()?;
    Ok(())
}

/// Write a GoBD-compliant XML index file.
///
/// Contains table schema descriptions for the journal and accounts exports.
pub fn write_gobd_index_xml(
    path: &Path,
    company_code: &str,
    fiscal_year: i32,
    tables: &[(&str, &str)],
) -> SynthResult<()> {
    let file = File::create(path)?;
    let mut w = BufWriter::new(file);

    writeln!(w, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(w, "<GoBD xmlns=\"urn:de:gobd:2024\" version=\"1.0\">")?;
    writeln!(w, "  <Header>")?;
    writeln!(w, "    <Company>{}</Company>", escape_xml(company_code))?;
    writeln!(w, "    <FiscalYear>{}</FiscalYear>", fiscal_year)?;
    writeln!(
        w,
        "    <ExportDate>{}</ExportDate>",
        chrono::Utc::now().format("%Y-%m-%d")
    )?;
    writeln!(w, "    <Format>CSV</Format>")?;
    writeln!(w, "    <Delimiter>semicolon</Delimiter>")?;
    writeln!(w, "    <Encoding>UTF-8</Encoding>")?;
    writeln!(w, "  </Header>")?;
    writeln!(w, "  <Tables>")?;

    for (filename, description) in tables {
        writeln!(w, "    <Table>")?;
        writeln!(w, "      <Filename>{}</Filename>", escape_xml(filename))?;
        writeln!(
            w,
            "      <Description>{}</Description>",
            escape_xml(description)
        )?;
        writeln!(w, "    </Table>")?;
    }

    writeln!(w, "  </Tables>")?;
    writeln!(w, "</GoBD>")?;

    w.flush()?;
    Ok(())
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_core::models::{
        AccountSubType, AccountType, CoAComplexity, GLAccount, IndustrySector, JournalEntryHeader,
        JournalEntryLine,
    };
    use rust_decimal_macros::dec;

    fn test_coa() -> ChartOfAccounts {
        let mut coa = ChartOfAccounts::new(
            "TEST".to_string(),
            "Test CoA".to_string(),
            "DE".to_string(),
            IndustrySector::Manufacturing,
            CoAComplexity::Small,
        );
        coa.add_account(GLAccount::new(
            "1200".to_string(),
            "Forderungen aus L+L".to_string(),
            AccountType::Asset,
            AccountSubType::AccountsReceivable,
        ));
        coa.add_account(GLAccount::new(
            "4000".to_string(),
            "Umsatzerlöse".to_string(),
            AccountType::Revenue,
            AccountSubType::ProductRevenue,
        ));
        coa
    }

    fn test_je() -> JournalEntry {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let mut header = JournalEntryHeader::new("C001".to_string(), date);
        header.currency = "EUR".to_string();
        header.header_text = Some("Umsatzerlöse".to_string());
        let mut je = JournalEntry::new(header);
        je.add_line(JournalEntryLine::debit(
            je.header.document_id,
            1,
            "1200".to_string(),
            dec!(1500.00),
        ));
        je.add_line(JournalEntryLine::credit(
            je.header.document_id,
            2,
            "4000".to_string(),
            dec!(1500.00),
        ));
        je
    }

    #[test]
    fn test_gobd_journal_13_columns() {
        let coa = test_coa();
        let je = test_je();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("gobd_journal.csv");
        write_gobd_journal_csv(&path, &[je], &coa).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3, "header + 2 data rows");

        for (i, line) in lines.iter().enumerate() {
            let cols: Vec<&str> = line.split(';').collect();
            assert_eq!(
                cols.len(),
                13,
                "row {} has {} columns, expected 13",
                i,
                cols.len()
            );
        }
    }

    #[test]
    fn test_gobd_accounts_csv() {
        let coa = test_coa();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("gobd_accounts.csv");
        write_gobd_accounts_csv(&path, &coa).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        // Header + 2 accounts
        assert_eq!(lines.len(), 3);
        assert!(lines[1].starts_with("1200;"));
        assert!(lines[2].starts_with("4000;"));
    }

    #[test]
    fn test_gobd_index_xml_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("index.xml");

        let tables = vec![
            ("gobd_journal.csv", "Buchungsjournal"),
            ("gobd_accounts.csv", "Kontenplan"),
        ];
        write_gobd_index_xml(&path, "C001", 2024, &tables).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("<?xml"));
        assert!(content.contains("<Company>C001</Company>"));
        assert!(content.contains("<FiscalYear>2024</FiscalYear>"));
        assert!(content.contains("<Filename>gobd_journal.csv</Filename>"));
        assert!(content.contains("<Filename>gobd_accounts.csv</Filename>"));
        assert!(content.contains("</GoBD>"));
    }

    #[test]
    fn test_gobd_data_row_round_trip() {
        let coa = test_coa();
        let je = test_je();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("gobd_rt.csv");
        write_gobd_journal_csv(&path, &[je], &coa).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Row 1: debit line
        let cols: Vec<&str> = lines[1].split(';').collect();
        assert_eq!(cols[0], "20240615", "Belegdatum");
        assert_eq!(cols[4], "1200", "Kontonummer");
        assert_eq!(cols[5], "4000", "Gegenkontonummer");
        assert_eq!(cols[6], "1500.00", "Sollbetrag");
        assert_eq!(cols[7], "0.00", "Habenbetrag");
        assert_eq!(cols[10], "EUR", "Währung");
    }
}
