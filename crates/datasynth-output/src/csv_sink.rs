//! CSV output sink with optional disk space monitoring.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;

use datasynth_core::error::{SynthError, SynthResult};
use datasynth_core::models::subledger::ar::{
    DunningItem, DunningLetter, DunningRun, OnAccountPayment, PaymentCorrection, ShortPayment,
};
use datasynth_core::models::JournalEntry;
use datasynth_core::traits::Sink;
use datasynth_core::{DiskSpaceGuard, DiskSpaceGuardConfig};

/// CSV sink for journal entry output with optional disk space monitoring.
pub struct CsvSink {
    writer: BufWriter<File>,
    items_written: u64,
    bytes_written: u64,
    header_written: bool,
    /// Optional disk space guard for monitoring available space
    disk_guard: Option<Arc<DiskSpaceGuard>>,
    /// Interval for disk checks (every N items)
    check_interval: u64,
}

impl CsvSink {
    /// Create a new CSV sink.
    pub fn new(path: PathBuf) -> SynthResult<Self> {
        let file = File::create(&path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            items_written: 0,
            bytes_written: 0,
            header_written: false,
            disk_guard: None,
            check_interval: 500,
        })
    }

    /// Create a new CSV sink with disk space monitoring.
    pub fn with_disk_guard(path: PathBuf, min_free_mb: usize) -> SynthResult<Self> {
        let file = File::create(&path)?;
        let disk_config = DiskSpaceGuardConfig::with_min_free_mb(min_free_mb).with_path(&path);
        let disk_guard = Arc::new(DiskSpaceGuard::new(disk_config));

        Ok(Self {
            writer: BufWriter::new(file),
            items_written: 0,
            bytes_written: 0,
            header_written: false,
            disk_guard: Some(disk_guard),
            check_interval: 500,
        })
    }

    /// Set a custom disk guard.
    pub fn set_disk_guard(&mut self, guard: Arc<DiskSpaceGuard>) {
        self.disk_guard = Some(guard);
    }

    /// Set the disk check interval.
    pub fn set_check_interval(&mut self, interval: u64) {
        self.check_interval = interval;
    }

    /// Check disk space if guard is configured.
    fn check_disk_space(&self) -> SynthResult<()> {
        if let Some(guard) = &self.disk_guard {
            if self.items_written.is_multiple_of(self.check_interval) {
                guard
                    .check()
                    .map_err(|e| SynthError::disk_exhausted(e.available_mb, e.required_mb))?;
            }
        }
        Ok(())
    }

    /// Record bytes written for tracking.
    fn record_write(&self, bytes: u64) {
        if let Some(guard) = &self.disk_guard {
            guard.record_write(bytes);
        }
    }

    fn write_header(&mut self) -> SynthResult<()> {
        if self.header_written {
            return Ok(());
        }

        let header = "document_id,company_code,fiscal_year,fiscal_period,posting_date,\
            document_type,currency,source,line_number,gl_account,debit_amount,credit_amount\n";
        let bytes = header.as_bytes();
        self.writer.write_all(bytes)?;
        self.bytes_written += bytes.len() as u64;
        self.record_write(bytes.len() as u64);
        self.header_written = true;
        Ok(())
    }

    /// Get total bytes written.
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}

impl Sink for CsvSink {
    type Item = JournalEntry;

    fn write(&mut self, item: Self::Item) -> SynthResult<()> {
        // Check disk space periodically
        self.check_disk_space()?;

        self.write_header()?;

        for line in &item.lines {
            let row = format!(
                "{},{},{},{},{},{},{},{:?},{},{},{},{}\n",
                item.header.document_id,
                item.header.company_code,
                item.header.fiscal_year,
                item.header.fiscal_period,
                item.header.posting_date,
                item.header.document_type,
                item.header.currency,
                item.header.source,
                line.line_number,
                line.gl_account,
                line.debit_amount,
                line.credit_amount,
            );
            let bytes = row.as_bytes();
            self.writer.write_all(bytes)?;
            self.bytes_written += bytes.len() as u64;
            self.record_write(bytes.len() as u64);
        }

        self.items_written += 1;
        Ok(())
    }

