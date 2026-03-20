//! AR Aging analysis model.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ARInvoice;
use crate::models::subledger::SubledgerDocumentStatus;

/// Aging bucket definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgingBucket {
    /// Not yet due.
    Current,
    /// 1-30 days overdue.
    Days1To30,
    /// 31-60 days overdue.
    Days31To60,
    /// 61-90 days overdue.
    Days61To90,
    /// Over 90 days overdue.
    Over90Days,
}

impl AgingBucket {
    /// Gets all buckets in order.
    pub fn all() -> Vec<AgingBucket> {
        vec![
            AgingBucket::Current,
            AgingBucket::Days1To30,
            AgingBucket::Days31To60,
            AgingBucket::Days61To90,
            AgingBucket::Over90Days,
        ]
    }

    /// Gets bucket name.
    pub fn name(&self) -> &'static str {
        match self {
            AgingBucket::Current => "Current",
            AgingBucket::Days1To30 => "1-30 Days",
            AgingBucket::Days31To60 => "31-60 Days",
            AgingBucket::Days61To90 => "61-90 Days",
            AgingBucket::Over90Days => "Over 90 Days",
        }
    }

    /// Determines bucket from days overdue.
    pub fn from_days_overdue(days: i64) -> Self {
        if days <= 0 {
            AgingBucket::Current
        } else if days <= 30 {
            AgingBucket::Days1To30
        } else if days <= 60 {
            AgingBucket::Days31To60
        } else if days <= 90 {
            AgingBucket::Days61To90
        } else {
            AgingBucket::Over90Days
        }
    }
}

/// AR Aging report for a company.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARAgingReport {
    /// Company code.
    pub company_code: String,
    /// As-of date for aging calculation.
    pub as_of_date: NaiveDate,
    /// Customer aging details.
    pub customer_details: Vec<CustomerAging>,
    /// Summary by bucket.
    pub bucket_totals: HashMap<AgingBucket, Decimal>,
    /// Total AR balance.
    pub total_ar_balance: Decimal,
    /// Total current.
    pub total_current: Decimal,
    /// Total overdue.
    pub total_overdue: Decimal,
    /// Percentage overdue.
    pub overdue_percentage: Decimal,
    /// Generated timestamp.
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl ARAgingReport {
    /// Creates an aging report from invoices.
    pub fn from_invoices(
        company_code: String,
        invoices: &[ARInvoice],
        as_of_date: NaiveDate,
    ) -> Self {
        // Group invoices by customer.
        // Only include invoices that:
        //   1. Belong to the requested company.
        //   2. Are open or partially cleared (i.e. still have an outstanding balance).
        //   3. Are dated on or before the as_of_date (future-dated invoices are excluded
        //      from an aging report because they have not yet been recognised as receivables).
        let mut customer_invoices: HashMap<String, Vec<&ARInvoice>> = HashMap::new();
        for invoice in invoices.iter().filter(|i| {
            i.company_code == company_code
                && matches!(
                    i.status,
                    SubledgerDocumentStatus::Open | SubledgerDocumentStatus::PartiallyCleared
                )
                && i.invoice_date <= as_of_date
        }) {
            customer_invoices
                .entry(invoice.customer_id.clone())
                .or_default()
                .push(invoice);
        }

        // Calculate customer aging
        let mut customer_details = Vec::new();
        let mut bucket_totals: HashMap<AgingBucket, Decimal> = AgingBucket::all()
            .into_iter()
            .map(|b| (b, Decimal::ZERO))
            .collect();

        for (customer_id, invoices) in customer_invoices {
            let customer_name = invoices
                .first()
                .map(|i| i.customer_name.clone())
                .unwrap_or_default();

            let aging =
                CustomerAging::from_invoices(customer_id, customer_name, &invoices, as_of_date);

            // Add to bucket totals
            for (bucket, amount) in &aging.bucket_amounts {
                *bucket_totals
                    .get_mut(bucket)
                    .expect("bucket initialized in map") += amount;
            }

            customer_details.push(aging);
        }

        // Sort by total balance descending
        customer_details.sort_by(|a, b| b.total_balance.cmp(&a.total_balance));

        // Calculate totals
        let total_ar_balance: Decimal = bucket_totals.values().sum();
        let total_current = bucket_totals
            .get(&AgingBucket::Current)
            .copied()
            .unwrap_or_default();
        let total_overdue = total_ar_balance - total_current;
        let overdue_percentage = if total_ar_balance > Decimal::ZERO {
            (total_overdue / total_ar_balance * dec!(100)).round_dp(2)
        } else {
            Decimal::ZERO
        };

        Self {
            company_code,
            as_of_date,
            customer_details,
            bucket_totals,
            total_ar_balance,
            total_current,
            total_overdue,
            overdue_percentage,
            generated_at: chrono::Utc::now(),
        }
    }

    /// Gets customers with balance over threshold in specific bucket.
    pub fn customers_in_bucket(
        &self,
        bucket: AgingBucket,
        min_amount: Decimal,
    ) -> Vec<&CustomerAging> {
        self.customer_details
            .iter()
            .filter(|c| c.bucket_amounts.get(&bucket).copied().unwrap_or_default() >= min_amount)
            .collect()
    }

    /// Gets top N customers by total balance.
    pub fn top_customers(&self, n: usize) -> Vec<&CustomerAging> {
        self.customer_details.iter().take(n).collect()
    }

    /// Gets customers exceeding credit limit.
    pub fn over_credit_limit(&self) -> Vec<&CustomerAging> {
        self.customer_details
            .iter()
            .filter(|c| c.is_over_credit_limit())
            .collect()
    }
}

