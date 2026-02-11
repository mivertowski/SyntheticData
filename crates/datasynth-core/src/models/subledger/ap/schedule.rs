//! AP Payment Schedule and Forecasting models.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::APInvoice;
use crate::models::subledger::SubledgerDocumentStatus;

/// AP Aging bucket (similar to AR but for payables).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum APAgingBucket {
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

impl APAgingBucket {
    /// Gets all buckets in order.
    pub fn all() -> Vec<APAgingBucket> {
        vec![
            APAgingBucket::Current,
            APAgingBucket::Days1To30,
            APAgingBucket::Days31To60,
            APAgingBucket::Days61To90,
            APAgingBucket::Over90Days,
        ]
    }

    /// Gets bucket name.
    pub fn name(&self) -> &'static str {
        match self {
            APAgingBucket::Current => "Current",
            APAgingBucket::Days1To30 => "1-30 Days",
            APAgingBucket::Days31To60 => "31-60 Days",
            APAgingBucket::Days61To90 => "61-90 Days",
            APAgingBucket::Over90Days => "Over 90 Days",
        }
    }

    /// Determines bucket from days overdue.
    pub fn from_days_overdue(days: i64) -> Self {
        if days <= 0 {
            APAgingBucket::Current
        } else if days <= 30 {
            APAgingBucket::Days1To30
        } else if days <= 60 {
            APAgingBucket::Days31To60
        } else if days <= 90 {
            APAgingBucket::Days61To90
        } else {
            APAgingBucket::Over90Days
        }
    }
}

/// AP Aging report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APAgingReport {
    /// Company code.
    pub company_code: String,
    /// As-of date.
    pub as_of_date: NaiveDate,
    /// Vendor aging details.
    pub vendor_details: Vec<VendorAging>,
    /// Summary by bucket.
    pub bucket_totals: HashMap<APAgingBucket, Decimal>,
    /// Total AP balance.
    pub total_ap_balance: Decimal,
    /// Total current.
    pub total_current: Decimal,
    /// Total overdue.
    pub total_overdue: Decimal,
    /// Generated timestamp.
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl APAgingReport {
    /// Creates an aging report from invoices.
    pub fn from_invoices(
        company_code: String,
        invoices: &[APInvoice],
        as_of_date: NaiveDate,
    ) -> Self {
        // Group by vendor
        let mut vendor_invoices: HashMap<String, Vec<&APInvoice>> = HashMap::new();
        for invoice in invoices.iter().filter(|i| {
            i.company_code == company_code
                && matches!(
                    i.status,
                    SubledgerDocumentStatus::Open | SubledgerDocumentStatus::PartiallyCleared
                )
        }) {
            vendor_invoices
                .entry(invoice.vendor_id.clone())
                .or_default()
                .push(invoice);
        }

        let mut vendor_details = Vec::new();
        let mut bucket_totals: HashMap<APAgingBucket, Decimal> = APAgingBucket::all()
            .into_iter()
            .map(|b| (b, Decimal::ZERO))
            .collect();

        for (vendor_id, invoices) in vendor_invoices {
            let vendor_name = invoices
                .first()
                .map(|i| i.vendor_name.clone())
                .unwrap_or_default();

            let aging = VendorAging::from_invoices(vendor_id, vendor_name, &invoices, as_of_date);

            for (bucket, amount) in &aging.bucket_amounts {
                *bucket_totals
                    .get_mut(bucket)
                    .expect("bucket initialized in map") += amount;
            }

            vendor_details.push(aging);
        }

        vendor_details.sort_by(|a, b| b.total_balance.cmp(&a.total_balance));

        let total_ap_balance: Decimal = bucket_totals.values().sum();
        let total_current = bucket_totals
            .get(&APAgingBucket::Current)
            .copied()
            .unwrap_or_default();
        let total_overdue = total_ap_balance - total_current;

        Self {
            company_code,
            as_of_date,
            vendor_details,
            bucket_totals,
            total_ap_balance,
            total_current,
            total_overdue,
            generated_at: chrono::Utc::now(),
        }
    }
}