    fn flush(&mut self) -> SynthResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn close(mut self) -> SynthResult<()> {
        self.flush()?;
        Ok(())
    }

    fn items_written(&self) -> u64 {
        self.items_written
    }
}

// ============================================================================
// Dunning Run CSV Sink
// ============================================================================

/// CSV sink for dunning runs.
pub struct DunningRunCsvSink {
    writer: BufWriter<File>,
    items_written: u64,
    header_written: bool,
}

impl DunningRunCsvSink {
    /// Create a new dunning run CSV sink.
    pub fn new(path: PathBuf) -> SynthResult<Self> {
        let file = File::create(&path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            items_written: 0,
            header_written: false,
        })
    }

    fn write_header(&mut self) -> SynthResult<()> {
        if self.header_written {
            return Ok(());
        }

        let header = "run_id,company_code,run_date,dunning_date,customers_evaluated,\
            customers_with_letters,letters_generated,total_amount_dunned,\
            total_dunning_charges,total_interest_amount,status,started_at,completed_at\n";
        self.writer.write_all(header.as_bytes())?;
        self.header_written = true;
        Ok(())
    }
}

impl Sink for DunningRunCsvSink {
    type Item = DunningRun;

    fn write(&mut self, item: Self::Item) -> SynthResult<()> {
        self.write_header()?;

        let row = format!(
            "{},{},{},{},{},{},{},{},{},{},{:?},{},{}\n",
            item.run_id,
            item.company_code,
            item.run_date,
            item.dunning_date,
            item.customers_evaluated,
            item.customers_with_letters,
            item.letters_generated,
            item.total_amount_dunned,
            item.total_dunning_charges,
            item.total_interest_amount,
            item.status,
            item.started_at,
            item.completed_at.map(|d| d.to_string()).unwrap_or_default(),
        );
        self.writer.write_all(row.as_bytes())?;

        self.items_written += 1;
        Ok(())
    }

    fn flush(&mut self) -> SynthResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn close(mut self) -> SynthResult<()> {
        self.flush()?;
        Ok(())
    }

    fn items_written(&self) -> u64 {
        self.items_written
    }
}

// ============================================================================
// Dunning Letter CSV Sink
// ============================================================================

/// CSV sink for dunning letters.
pub struct DunningLetterCsvSink {
    writer: BufWriter<File>,
    items_written: u64,
    header_written: bool,
}

impl DunningLetterCsvSink {
    /// Create a new dunning letter CSV sink.
    pub fn new(path: PathBuf) -> SynthResult<Self> {
        let file = File::create(&path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            items_written: 0,
            header_written: false,
        })
    }

    fn write_header(&mut self) -> SynthResult<()> {
        if self.header_written {
            return Ok(());
        }

        let header = "letter_id,dunning_run_id,company_code,customer_id,customer_name,\
            dunning_level,dunning_date,total_dunned_amount,dunning_charges,\
            interest_amount,total_amount_due,currency,payment_deadline,\
            is_sent,sent_date,response_type,status\n";
        self.writer.write_all(header.as_bytes())?;
        self.header_written = true;
        Ok(())
    }
}

impl Sink for DunningLetterCsvSink {
    type Item = DunningLetter;

    fn write(&mut self, item: Self::Item) -> SynthResult<()> {
        self.write_header()?;

        let row = format!(
            "{},{},{},{},\"{}\",{},{},{},{},{},{},{},{},{},{},{:?},{:?}\n",
            item.letter_id,
            item.dunning_run_id,
            item.company_code,
            item.customer_id,
            item.customer_name.replace('"', "\"\""),
            item.dunning_level,
            item.dunning_date,
            item.total_dunned_amount,
            item.dunning_charges,
            item.interest_amount,
            item.total_amount_due,
            item.currency,
            item.payment_deadline,
            item.is_sent,
            item.sent_date.map(|d| d.to_string()).unwrap_or_default(),
            item.response_type,
            item.status,
        );
        self.writer.write_all(row.as_bytes())?;

        self.items_written += 1;
        Ok(())
    }