/// Aging detail for a single customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerAging {
    /// Customer ID.
    pub customer_id: String,
    /// Customer name.
    pub customer_name: String,
    /// Credit limit.
    pub credit_limit: Option<Decimal>,
    /// Total balance.
    pub total_balance: Decimal,
    /// Amounts by bucket.
    pub bucket_amounts: HashMap<AgingBucket, Decimal>,
    /// Invoice count by bucket.
    pub invoice_counts: HashMap<AgingBucket, u32>,
    /// Oldest invoice date.
    pub oldest_invoice_date: Option<NaiveDate>,
    /// Weighted average days outstanding.
    pub weighted_avg_days: Decimal,
    /// Invoice details.
    pub invoices: Vec<AgingInvoiceDetail>,
}

impl CustomerAging {
    /// Creates customer aging from invoices.
    pub fn from_invoices(
        customer_id: String,
        customer_name: String,
        invoices: &[&ARInvoice],
        as_of_date: NaiveDate,
    ) -> Self {
        let mut bucket_amounts: HashMap<AgingBucket, Decimal> = AgingBucket::all()
            .into_iter()
            .map(|b| (b, Decimal::ZERO))
            .collect();
        let mut invoice_counts: HashMap<AgingBucket, u32> =
            AgingBucket::all().into_iter().map(|b| (b, 0)).collect();

        let mut invoice_details = Vec::new();
        let mut total_days_weighted = Decimal::ZERO;
        let mut total_balance = Decimal::ZERO;
        let mut oldest_date: Option<NaiveDate> = None;

        for invoice in invoices {
            let days_overdue = invoice.days_overdue(as_of_date);
            let bucket = AgingBucket::from_days_overdue(days_overdue);
            let amount = invoice.amount_remaining;

            *bucket_amounts
                .get_mut(&bucket)
                .expect("bucket initialized in map") += amount;
            *invoice_counts
                .get_mut(&bucket)
                .expect("bucket initialized in map") += 1;
            total_balance += amount;

            // Weighted average calculation
            let days_outstanding = (as_of_date - invoice.invoice_date).num_days();
            total_days_weighted += Decimal::from(days_outstanding) * amount;

            // Track oldest invoice
            if oldest_date.is_none_or(|d| invoice.invoice_date < d) {
                oldest_date = Some(invoice.invoice_date);
            }

            invoice_details.push(AgingInvoiceDetail {
                invoice_number: invoice.invoice_number.clone(),
                invoice_date: invoice.invoice_date,
                due_date: invoice.due_date,
                amount_remaining: amount,
                days_overdue,
                bucket,
            });
        }

        // Sort invoices by days overdue descending
        invoice_details.sort_by(|a, b| b.days_overdue.cmp(&a.days_overdue));

        let weighted_avg_days = if total_balance > Decimal::ZERO {
            (total_days_weighted / total_balance).round_dp(1)
        } else {
            Decimal::ZERO
        };

        Self {
            customer_id,
            customer_name,
            credit_limit: None,
            total_balance,
            bucket_amounts,
            invoice_counts,
            oldest_invoice_date: oldest_date,
            weighted_avg_days,
            invoices: invoice_details,
        }
    }