/// Aging detail for a single vendor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorAging {
    /// Vendor ID.
    pub vendor_id: String,
    /// Vendor name.
    pub vendor_name: String,
    /// Total balance.
    pub total_balance: Decimal,
    /// Amounts by bucket.
    pub bucket_amounts: HashMap<APAgingBucket, Decimal>,
    /// Invoice count by bucket.
    pub invoice_counts: HashMap<APAgingBucket, u32>,
    /// Oldest invoice date.
    pub oldest_invoice_date: Option<NaiveDate>,
    /// Weighted average days outstanding.
    pub weighted_avg_days: Decimal,
}

impl VendorAging {
    /// Creates vendor aging from invoices.
    pub fn from_invoices(
        vendor_id: String,
        vendor_name: String,
        invoices: &[&APInvoice],
        as_of_date: NaiveDate,
    ) -> Self {
        let mut bucket_amounts: HashMap<APAgingBucket, Decimal> = APAgingBucket::all()
            .into_iter()
            .map(|b| (b, Decimal::ZERO))
            .collect();
        let mut invoice_counts: HashMap<APAgingBucket, u32> =
            APAgingBucket::all().into_iter().map(|b| (b, 0)).collect();

        let mut total_days_weighted = Decimal::ZERO;
        let mut total_balance = Decimal::ZERO;
        let mut oldest_date: Option<NaiveDate> = None;

        for invoice in invoices {
            let days_overdue = invoice.days_overdue(as_of_date);
            let bucket = APAgingBucket::from_days_overdue(days_overdue);
            let amount = invoice.amount_remaining;

            *bucket_amounts
                .get_mut(&bucket)
                .expect("bucket initialized in map") += amount;
            *invoice_counts
                .get_mut(&bucket)
                .expect("bucket initialized in map") += 1;
            total_balance += amount;

            let days_outstanding = (as_of_date - invoice.invoice_date).num_days();
            total_days_weighted += Decimal::from(days_outstanding) * amount;

            if oldest_date.is_none_or(|d| invoice.invoice_date < d) {
                oldest_date = Some(invoice.invoice_date);
            }
        }

        let weighted_avg_days = if total_balance > Decimal::ZERO {
            (total_days_weighted / total_balance).round_dp(1)
        } else {
            Decimal::ZERO
        };

        Self {
            vendor_id,
            vendor_name,
            total_balance,
            bucket_amounts,
            invoice_counts,
            oldest_invoice_date: oldest_date,
            weighted_avg_days,
        }
    }
}