    fn flush(&mut self) -> SynthResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn close(mut self) -> SynthResult<()> {
        self.flush()?;
        Ok(())
    }

    fn items_written(&self) -> u64 {
        self.items_written
    }
}

// ============================================================================
// Dunning Item CSV Sink
// ============================================================================

/// CSV sink for dunning items.
pub struct DunningItemCsvSink {
    writer: BufWriter<File>,
    items_written: u64,
    header_written: bool,
}

impl DunningItemCsvSink {
    /// Create a new dunning item CSV sink.
    pub fn new(path: PathBuf) -> SynthResult<Self> {
        let file = File::create(&path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            items_written: 0,
            header_written: false,
        })
    }

    fn write_header(&mut self) -> SynthResult<()> {
        if self.header_written {
            return Ok(());
        }

        let header = "letter_id,invoice_number,invoice_date,due_date,original_amount,\
            open_amount,days_overdue,interest_amount,previous_dunning_level,\
            new_dunning_level,is_blocked,block_reason\n";
        self.writer.write_all(header.as_bytes())?;
        self.header_written = true;
        Ok(())
    }

    /// Write a dunning item with its associated letter ID.
    pub fn write_with_letter_id(&mut self, letter_id: &str, item: &DunningItem) -> SynthResult<()> {
        self.write_header()?;

        let row = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            letter_id,
            item.invoice_number,
            item.invoice_date,
            item.due_date,
            item.original_amount,
            item.open_amount,
            item.days_overdue,
            item.interest_amount,
            item.previous_dunning_level,
            item.new_dunning_level,
            item.is_blocked,
            item.block_reason.as_deref().unwrap_or(""),
        );
        self.writer.write_all(row.as_bytes())?;

        self.items_written += 1;
        Ok(())
    }

    /// Flush the writer.
    pub fn flush(&mut self) -> SynthResult<()> {
        self.writer.flush()?;
        Ok(())
    }
}

// ============================================================================
// Payment Correction CSV Sink
// ============================================================================

/// CSV sink for payment corrections.
pub struct PaymentCorrectionCsvSink {
    writer: BufWriter<File>,
    items_written: u64,
    header_written: bool,
}

impl PaymentCorrectionCsvSink {
    /// Create a new payment correction CSV sink.
    pub fn new(path: PathBuf) -> SynthResult<Self> {
        let file = File::create(&path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            items_written: 0,
            header_written: false,
        })
    }

    fn write_header(&mut self) -> SynthResult<()> {
        if self.header_written {
            return Ok(());
        }

        let header = "correction_id,company_code,customer_id,original_payment_id,\
            correction_type,original_amount,correction_amount,currency,\
            correction_date,reversal_je_id,correcting_payment_id,status,\
            reason,bank_reference,chargeback_code,fee_amount\n";
        self.writer.write_all(header.as_bytes())?;
        self.header_written = true;
        Ok(())
    }
}

impl Sink for PaymentCorrectionCsvSink {
    type Item = PaymentCorrection;

    fn write(&mut self, item: Self::Item) -> SynthResult<()> {
        self.write_header()?;

        let row = format!(
            "{},{},{},{},{:?},{},{},{},{},{},{},{:?},\"{}\",{},{},{}\n",
            item.correction_id,
            item.company_code,
            item.customer_id,
            item.original_payment_id,
            item.correction_type,
            item.original_amount,
            item.correction_amount,
            item.currency,
            item.correction_date,
            item.reversal_je_id.as_deref().unwrap_or(""),
            item.correcting_payment_id.as_deref().unwrap_or(""),
            item.status,
            item.reason.as_deref().unwrap_or("").replace('"', "\"\""),
            item.bank_reference.as_deref().unwrap_or(""),
            item.chargeback_code.as_deref().unwrap_or(""),
            item.fee_amount,
        );
        self.writer.write_all(row.as_bytes())?;

        self.items_written += 1;
        Ok(())
    }

    fn flush(&mut self) -> SynthResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn close(mut self) -> SynthResult<()> {
        self.flush()?;
        Ok(())
    }