    /// Sets credit limit.
    pub fn with_credit_limit(mut self, limit: Decimal) -> Self {
        self.credit_limit = Some(limit);
        self
    }

    /// Checks if over credit limit.
    pub fn is_over_credit_limit(&self) -> bool {
        self.credit_limit
            .map(|limit| self.total_balance > limit)
            .unwrap_or(false)
    }

    /// Gets credit utilization percentage.
    pub fn credit_utilization(&self) -> Option<Decimal> {
        self.credit_limit.map(|limit| {
            if limit > Decimal::ZERO {
                (self.total_balance / limit * dec!(100)).round_dp(2)
            } else {
                Decimal::ZERO
            }
        })
    }

    /// Gets amount in a specific bucket.
    pub fn amount_in_bucket(&self, bucket: AgingBucket) -> Decimal {
        self.bucket_amounts
            .get(&bucket)
            .copied()
            .unwrap_or_default()
    }

    /// Gets percentage in a specific bucket.
    pub fn percentage_in_bucket(&self, bucket: AgingBucket) -> Decimal {
        if self.total_balance > Decimal::ZERO {
            let bucket_amount = self.amount_in_bucket(bucket);
            (bucket_amount / self.total_balance * dec!(100)).round_dp(2)
        } else {
            Decimal::ZERO
        }
    }
}

/// Invoice detail for aging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgingInvoiceDetail {
    /// Invoice number.
    pub invoice_number: String,
    /// Invoice date.
    pub invoice_date: NaiveDate,
    /// Due date.
    pub due_date: NaiveDate,
    /// Amount remaining.
    pub amount_remaining: Decimal,
    /// Days overdue.
    pub days_overdue: i64,
    /// Aging bucket.
    pub bucket: AgingBucket,
}

/// Bad debt reserve calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadDebtReserve {
    /// Company code.
    pub company_code: String,
    /// As-of date.
    pub as_of_date: NaiveDate,
    /// Reserve percentages by bucket.
    pub reserve_rates: HashMap<AgingBucket, Decimal>,
    /// Calculated reserves by bucket.
    pub reserves_by_bucket: HashMap<AgingBucket, Decimal>,
    /// Total reserve.
    pub total_reserve: Decimal,
    /// Total AR balance.
    pub total_ar_balance: Decimal,
    /// Reserve as percentage of AR.
    pub reserve_percentage: Decimal,
}

impl BadDebtReserve {
    /// Calculates bad debt reserve from aging report.
    pub fn calculate(
        aging_report: &ARAgingReport,
        reserve_rates: HashMap<AgingBucket, Decimal>,
    ) -> Self {
        let mut reserves_by_bucket = HashMap::new();
        let mut total_reserve = Decimal::ZERO;

        for bucket in AgingBucket::all() {
            let balance = aging_report
                .bucket_totals
                .get(&bucket)
                .copied()
                .unwrap_or_default();
            let rate = reserve_rates.get(&bucket).copied().unwrap_or_default();
            let reserve = (balance * rate / dec!(100)).round_dp(2);

            reserves_by_bucket.insert(bucket, reserve);
            total_reserve += reserve;
        }

        let reserve_percentage = if aging_report.total_ar_balance > Decimal::ZERO {
            (total_reserve / aging_report.total_ar_balance * dec!(100)).round_dp(2)
        } else {
            Decimal::ZERO
        };

        Self {
            company_code: aging_report.company_code.clone(),
            as_of_date: aging_report.as_of_date,
            reserve_rates,
            reserves_by_bucket,
            total_reserve,
            total_ar_balance: aging_report.total_ar_balance,
            reserve_percentage,
        }
    }

    /// Default reserve rates.
    pub fn default_rates() -> HashMap<AgingBucket, Decimal> {
        let mut rates = HashMap::new();
        rates.insert(AgingBucket::Current, dec!(0.5));
        rates.insert(AgingBucket::Days1To30, dec!(2));
        rates.insert(AgingBucket::Days31To60, dec!(5));
        rates.insert(AgingBucket::Days61To90, dec!(15));
        rates.insert(AgingBucket::Over90Days, dec!(50));
        rates
    }
}

/// DSO (Days Sales Outstanding) calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DSOCalculation {
    /// Company code.
    pub company_code: String,
    /// Calculation period start.
    pub period_start: NaiveDate,
    /// Calculation period end.
    pub period_end: NaiveDate,
    /// Average AR balance.
    pub average_ar: Decimal,
    /// Total revenue for period.
    pub total_revenue: Decimal,
    /// DSO result.
    pub dso_days: Decimal,
    /// Prior period DSO for comparison.
    pub prior_period_dso: Option<Decimal>,
    /// DSO change.
    pub dso_change: Option<Decimal>,
}