/// Cash flow forecast for AP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APCashForecast {
    /// Company code.
    pub company_code: String,
    /// Forecast start date.
    pub start_date: NaiveDate,
    /// Forecast end date.
    pub end_date: NaiveDate,
    /// Daily forecast.
    pub daily_forecast: Vec<DailyForecast>,
    /// Weekly summary.
    pub weekly_summary: Vec<WeeklyForecast>,
    /// Total forecasted outflow.
    pub total_outflow: Decimal,
    /// Total discount opportunity.
    pub total_discount_opportunity: Decimal,
    /// Generated timestamp.
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl APCashForecast {
    /// Creates a cash forecast from invoices.
    pub fn from_invoices(
        company_code: String,
        invoices: &[APInvoice],
        start_date: NaiveDate,
        end_date: NaiveDate,
        include_discounts: bool,
    ) -> Self {
        let open_invoices: Vec<_> = invoices
            .iter()
            .filter(|i| {
                i.company_code == company_code
                    && matches!(
                        i.status,
                        SubledgerDocumentStatus::Open | SubledgerDocumentStatus::PartiallyCleared
                    )
                    && i.due_date >= start_date
                    && i.due_date <= end_date
            })
            .collect();

        // Build daily forecast
        let mut daily_map: HashMap<NaiveDate, DailyForecast> = HashMap::new();
        let mut total_outflow = Decimal::ZERO;
        let mut total_discount = Decimal::ZERO;

        for invoice in open_invoices {
            let amount = invoice.amount_remaining;
            let discount = if include_discounts {
                invoice.available_discount(start_date)
            } else {
                Decimal::ZERO
            };

            let entry = daily_map
                .entry(invoice.due_date)
                .or_insert_with(|| DailyForecast {
                    date: invoice.due_date,
                    amount_due: Decimal::ZERO,
                    invoice_count: 0,
                    discount_available: Decimal::ZERO,
                    vendor_count: 0,
                    vendors: Vec::new(),
                });

            entry.amount_due += amount;
            entry.invoice_count += 1;
            entry.discount_available += discount;
            if !entry.vendors.contains(&invoice.vendor_id) {
                entry.vendors.push(invoice.vendor_id.clone());
                entry.vendor_count += 1;
            }

            total_outflow += amount;
            total_discount += discount;
        }

        // Convert to sorted vector
        let mut daily_forecast: Vec<DailyForecast> = daily_map.into_values().collect();
        daily_forecast.sort_by_key(|d| d.date);

        // Build weekly summary
        let weekly_summary = Self::build_weekly_summary(&daily_forecast);

        Self {
            company_code,
            start_date,
            end_date,
            daily_forecast,
            weekly_summary,
            total_outflow,
            total_discount_opportunity: total_discount,
            generated_at: chrono::Utc::now(),
        }
    }

    /// Builds weekly summary from daily forecast.
    fn build_weekly_summary(daily: &[DailyForecast]) -> Vec<WeeklyForecast> {
        let mut weekly: HashMap<NaiveDate, WeeklyForecast> = HashMap::new();

        for day in daily {
            // Get Monday of the week
            let weekday = day.date.weekday().num_days_from_monday();
            let week_start = day.date - chrono::Duration::days(weekday as i64);

            let entry = weekly.entry(week_start).or_insert_with(|| WeeklyForecast {
                week_start,
                week_end: week_start + chrono::Duration::days(6),
                amount_due: Decimal::ZERO,
                invoice_count: 0,
                discount_available: Decimal::ZERO,
            });

            entry.amount_due += day.amount_due;
            entry.invoice_count += day.invoice_count;
            entry.discount_available += day.discount_available;
        }

        let mut result: Vec<WeeklyForecast> = weekly.into_values().collect();
        result.sort_by_key(|w| w.week_start);
        result
    }
}

/// Daily forecast entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyForecast {
    /// Date.
    pub date: NaiveDate,
    /// Amount due.
    pub amount_due: Decimal,
    /// Invoice count.
    pub invoice_count: u32,
    /// Discount available if paid today.
    pub discount_available: Decimal,
    /// Number of vendors.
    pub vendor_count: u32,
    /// Vendor IDs.
    pub vendors: Vec<String>,
}

/// Weekly forecast summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyForecast {
    /// Week start date (Monday).
    pub week_start: NaiveDate,
    /// Week end date (Sunday).
    pub week_end: NaiveDate,
    /// Amount due.
    pub amount_due: Decimal,
    /// Invoice count.
    pub invoice_count: u32,
    /// Discount available.
    pub discount_available: Decimal,
}

/// DPO (Days Payable Outstanding) calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DPOCalculation {
    /// Company code.
    pub company_code: String,
    /// Calculation period start.
    pub period_start: NaiveDate,
    /// Calculation period end.
    pub period_end: NaiveDate,
    /// Average AP balance.
    pub average_ap: Decimal,
    /// Total COGS/purchases for period.
    pub total_cogs: Decimal,
    /// DPO result.
    pub dpo_days: Decimal,
    /// Prior period DPO for comparison.
    pub prior_period_dpo: Option<Decimal>,
    /// DPO change.
    pub dpo_change: Option<Decimal>,
}

