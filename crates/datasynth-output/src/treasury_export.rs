//! Export treasury and cash management data to CSV files.
//!
//! Exports cash positions, forecasts, pool sweeps, hedging instruments,
//! hedge relationships, debt instruments, covenants, amortization schedules,
//! bank guarantees, netting runs, netting positions, and anomaly labels.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use datasynth_core::error::SynthResult;
use datasynth_core::models::{
    BankGuarantee, CashForecast, CashPoolSweep, CashPosition, DebtInstrument, HedgeRelationship,
    HedgingInstrument, NettingRun,
};

// ---------------------------------------------------------------------------
// Anomaly label row (string-based for export)
// ---------------------------------------------------------------------------

/// A pre-serialized treasury anomaly label row for CSV export.
#[derive(Debug, Clone)]
pub struct TreasuryAnomalyLabelRow {
    pub id: String,
    pub anomaly_type: String,
    pub severity: String,
    pub document_type: String,
    pub document_id: String,
    pub description: String,
    pub original_value: String,
    pub anomalous_value: String,
}

// ---------------------------------------------------------------------------
// Export summary
// ---------------------------------------------------------------------------

/// Summary of exported treasury data.
#[derive(Debug, Default)]
pub struct TreasuryExportSummary {
    pub cash_positions_count: usize,
    pub cash_forecasts_count: usize,
    pub cash_forecast_items_count: usize,
    pub cash_pool_sweeps_count: usize,
    pub hedging_instruments_count: usize,
    pub hedge_relationships_count: usize,
    pub debt_instruments_count: usize,
    pub debt_covenants_count: usize,
    pub amortization_schedules_count: usize,
    pub bank_guarantees_count: usize,
    pub netting_runs_count: usize,
    pub netting_positions_count: usize,
    pub anomaly_labels_count: usize,
}

impl TreasuryExportSummary {
    /// Total number of rows exported across all files.
    pub fn total(&self) -> usize {
        self.cash_positions_count
            + self.cash_forecasts_count
            + self.cash_forecast_items_count
            + self.cash_pool_sweeps_count
            + self.hedging_instruments_count
            + self.hedge_relationships_count
            + self.debt_instruments_count
            + self.debt_covenants_count
            + self.amortization_schedules_count
            + self.bank_guarantees_count
            + self.netting_runs_count
            + self.netting_positions_count
            + self.anomaly_labels_count
    }
}

// ---------------------------------------------------------------------------
// Exporter
// ---------------------------------------------------------------------------

/// Exporter for treasury and cash management data.
pub struct TreasuryExporter {
    output_dir: PathBuf,
}