impl DSOCalculation {
    /// Calculates DSO.
    pub fn calculate(
        company_code: String,
        period_start: NaiveDate,
        period_end: NaiveDate,
        beginning_ar: Decimal,
        ending_ar: Decimal,
        total_revenue: Decimal,
    ) -> Self {
        let average_ar = (beginning_ar + ending_ar) / dec!(2);
        let days_in_period = (period_end - period_start).num_days();

        let dso_days = if total_revenue > Decimal::ZERO {
            (average_ar / total_revenue * Decimal::from(days_in_period)).round_dp(1)
        } else {
            Decimal::ZERO
        };

        Self {
            company_code,
            period_start,
            period_end,
            average_ar,
            total_revenue,
            dso_days,
            prior_period_dso: None,
            dso_change: None,
        }
    }

    /// Sets prior period comparison.
    pub fn with_prior_period(mut self, prior_dso: Decimal) -> Self {
        self.prior_period_dso = Some(prior_dso);
        self.dso_change = Some(self.dso_days - prior_dso);
        self
    }

    /// Checks if DSO improved (decreased).
    pub fn is_improved(&self) -> Option<bool> {
        self.dso_change.map(|change| change < Decimal::ZERO)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::subledger::PaymentTerms;

    fn create_test_invoices() -> Vec<ARInvoice> {
        vec![
            {
                let mut inv = ARInvoice::new(
                    "INV001".to_string(),
                    "1000".to_string(),
                    "CUST001".to_string(),
                    "Customer A".to_string(),
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    PaymentTerms::net_30(),
                    "USD".to_string(),
                );
                inv.amount_remaining = dec!(1000);
                inv
            },
            {
                let mut inv = ARInvoice::new(
                    "INV002".to_string(),
                    "1000".to_string(),
                    "CUST001".to_string(),
                    "Customer A".to_string(),
                    NaiveDate::from_ymd_opt(2023, 11, 1).unwrap(),
                    PaymentTerms::net_30(),
                    "USD".to_string(),
                );
                inv.amount_remaining = dec!(500);
                inv
            },
            {
                let mut inv = ARInvoice::new(
                    "INV003".to_string(),
                    "1000".to_string(),
                    "CUST002".to_string(),
                    "Customer B".to_string(),
                    NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                    PaymentTerms::net_30(),
                    "USD".to_string(),
                );
                inv.amount_remaining = dec!(2000);
                inv
            },
        ]
    }

    #[test]
    fn test_aging_bucket_from_days() {
        assert_eq!(AgingBucket::from_days_overdue(-5), AgingBucket::Current);
        assert_eq!(AgingBucket::from_days_overdue(0), AgingBucket::Current);
        assert_eq!(AgingBucket::from_days_overdue(15), AgingBucket::Days1To30);
        assert_eq!(AgingBucket::from_days_overdue(45), AgingBucket::Days31To60);
        assert_eq!(AgingBucket::from_days_overdue(75), AgingBucket::Days61To90);
        assert_eq!(AgingBucket::from_days_overdue(120), AgingBucket::Over90Days);
    }

    #[test]
    fn test_aging_report() {
        let invoices = create_test_invoices();
        let as_of_date = NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();

        let report = ARAgingReport::from_invoices("1000".to_string(), &invoices, as_of_date);

        assert_eq!(report.total_ar_balance, dec!(3500));
        assert_eq!(report.customer_details.len(), 2);
    }

    #[test]
    fn test_bad_debt_reserve() {
        let invoices = create_test_invoices();
        let as_of_date = NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
        let report = ARAgingReport::from_invoices("1000".to_string(), &invoices, as_of_date);

        let reserve = BadDebtReserve::calculate(&report, BadDebtReserve::default_rates());

        assert!(reserve.total_reserve > Decimal::ZERO);
    }

    #[test]
    fn test_dso_calculation() {
        let dso = DSOCalculation::calculate(
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            dec!(100_000),
            dec!(120_000),
            dec!(500_000),
        );

        // Average AR = 110,000, Revenue = 500,000, Days = 31
        // DSO = (110,000 / 500,000) * 31 = 6.82
        assert!(dso.dso_days > Decimal::ZERO);
    }
}