impl DPOCalculation {
    /// Calculates DPO.
    pub fn calculate(
        company_code: String,
        period_start: NaiveDate,
        period_end: NaiveDate,
        beginning_ap: Decimal,
        ending_ap: Decimal,
        total_cogs: Decimal,
    ) -> Self {
        let average_ap = (beginning_ap + ending_ap) / dec!(2);
        let days_in_period = (period_end - period_start).num_days();

        let dpo_days = if total_cogs > Decimal::ZERO {
            (average_ap / total_cogs * Decimal::from(days_in_period)).round_dp(1)
        } else {
            Decimal::ZERO
        };

        Self {
            company_code,
            period_start,
            period_end,
            average_ap,
            total_cogs,
            dpo_days,
            prior_period_dpo: None,
            dpo_change: None,
        }
    }

    /// Sets prior period comparison.
    pub fn with_prior_period(mut self, prior_dpo: Decimal) -> Self {
        self.prior_period_dpo = Some(prior_dpo);
        self.dpo_change = Some(self.dpo_days - prior_dpo);
        self
    }
}

/// Payment optimization result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentOptimization {
    /// Analysis date.
    pub analysis_date: NaiveDate,
    /// Available cash for payments.
    pub available_cash: Decimal,
    /// Recommended payments.
    pub recommended_payments: Vec<OptimizedPayment>,
    /// Total payment amount.
    pub total_payment: Decimal,
    /// Total discount captured.
    pub discount_captured: Decimal,
    /// Effective discount rate.
    pub effective_discount_rate: Decimal,
    /// Unpaid invoices.
    pub deferred_invoices: Vec<DeferredInvoice>,
}

impl PaymentOptimization {
    /// Optimizes payments to maximize discount capture.
    pub fn optimize(
        invoices: &[APInvoice],
        available_cash: Decimal,
        analysis_date: NaiveDate,
        company_code: &str,
    ) -> Self {
        let mut open_invoices: Vec<_> = invoices
            .iter()
            .filter(|i| {
                i.company_code == company_code
                    && i.status == SubledgerDocumentStatus::Open
                    && i.is_payable()
            })
            .collect();

        // Sort by discount opportunity (highest discount rate first)
        open_invoices.sort_by(|a, b| {
            let a_discount_rate = if a.amount_remaining > Decimal::ZERO {
                a.available_discount(analysis_date) / a.amount_remaining
            } else {
                Decimal::ZERO
            };
            let b_discount_rate = if b.amount_remaining > Decimal::ZERO {
                b.available_discount(analysis_date) / b.amount_remaining
            } else {
                Decimal::ZERO
            };
            b_discount_rate.cmp(&a_discount_rate)
        });

        let mut remaining_cash = available_cash;
        let mut recommended_payments = Vec::new();
        let mut deferred_invoices = Vec::new();
        let mut total_payment = Decimal::ZERO;
        let mut discount_captured = Decimal::ZERO;

        for invoice in open_invoices {
            let discount = invoice.available_discount(analysis_date);
            let payment_amount = invoice.amount_remaining - discount;

            if payment_amount <= remaining_cash {
                recommended_payments.push(OptimizedPayment {
                    vendor_id: invoice.vendor_id.clone(),
                    vendor_name: invoice.vendor_name.clone(),
                    invoice_number: invoice.invoice_number.clone(),
                    invoice_amount: invoice.amount_remaining,
                    payment_amount,
                    discount,
                    due_date: invoice.due_date,
                    priority: PaymentPriority::from_discount(discount, invoice.amount_remaining),
                });

                total_payment += payment_amount;
                discount_captured += discount;
                remaining_cash -= payment_amount;
            } else {
                deferred_invoices.push(DeferredInvoice {
                    vendor_id: invoice.vendor_id.clone(),
                    invoice_number: invoice.invoice_number.clone(),
                    amount: invoice.amount_remaining,
                    due_date: invoice.due_date,
                    discount_lost: discount,
                });
            }
        }

        let effective_discount_rate = if total_payment > Decimal::ZERO {
            (discount_captured / (total_payment + discount_captured) * dec!(100)).round_dp(2)
        } else {
            Decimal::ZERO
        };

        Self {
            analysis_date,
            available_cash,
            recommended_payments,
            total_payment,
            discount_captured,
            effective_discount_rate,
            deferred_invoices,
        }
    }
}