impl TreasuryExporter {
    /// Create a new treasury exporter writing to the given directory.
    pub fn new(output_dir: impl AsRef<Path>) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Export cash positions to `cash_positions.csv`.
    pub fn export_cash_positions(&self, data: &[CashPosition]) -> SynthResult<usize> {
        let path = self.output_dir.join("cash_positions.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,entity_id,bank_account_id,currency,date,opening_balance,inflows,outflows,closing_balance,available_balance,value_date_balance"
        )?;

        for p in data {
            writeln!(
                w,
                "{},{},{},{},{},{},{},{},{},{},{}",
                esc(&p.id),
                esc(&p.entity_id),
                esc(&p.bank_account_id),
                esc(&p.currency),
                p.date,
                p.opening_balance,
                p.inflows,
                p.outflows,
                p.closing_balance,
                p.available_balance,
                p.value_date_balance,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export cash forecasts to `cash_forecasts.csv`.
    pub fn export_cash_forecasts(&self, data: &[CashForecast]) -> SynthResult<usize> {
        let path = self.output_dir.join("cash_forecasts.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,entity_id,currency,forecast_date,horizon_days,net_position,confidence_level,item_count"
        )?;

        for f in data {
            writeln!(
                w,
                "{},{},{},{},{},{},{},{}",
                esc(&f.id),
                esc(&f.entity_id),
                esc(&f.currency),
                f.forecast_date,
                f.horizon_days,
                f.net_position,
                f.confidence_level,
                f.items.len(),
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export cash forecast items to `cash_forecast_items.csv`.
    pub fn export_cash_forecast_items(&self, data: &[CashForecast]) -> SynthResult<usize> {
        let path = self.output_dir.join("cash_forecast_items.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,forecast_id,date,category,amount,probability,source_document_type,source_document_id"
        )?;

        let mut count = 0;
        for forecast in data {
            for item in &forecast.items {
                writeln!(
                    w,
                    "{},{},{},{:?},{},{},{},{}",
                    esc(&item.id),
                    esc(&forecast.id),
                    item.date,
                    item.category,
                    item.amount,
                    item.probability,
                    item.source_document_type.as_deref().unwrap_or(""),
                    item.source_document_id.as_deref().unwrap_or(""),
                )?;
                count += 1;
            }
        }

        w.flush()?;
        Ok(count)
    }

    /// Export cash pool sweeps to `cash_pool_sweeps.csv`.
    pub fn export_cash_pool_sweeps(&self, data: &[CashPoolSweep]) -> SynthResult<usize> {
        let path = self.output_dir.join("cash_pool_sweeps.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(w, "id,pool_id,date,from_account_id,to_account_id,amount,currency")?;

        for s in data {
            writeln!(
                w,
                "{},{},{},{},{},{},{}",
                esc(&s.id),
                esc(&s.pool_id),
                s.date,
                esc(&s.from_account_id),
                esc(&s.to_account_id),
                s.amount,
                esc(&s.currency),
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export hedging instruments to `hedging_instruments.csv`.
    pub fn export_hedging_instruments(&self, data: &[HedgingInstrument]) -> SynthResult<usize> {
        let path = self.output_dir.join("hedging_instruments.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,instrument_type,notional_amount,currency,currency_pair,fixed_rate,floating_index,strike_rate,trade_date,maturity_date,counterparty,fair_value,status"
        )?;

        for h in data {
            writeln!(
                w,
                "{},{:?},{},{},{},{},{},{},{},{},{},{},{:?}",
                esc(&h.id),
                h.instrument_type,
                h.notional_amount,
                esc(&h.currency),
                h.currency_pair.as_deref().unwrap_or(""),
                h.fixed_rate.map(|r| r.to_string()).unwrap_or_default(),
                h.floating_index.as_deref().unwrap_or(""),
                h.strike_rate.map(|r| r.to_string()).unwrap_or_default(),
                h.trade_date,
                h.maturity_date,
                esc(&h.counterparty),
                h.fair_value,
                h.status,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export hedge relationships to `hedge_relationships.csv`.
    pub fn export_hedge_relationships(&self, data: &[HedgeRelationship]) -> SynthResult<usize> {
        let path = self.output_dir.join("hedge_relationships.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,hedged_item_type,hedged_item_description,hedging_instrument_id,hedge_type,designation_date,effectiveness_test_method,effectiveness_ratio,is_effective,ineffectiveness_amount"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{:?},{},{},{:?},{},{:?},{},{},{}",
                esc(&r.id),
                r.hedged_item_type,
                esc(&r.hedged_item_description),
                esc(&r.hedging_instrument_id),
                r.hedge_type,
                r.designation_date,
                r.effectiveness_test_method,
                r.effectiveness_ratio,
                r.is_effective,
                r.ineffectiveness_amount,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export debt instruments to `debt_instruments.csv`.
    pub fn export_debt_instruments(&self, data: &[DebtInstrument]) -> SynthResult<usize> {
        let path = self.output_dir.join("debt_instruments.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,entity_id,instrument_type,lender,principal,currency,interest_rate,rate_type,origination_date,maturity_date,drawn_amount,facility_limit"
        )?;

        for d in data {
            writeln!(
                w,
                "{},{},{:?},{},{},{},{},{:?},{},{},{},{}",
                esc(&d.id),
                esc(&d.entity_id),
                d.instrument_type,
                esc(&d.lender),
                d.principal,
                esc(&d.currency),
                d.interest_rate,
                d.rate_type,
                d.origination_date,
                d.maturity_date,
                d.drawn_amount,
                d.facility_limit,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export debt covenants to `debt_covenants.csv`.
    pub fn export_debt_covenants(&self, instruments: &[DebtInstrument]) -> SynthResult<usize> {
        let path = self.output_dir.join("debt_covenants.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,debt_instrument_id,covenant_type,threshold,measurement_frequency,actual_value,measurement_date,is_compliant,headroom,waiver_obtained"
        )?;

        let mut count = 0;
        for instrument in instruments {
            for c in &instrument.covenants {
                writeln!(
                    w,
                    "{},{},{:?},{},{:?},{},{},{},{},{}",
                    esc(&c.id),
                    esc(&instrument.id),
                    c.covenant_type,
                    c.threshold,
                    c.measurement_frequency,
                    c.actual_value,
                    c.measurement_date,
                    c.is_compliant,
                    c.headroom,
                    c.waiver_obtained,
                )?;
                count += 1;
            }
        }

        w.flush()?;
        Ok(count)
    }

    /// Export amortization schedules to `amortization_schedules.csv`.
    pub fn export_amortization_schedules(
        &self,
        instruments: &[DebtInstrument],
    ) -> SynthResult<usize> {
        let path = self.output_dir.join("amortization_schedules.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "debt_instrument_id,date,principal_payment,interest_payment,total_payment,balance_after"
        )?;

        let mut count = 0;
        for instrument in instruments {
            for p in &instrument.amortization_schedule {
                writeln!(
                    w,
                    "{},{},{},{},{},{}",
                    esc(&instrument.id),
                    p.date,
                    p.principal_payment,
                    p.interest_payment,
                    p.total_payment(),
                    p.balance_after,
                )?;
                count += 1;
            }
        }

        w.flush()?;
        Ok(count)
    }

    /// Export bank guarantees to `bank_guarantees.csv`.
    pub fn export_bank_guarantees(&self, data: &[BankGuarantee]) -> SynthResult<usize> {
        let path = self.output_dir.join("bank_guarantees.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,entity_id,guarantee_type,amount,currency,beneficiary,issuing_bank,issue_date,expiry_date,status,linked_contract_id,linked_project_id"
        )?;

        for g in data {
            writeln!(
                w,
                "{},{},{:?},{},{},{},{},{},{},{:?},{},{}",
                esc(&g.id),
                esc(&g.entity_id),
                g.guarantee_type,
                g.amount,
                esc(&g.currency),
                esc(&g.beneficiary),
                esc(&g.issuing_bank),
                g.issue_date,
                g.expiry_date,
                g.status,
                g.linked_contract_id.as_deref().unwrap_or(""),
                g.linked_project_id.as_deref().unwrap_or(""),
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export netting runs to `netting_runs.csv`.
    pub fn export_netting_runs(&self, data: &[NettingRun]) -> SynthResult<usize> {
        let path = self.output_dir.join("netting_runs.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,netting_date,cycle,gross_receivables,gross_payables,net_settlement,settlement_currency,savings,savings_pct,participant_count"
        )?;

        for n in data {
            writeln!(
                w,
                "{},{},{:?},{},{},{},{},{},{},{}",
                esc(&n.id),
                n.netting_date,
                n.cycle,
                n.gross_receivables,
                n.gross_payables,
                n.net_settlement,
                esc(&n.settlement_currency),
                n.savings(),
                n.savings_pct(),
                n.participating_entities.len(),
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export netting positions to `netting_positions.csv`.
    pub fn export_netting_positions(&self, data: &[NettingRun]) -> SynthResult<usize> {
        let path = self.output_dir.join("netting_positions.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "netting_run_id,entity_id,gross_receivable,gross_payable,net_position,settlement_direction"
        )?;

        let mut count = 0;
        for run in data {
            for pos in &run.positions {
                writeln!(
                    w,
                    "{},{},{},{},{},{:?}",
                    esc(&run.id),
                    esc(&pos.entity_id),
                    pos.gross_receivable,
                    pos.gross_payable,
                    pos.net_position,
                    pos.settlement_direction,
                )?;
                count += 1;
            }
        }

        w.flush()?;
        Ok(count)
    }

    /// Export anomaly labels to `treasury_anomaly_labels.csv`.
    pub fn export_anomaly_labels(&self, data: &[TreasuryAnomalyLabelRow]) -> SynthResult<usize> {
        let path = self.output_dir.join("treasury_anomaly_labels.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,anomaly_type,severity,document_type,document_id,description,original_value,anomalous_value"
        )?;

        for a in data {
            writeln!(
                w,
                "{},{},{},{},{},{},{},{}",
                esc(&a.id),
                esc(&a.anomaly_type),
                esc(&a.severity),
                esc(&a.document_type),
                esc(&a.document_id),
                esc(&a.description),
                esc(&a.original_value),
                esc(&a.anomalous_value),
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }
}

/// Escape a string for CSV output.
fn esc(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use tempfile::TempDir;

    use datasynth_core::models::{
        AmortizationPayment, CovenantType, DebtCovenant, DebtType, EffectivenessMethod, Frequency,
        HedgeInstrumentType, HedgeType, HedgedItemType, InterestRateType, NettingCycle,
        NettingPosition, PayOrReceive, TreasuryCashFlowCategory,
    };

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn test_export_cash_positions() {
        let temp = TempDir::new().unwrap();
        let exporter = TreasuryExporter::new(temp.path());

        let positions = vec![
            CashPosition::new(
                "CP-001",
                "C001",
                "BA-001",
                "USD",
                d("2025-01-15"),
                dec!(100000),
                dec!(5000),
                dec!(2000),
            ),
            CashPosition::new(
                "CP-002",
                "C001",
                "BA-001",
                "USD",
                d("2025-01-16"),
                dec!(103000),
                dec!(0),
                dec!(1000),
            ),
        ];

        let count = exporter.export_cash_positions(&positions).unwrap();
        assert_eq!(count, 2);

        let content = std::fs::read_to_string(temp.path().join("cash_positions.csv")).unwrap();
        assert_eq!(content.lines().count(), 3); // header + 2 rows
        assert!(content.contains("CP-001"));
        assert!(content.contains("CP-002"));
        assert!(content.contains("opening_balance"));
    }

    #[test]
    fn test_export_cash_forecasts_and_items() {
        let temp = TempDir::new().unwrap();
        let exporter = TreasuryExporter::new(temp.path());

        let items = vec![datasynth_core::models::CashForecastItem {
            id: "CFI-001".to_string(),
            date: d("2025-02-15"),
            category: TreasuryCashFlowCategory::ArCollection,
            amount: dec!(50000),
            probability: dec!(0.90),
            source_document_type: Some("SalesOrder".to_string()),
            source_document_id: Some("SO-001".to_string()),
        }];
        let forecasts = vec![CashForecast::new(
            "CF-001",
            "C001",
            "USD",
            d("2025-01-31"),
            90,
            items,
            dec!(0.90),
        )];

        let fc_count = exporter.export_cash_forecasts(&forecasts).unwrap();
        let fi_count = exporter.export_cash_forecast_items(&forecasts).unwrap();

        assert_eq!(fc_count, 1);
        assert_eq!(fi_count, 1);

        let fc_content = std::fs::read_to_string(temp.path().join("cash_forecasts.csv")).unwrap();
        assert!(fc_content.contains("CF-001"));

        let fi_content =
            std::fs::read_to_string(temp.path().join("cash_forecast_items.csv")).unwrap();
        assert!(fi_content.contains("CFI-001"));
        assert!(fi_content.contains("CF-001")); // parent reference
    }

    #[test]
    fn test_export_hedging_and_relationships() {
        let temp = TempDir::new().unwrap();
        let exporter = TreasuryExporter::new(temp.path());

        let instruments = vec![HedgingInstrument::new(
            "HI-001",
            HedgeInstrumentType::FxForward,
            dec!(1000000),
            "EUR",
            d("2025-01-01"),
            d("2025-06-30"),
            "Deutsche Bank",
        )
        .with_currency_pair("EUR/USD")
        .with_fixed_rate(dec!(1.0850))];

        let relationships = vec![HedgeRelationship::new(
            "HR-001",
            HedgedItemType::ForecastedTransaction,
            "EUR receivables Q2",
            "HI-001",
            HedgeType::CashFlowHedge,
            d("2025-01-01"),
            EffectivenessMethod::Regression,
            dec!(0.95),
        )];

        let hi_count = exporter.export_hedging_instruments(&instruments).unwrap();
        let hr_count = exporter.export_hedge_relationships(&relationships).unwrap();

        assert_eq!(hi_count, 1);
        assert_eq!(hr_count, 1);
    }

    #[test]
    fn test_export_debt_with_covenants_and_amortization() {
        let temp = TempDir::new().unwrap();
        let exporter = TreasuryExporter::new(temp.path());

        let instruments = vec![DebtInstrument::new(
            "DEBT-001",
            "C001",
            DebtType::TermLoan,
            "First Bank",
            dec!(1000000),
            "USD",
            dec!(0.05),
            InterestRateType::Fixed,
            d("2025-01-01"),
            d("2026-01-01"),
        )
        .with_amortization_schedule(vec![AmortizationPayment {
            date: d("2025-06-30"),
            principal_payment: dec!(500000),
            interest_payment: dec!(25000),
            balance_after: dec!(500000),
        }])
        .with_covenant(DebtCovenant::new(
            "COV-001",
            CovenantType::DebtToEbitda,
            dec!(3.5),
            Frequency::Quarterly,
            dec!(2.5),
            d("2025-03-31"),
        ))];

        let di_count = exporter.export_debt_instruments(&instruments).unwrap();
        let dc_count = exporter.export_debt_covenants(&instruments).unwrap();
        let as_count = exporter.export_amortization_schedules(&instruments).unwrap();

        assert_eq!(di_count, 1);
        assert_eq!(dc_count, 1);
        assert_eq!(as_count, 1);

        let dc_csv = std::fs::read_to_string(temp.path().join("debt_covenants.csv")).unwrap();
        assert!(dc_csv.contains("DEBT-001")); // parent reference
        assert!(dc_csv.contains("COV-001"));
    }

    #[test]
    fn test_export_netting() {
        let temp = TempDir::new().unwrap();
        let exporter = TreasuryExporter::new(temp.path());

        let runs = vec![NettingRun::new(
            "NR-001",
            d("2025-01-31"),
            NettingCycle::Monthly,
            "USD",
            vec![
                NettingPosition {
                    entity_id: "C001".to_string(),
                    gross_receivable: dec!(100000),
                    gross_payable: dec!(60000),
                    net_position: dec!(40000),
                    settlement_direction: PayOrReceive::Receive,
                },
                NettingPosition {
                    entity_id: "C002".to_string(),
                    gross_receivable: dec!(60000),
                    gross_payable: dec!(100000),
                    net_position: dec!(-40000),
                    settlement_direction: PayOrReceive::Pay,
                },
            ],
        )];

        let nr_count = exporter.export_netting_runs(&runs).unwrap();
        let np_count = exporter.export_netting_positions(&runs).unwrap();

        assert_eq!(nr_count, 1);
        assert_eq!(np_count, 2);
    }

    #[test]
    fn test_export_anomaly_labels() {
        let temp = TempDir::new().unwrap();
        let exporter = TreasuryExporter::new(temp.path());

        let labels = vec![TreasuryAnomalyLabelRow {
            id: "TANOM-001".to_string(),
            anomaly_type: "hedge_ineffectiveness".to_string(),
            severity: "high".to_string(),
            document_type: "hedge_relationship".to_string(),
            document_id: "HR-001".to_string(),
            description: "Ratio 0.72 outside corridor".to_string(),
            original_value: "0.95".to_string(),
            anomalous_value: "0.72".to_string(),
        }];

        let count = exporter.export_anomaly_labels(&labels).unwrap();
        assert_eq!(count, 1);

        let content =
            std::fs::read_to_string(temp.path().join("treasury_anomaly_labels.csv")).unwrap();
        assert!(content.contains("TANOM-001"));
        assert!(content.contains("hedge_ineffectiveness"));
    }

    #[test]
    fn test_export_summary_total() {
        let summary = TreasuryExportSummary {
            cash_positions_count: 30,
            cash_forecasts_count: 1,
            cash_forecast_items_count: 15,
            cash_pool_sweeps_count: 20,
            hedging_instruments_count: 5,
            hedge_relationships_count: 5,
            debt_instruments_count: 2,
            debt_covenants_count: 4,
            amortization_schedules_count: 40,
            bank_guarantees_count: 3,
            netting_runs_count: 1,
            netting_positions_count: 4,
            anomaly_labels_count: 3,
        };
        assert_eq!(summary.total(), 133);
    }
}