    fn items_written(&self) -> u64 {
        self.items_written
    }
}

// ============================================================================
// Short Payment CSV Sink
// ============================================================================

/// CSV sink for short payments.
pub struct ShortPaymentCsvSink {
    writer: BufWriter<File>,
    items_written: u64,
    header_written: bool,
}

impl ShortPaymentCsvSink {
    /// Create a new short payment CSV sink.
    pub fn new(path: PathBuf) -> SynthResult<Self> {
        let file = File::create(&path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            items_written: 0,
            header_written: false,
        })
    }

    fn write_header(&mut self) -> SynthResult<()> {
        if self.header_written {
            return Ok(());
        }

        let header = "short_payment_id,company_code,customer_id,payment_id,invoice_id,\
            expected_amount,paid_amount,short_amount,currency,payment_date,\
            reason_code,reason_description,disposition,credit_memo_id,\
            write_off_je_id,rebill_invoice_id\n";
        self.writer.write_all(header.as_bytes())?;
        self.header_written = true;
        Ok(())
    }
}

impl Sink for ShortPaymentCsvSink {
    type Item = ShortPayment;

    fn write(&mut self, item: Self::Item) -> SynthResult<()> {
        self.write_header()?;

        let row = format!(
            "{},{},{},{},{},{},{},{},{},{},{:?},\"{}\",{:?},{},{},{}\n",
            item.short_payment_id,
            item.company_code,
            item.customer_id,
            item.payment_id,
            item.invoice_id,
            item.expected_amount,
            item.paid_amount,
            item.short_amount,
            item.currency,
            item.payment_date,
            item.reason_code,
            item.reason_description
                .as_deref()
                .unwrap_or("")
                .replace('"', "\"\""),
            item.disposition,
            item.credit_memo_id.as_deref().unwrap_or(""),
            item.write_off_je_id.as_deref().unwrap_or(""),
            item.rebill_invoice_id.as_deref().unwrap_or(""),
        );
        self.writer.write_all(row.as_bytes())?;

        self.items_written += 1;
        Ok(())
    }

    fn flush(&mut self) -> SynthResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn close(mut self) -> SynthResult<()> {
        self.flush()?;
        Ok(())
    }

    fn items_written(&self) -> u64 {
        self.items_written
    }
}

// ============================================================================
// On-Account Payment CSV Sink
// ============================================================================

/// CSV sink for on-account payments.
pub struct OnAccountPaymentCsvSink {
    writer: BufWriter<File>,
    items_written: u64,
    header_written: bool,
}

impl OnAccountPaymentCsvSink {
    /// Create a new on-account payment CSV sink.
    pub fn new(path: PathBuf) -> SynthResult<Self> {
        let file = File::create(&path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            items_written: 0,
            header_written: false,
        })
    }

    fn write_header(&mut self) -> SynthResult<()> {
        if self.header_written {
            return Ok(());
        }

        let header = "on_account_id,company_code,customer_id,payment_id,amount,\
            remaining_amount,currency,received_date,status,reason,\
            applications_count,notes\n";
        self.writer.write_all(header.as_bytes())?;
        self.header_written = true;
        Ok(())
    }
}

impl Sink for OnAccountPaymentCsvSink {
    type Item = OnAccountPayment;

    fn write(&mut self, item: Self::Item) -> SynthResult<()> {
        self.write_header()?;

        let row = format!(
            "{},{},{},{},{},{},{},{},{:?},{:?},{},\"{}\"\n",
            item.on_account_id,
            item.company_code,
            item.customer_id,
            item.payment_id,
            item.amount,
            item.remaining_amount,
            item.currency,
            item.received_date,
            item.status,
            item.reason,
            item.applications.len(),
            item.notes.as_deref().unwrap_or("").replace('"', "\"\""),
        );
        self.writer.write_all(row.as_bytes())?;

        self.items_written += 1;
        Ok(())
    }

    fn flush(&mut self) -> SynthResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn close(mut self) -> SynthResult<()> {
        self.flush()?;
        Ok(())
    }

    fn items_written(&self) -> u64 {
        self.items_written
    }
}