/// An optimized payment recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedPayment {
    /// Vendor ID.
    pub vendor_id: String,
    /// Vendor name.
    pub vendor_name: String,
    /// Invoice number.
    pub invoice_number: String,
    /// Original invoice amount.
    pub invoice_amount: Decimal,
    /// Recommended payment amount.
    pub payment_amount: Decimal,
    /// Discount to capture.
    pub discount: Decimal,
    /// Due date.
    pub due_date: NaiveDate,
    /// Payment priority.
    pub priority: PaymentPriority,
}

/// Payment priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentPriority {
    /// High priority (high discount available).
    High,
    /// Medium priority.
    Medium,
    /// Low priority.
    Low,
}

impl PaymentPriority {
    /// Determines priority from discount percentage.
    pub fn from_discount(discount: Decimal, amount: Decimal) -> Self {
        if amount <= Decimal::ZERO {
            return PaymentPriority::Low;
        }
        let rate = discount / amount * dec!(100);
        if rate >= dec!(2) {
            PaymentPriority::High
        } else if rate >= dec!(1) {
            PaymentPriority::Medium
        } else {
            PaymentPriority::Low
        }
    }
}

/// An invoice deferred for later payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredInvoice {
    /// Vendor ID.
    pub vendor_id: String,
    /// Invoice number.
    pub invoice_number: String,
    /// Amount.
    pub amount: Decimal,
    /// Due date.
    pub due_date: NaiveDate,
    /// Discount that will be lost.
    pub discount_lost: Decimal,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::subledger::PaymentTerms;

    fn create_test_invoices() -> Vec<APInvoice> {
        vec![
            {
                let mut inv = APInvoice::new(
                    "AP001".to_string(),
                    "V001".to_string(),
                    "1000".to_string(),
                    "VEND001".to_string(),
                    "Vendor A".to_string(),
                    NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                    PaymentTerms::two_ten_net_30(),
                    "USD".to_string(),
                );
                inv.amount_remaining = dec!(1000);
                inv
            },
            {
                let mut inv = APInvoice::new(
                    "AP002".to_string(),
                    "V002".to_string(),
                    "1000".to_string(),
                    "VEND001".to_string(),
                    "Vendor A".to_string(),
                    NaiveDate::from_ymd_opt(2023, 12, 1).unwrap(),
                    PaymentTerms::net_30(),
                    "USD".to_string(),
                );
                inv.amount_remaining = dec!(500);
                inv
            },
        ]
    }

    #[test]
    fn test_ap_aging_report() {
        let invoices = create_test_invoices();
        let as_of_date = NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();

        let report = APAgingReport::from_invoices("1000".to_string(), &invoices, as_of_date);

        assert_eq!(report.total_ap_balance, dec!(1500));
    }

    #[test]
    fn test_cash_forecast() {
        let invoices = create_test_invoices();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let forecast = APCashForecast::from_invoices(
            "1000".to_string(),
            &invoices,
            start_date,
            end_date,
            true,
        );

        assert!(forecast.total_outflow > Decimal::ZERO);
    }

    #[test]
    fn test_dpo_calculation() {
        let dpo = DPOCalculation::calculate(
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            dec!(50_000),
            dec!(60_000),
            dec!(300_000),
        );

        // Average AP = 55,000, COGS = 300,000, Days = 31
        // DPO = (55,000 / 300,000) * 31 = 5.68
        assert!(dpo.dpo_days > Decimal::ZERO);
    }
}
