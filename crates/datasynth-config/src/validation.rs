//! Configuration validation.

use crate::schema::GeneratorConfig;
use datasynth_core::error::{SynthError, SynthResult};

/// Maximum allowed period in months (10 years).
const MAX_PERIOD_MONTHS: u32 = 120;

/// Validate a generator configuration.
pub fn validate_config(config: &GeneratorConfig) -> SynthResult<()> {
    validate_global_settings(config)?;
    validate_companies(config)?;
    validate_transactions(config)?;
    validate_output(config)?;
    validate_fraud(config)?;
    validate_internal_controls(config)?;
    validate_approval(config)?;
    validate_master_data(config)?;
    validate_document_flows(config)?;
    validate_intercompany(config)?;
    validate_balance(config)?;
    validate_accounting_standards(config)?;
    validate_audit_standards(config)?;
    validate_distributions(config)?;
    Ok(())
}

/// Validate global settings.
fn validate_global_settings(config: &GeneratorConfig) -> SynthResult<()> {
    if config.global.period_months == 0 {
        return Err(SynthError::validation(
            "period_months must be greater than 0",
        ));
    }
    if config.global.period_months > MAX_PERIOD_MONTHS {
        return Err(SynthError::validation(format!(
            "period_months must be at most {} (10 years), got {}",
            MAX_PERIOD_MONTHS, config.global.period_months
        )));
    }
    Ok(())
}

/// Validate company configuration.
fn validate_companies(config: &GeneratorConfig) -> SynthResult<()> {
    if config.companies.is_empty() {
        return Err(SynthError::validation(
            "At least one company must be configured",
        ));
    }

    for company in &config.companies {
        if company.code.is_empty() {
            return Err(SynthError::validation("Company code cannot be empty"));
        }
        if company.currency.len() != 3 {
            return Err(SynthError::validation(format!(
                "Invalid currency code '{}' for company '{}'",
                company.currency, company.code
            )));
        }
        if company.volume_weight < 0.0 {
            return Err(SynthError::validation(format!(
                "volume_weight must be non-negative for company '{}'",
                company.code
            )));
        }
    }
    Ok(())
}

/// Validate transaction configuration.
fn validate_transactions(config: &GeneratorConfig) -> SynthResult<()> {
    // Validate line item distribution
    let line_dist = &config.transactions.line_item_distribution;
    if let Err(e) = line_dist.validate() {
        return Err(SynthError::validation(e));
    }

    // Validate source distribution sums to ~1.0
    let source_sum = config.transactions.source_distribution.manual
        + config.transactions.source_distribution.automated
        + config.transactions.source_distribution.recurring
        + config.transactions.source_distribution.adjustment;
    if (source_sum - 1.0).abs() > 0.01 {
        return Err(SynthError::validation(format!(
            "Source distribution must sum to 1.0, got {}",
            source_sum
        )));
    }

    // Validate business process weights
    let bp_sum = config.business_processes.o2c_weight
        + config.business_processes.p2p_weight
        + config.business_processes.r2r_weight
        + config.business_processes.h2r_weight
        + config.business_processes.a2r_weight;
    if (bp_sum - 1.0).abs() > 0.01 {
        return Err(SynthError::validation(format!(
            "Business process weights must sum to 1.0, got {}",
            bp_sum
        )));
    }

    // Validate Benford tolerance
    let tolerance = config.transactions.benford.tolerance;
    if !(0.0..=1.0).contains(&tolerance) {
        return Err(SynthError::validation(format!(
            "benford.tolerance must be between 0.0 and 1.0, got {}",
            tolerance
        )));
    }

    Ok(())
}

/// Validate output configuration.
fn validate_output(config: &GeneratorConfig) -> SynthResult<()> {
    let level = config.output.compression.level;
    if config.output.compression.enabled && !(1..=9).contains(&level) {
        return Err(SynthError::validation(format!(
            "compression.level must be between 1 and 9, got {}",
            level
        )));
    }

    if config.output.batch_size == 0 {
        return Err(SynthError::validation("batch_size must be greater than 0"));
    }

    Ok(())
}

/// Validate fraud configuration.
fn validate_fraud(config: &GeneratorConfig) -> SynthResult<()> {
    if !config.fraud.enabled {
        return Ok(());
    }

    if config.fraud.fraud_rate < 0.0 || config.fraud.fraud_rate > 1.0 {
        return Err(SynthError::validation(
            "fraud_rate must be between 0.0 and 1.0",
        ));
    }

    if config.fraud.clustering_factor < 0.0 {
        return Err(SynthError::validation(
            "clustering_factor must be non-negative",
        ));
    }

    // Validate approval thresholds are in ascending order
    let thresholds = &config.fraud.approval_thresholds;
    for i in 1..thresholds.len() {
        if thresholds[i] <= thresholds[i - 1] {
            return Err(SynthError::validation(format!(
                "fraud.approval_thresholds must be in strictly ascending order: {} is not greater than {}",
                thresholds[i], thresholds[i - 1]
            )));
        }
    }

    // Validate fraud type distribution sums to ~1.0
    let dist = &config.fraud.fraud_type_distribution;
    let sum = dist.suspense_account_abuse
        + dist.fictitious_transaction
        + dist.revenue_manipulation
        + dist.expense_capitalization
        + dist.split_transaction
        + dist.timing_anomaly
        + dist.unauthorized_access
        + dist.duplicate_payment;
    if (sum - 1.0).abs() > 0.01 {
        return Err(SynthError::validation(format!(
            "fraud_type_distribution must sum to 1.0, got {}",
            sum
        )));
    }

    Ok(())
}

/// Validate internal controls configuration.
fn validate_internal_controls(config: &GeneratorConfig) -> SynthResult<()> {
    if !config.internal_controls.enabled {
        return Ok(());
    }

    let exception_rate = config.internal_controls.exception_rate;
    if !(0.0..=1.0).contains(&exception_rate) {
        return Err(SynthError::validation(format!(
            "exception_rate must be between 0.0 and 1.0, got {}",
            exception_rate
        )));
    }

    let sod_rate = config.internal_controls.sod_violation_rate;
    if !(0.0..=1.0).contains(&sod_rate) {
        return Err(SynthError::validation(format!(
            "sod_violation_rate must be between 0.0 and 1.0, got {}",
            sod_rate
        )));
    }

    if config.internal_controls.sox_materiality_threshold < 0.0 {
        return Err(SynthError::validation(
            "sox_materiality_threshold must be non-negative",
        ));
    }

    Ok(())
}

/// Validate approval configuration.
fn validate_approval(config: &GeneratorConfig) -> SynthResult<()> {
    if !config.approval.enabled {
        return Ok(());
    }

    if config.approval.auto_approve_threshold < 0.0 {
        return Err(SynthError::validation(
            "auto_approve_threshold must be non-negative",
        ));
    }

    let rejection_rate = config.approval.rejection_rate;
    if !(0.0..=1.0).contains(&rejection_rate) {
        return Err(SynthError::validation(format!(
            "rejection_rate must be between 0.0 and 1.0, got {}",
            rejection_rate
        )));
    }

    let revision_rate = config.approval.revision_rate;
    if !(0.0..=1.0).contains(&revision_rate) {
        return Err(SynthError::validation(format!(
            "revision_rate must be between 0.0 and 1.0, got {}",
            revision_rate
        )));
    }

    // rejection + revision should not exceed 1.0
    if rejection_rate + revision_rate > 1.0 {
        return Err(SynthError::validation(format!(
            "rejection_rate + revision_rate must not exceed 1.0, got {}",
            rejection_rate + revision_rate
        )));
    }

    // Validate approval thresholds are in ascending order by amount
    let thresholds = &config.approval.thresholds;
    for i in 1..thresholds.len() {
        if thresholds[i].amount <= thresholds[i - 1].amount {
            return Err(SynthError::validation(format!(
                "approval.thresholds must have strictly ascending amounts: {} is not greater than {}",
                thresholds[i].amount, thresholds[i - 1].amount
            )));
        }
    }

    Ok(())
}

/// Validate master data configuration.
fn validate_master_data(config: &GeneratorConfig) -> SynthResult<()> {
    // Vendor config
    let vendor_ic = config.master_data.vendors.intercompany_percent;
    if !(0.0..=1.0).contains(&vendor_ic) {
        return Err(SynthError::validation(format!(
            "vendors.intercompany_percent must be between 0.0 and 1.0, got {}",
            vendor_ic
        )));
    }

    // Customer config
    let customer_ic = config.master_data.customers.intercompany_percent;
    if !(0.0..=1.0).contains(&customer_ic) {
        return Err(SynthError::validation(format!(
            "customers.intercompany_percent must be between 0.0 and 1.0, got {}",
            customer_ic
        )));
    }

    // Material config
    let bom_percent = config.master_data.materials.bom_percent;
    if !(0.0..=1.0).contains(&bom_percent) {
        return Err(SynthError::validation(format!(
            "materials.bom_percent must be between 0.0 and 1.0, got {}",
            bom_percent
        )));
    }

    // Fixed asset config
    let fully_dep = config.master_data.fixed_assets.fully_depreciated_percent;
    if !(0.0..=1.0).contains(&fully_dep) {
        return Err(SynthError::validation(format!(
            "fixed_assets.fully_depreciated_percent must be between 0.0 and 1.0, got {}",
            fully_dep
        )));
    }

    Ok(())
}

/// Validate document flow configuration.
fn validate_document_flows(config: &GeneratorConfig) -> SynthResult<()> {
    // P2P config
    let p2p = &config.document_flows.p2p;
    if p2p.enabled {
        validate_rate("p2p.three_way_match_rate", p2p.three_way_match_rate)?;
        validate_rate("p2p.partial_delivery_rate", p2p.partial_delivery_rate)?;
        validate_rate("p2p.price_variance_rate", p2p.price_variance_rate)?;
        validate_rate("p2p.quantity_variance_rate", p2p.quantity_variance_rate)?;

        if p2p.max_price_variance_percent < 0.0 {
            return Err(SynthError::validation(
                "p2p.max_price_variance_percent must be non-negative",
            ));
        }

        // P2P payment behavior config
        validate_p2p_payment_behavior(&p2p.payment_behavior)?;
    }

    // O2C config
    let o2c = &config.document_flows.o2c;
    if o2c.enabled {
        validate_rate(
            "o2c.credit_check_failure_rate",
            o2c.credit_check_failure_rate,
        )?;
        validate_rate("o2c.partial_shipment_rate", o2c.partial_shipment_rate)?;
        validate_rate("o2c.return_rate", o2c.return_rate)?;
        validate_rate("o2c.bad_debt_rate", o2c.bad_debt_rate)?;

        // Cash discount config
        validate_rate(
            "o2c.cash_discount.eligible_rate",
            o2c.cash_discount.eligible_rate,
        )?;
        validate_rate("o2c.cash_discount.taken_rate", o2c.cash_discount.taken_rate)?;
        validate_rate(
            "o2c.cash_discount.discount_percent",
            o2c.cash_discount.discount_percent,
        )?;

        // O2C payment behavior config
        validate_o2c_payment_behavior(&o2c.payment_behavior)?;
    }

    Ok(())
}

/// Validate P2P payment behavior configuration.
fn validate_p2p_payment_behavior(
    config: &crate::schema::P2PPaymentBehaviorConfig,
) -> SynthResult<()> {
    validate_rate(
        "p2p.payment_behavior.late_payment_rate",
        config.late_payment_rate,
    )?;
    validate_rate(
        "p2p.payment_behavior.partial_payment_rate",
        config.partial_payment_rate,
    )?;
    validate_rate(
        "p2p.payment_behavior.payment_correction_rate",
        config.payment_correction_rate,
    )?;

    // Validate late payment days distribution sums to ~1.0
    let late_dist = &config.late_payment_days_distribution;
    let late_sum = late_dist.slightly_late_1_to_7
        + late_dist.late_8_to_14
        + late_dist.very_late_15_to_30
        + late_dist.severely_late_31_to_60
        + late_dist.extremely_late_over_60;
    if (late_sum - 1.0).abs() > 0.01 {
        return Err(SynthError::validation(format!(
            "p2p.payment_behavior.late_payment_days_distribution must sum to 1.0, got {}",
            late_sum
        )));
    }

    Ok(())
}

/// Validate O2C payment behavior configuration.
fn validate_o2c_payment_behavior(
    config: &crate::schema::O2CPaymentBehaviorConfig,
) -> SynthResult<()> {
    // Validate dunning config
    let dunning = &config.dunning;
    if dunning.enabled {
        validate_rate(
            "o2c.payment_behavior.dunning.dunning_block_rate",
            dunning.dunning_block_rate,
        )?;

        // Validate dunning level days are in ascending order
        if dunning.level_2_days_overdue <= dunning.level_1_days_overdue {
            return Err(SynthError::validation(
                "dunning.level_2_days_overdue must be greater than level_1_days_overdue",
            ));
        }
        if dunning.level_3_days_overdue <= dunning.level_2_days_overdue {
            return Err(SynthError::validation(
                "dunning.level_3_days_overdue must be greater than level_2_days_overdue",
            ));
        }
        if dunning.collection_days_overdue <= dunning.level_3_days_overdue {
            return Err(SynthError::validation(
                "dunning.collection_days_overdue must be greater than level_3_days_overdue",
            ));
        }

        // Validate dunning payment rates sum to ~1.0
        let rates = &dunning.payment_after_dunning_rates;
        let rates_sum = rates.after_level_1
            + rates.after_level_2
            + rates.after_level_3
            + rates.during_collection
            + rates.never_pay;
        if (rates_sum - 1.0).abs() > 0.01 {
            return Err(SynthError::validation(format!(
                "dunning.payment_after_dunning_rates must sum to 1.0, got {}",
                rates_sum
            )));
        }
    }

    // Validate partial payments config
    let partial = &config.partial_payments;
    validate_rate("o2c.payment_behavior.partial_payments.rate", partial.rate)?;
    let partial_dist = &partial.percentage_distribution;
    let partial_sum = partial_dist.pay_25_percent
        + partial_dist.pay_50_percent
        + partial_dist.pay_75_percent
        + partial_dist.pay_random_percent;
    if (partial_sum - 1.0).abs() > 0.01 {
        return Err(SynthError::validation(format!(
            "partial_payments.percentage_distribution must sum to 1.0, got {}",
            partial_sum
        )));
    }

    // Validate short payments config
    let short = &config.short_payments;
    validate_rate("o2c.payment_behavior.short_payments.rate", short.rate)?;
    validate_rate(
        "o2c.payment_behavior.short_payments.max_short_percent",
        short.max_short_percent,
    )?;
    let short_dist = &short.reason_distribution;
    let short_sum = short_dist.pricing_dispute
        + short_dist.quality_issue
        + short_dist.quantity_discrepancy
        + short_dist.unauthorized_deduction
        + short_dist.incorrect_discount;
    if (short_sum - 1.0).abs() > 0.01 {
        return Err(SynthError::validation(format!(
            "short_payments.reason_distribution must sum to 1.0, got {}",
            short_sum
        )));
    }

    // Validate on-account payments config
    validate_rate(
        "o2c.payment_behavior.on_account_payments.rate",
        config.on_account_payments.rate,
    )?;

    // Validate payment corrections config
    let corrections = &config.payment_corrections;
    validate_rate(
        "o2c.payment_behavior.payment_corrections.rate",
        corrections.rate,
    )?;
    let corr_dist = &corrections.type_distribution;
    let corr_sum = corr_dist.nsf
        + corr_dist.chargeback
        + corr_dist.wrong_amount
        + corr_dist.wrong_customer
        + corr_dist.duplicate_payment;
    if (corr_sum - 1.0).abs() > 0.01 {
        return Err(SynthError::validation(format!(
            "payment_corrections.type_distribution must sum to 1.0, got {}",
            corr_sum
        )));
    }

    Ok(())
}

/// Validate intercompany configuration.
fn validate_intercompany(config: &GeneratorConfig) -> SynthResult<()> {
    if !config.intercompany.enabled {
        return Ok(());
    }

    validate_rate(
        "intercompany.ic_transaction_rate",
        config.intercompany.ic_transaction_rate,
    )?;

    if config.intercompany.markup_percent < 0.0 {
        return Err(SynthError::validation(
            "intercompany.markup_percent must be non-negative",
        ));
    }

    // Validate IC transaction type distribution sums to ~1.0
    let dist = &config.intercompany.transaction_type_distribution;
    let sum = dist.goods_sale
        + dist.service_provided
        + dist.loan
        + dist.dividend
        + dist.management_fee
        + dist.royalty
        + dist.cost_sharing;
    if (sum - 1.0).abs() > 0.01 {
        return Err(SynthError::validation(format!(
            "intercompany.transaction_type_distribution must sum to 1.0, got {}",
            sum
        )));
    }

    Ok(())
}

/// Validate balance configuration.
fn validate_balance(config: &GeneratorConfig) -> SynthResult<()> {
    let balance = &config.balance;

    if balance.target_gross_margin < 0.0 || balance.target_gross_margin > 1.0 {
        return Err(SynthError::validation(format!(
            "target_gross_margin must be between 0.0 and 1.0, got {}",
            balance.target_gross_margin
        )));
    }

    if balance.target_current_ratio < 0.0 {
        return Err(SynthError::validation(
            "target_current_ratio must be non-negative",
        ));
    }

    if balance.target_debt_to_equity < 0.0 {
        return Err(SynthError::validation(
            "target_debt_to_equity must be non-negative",
        ));
    }

    Ok(())
}

/// Helper to validate a rate field is between 0.0 and 1.0.
fn validate_rate(field_name: &str, value: f64) -> SynthResult<()> {
    if !(0.0..=1.0).contains(&value) {
        return Err(SynthError::validation(format!(
            "{} must be between 0.0 and 1.0, got {}",
            field_name, value
        )));
    }
    Ok(())
}

/// Validate accounting standards configuration (IFRS, US GAAP).
fn validate_accounting_standards(config: &GeneratorConfig) -> SynthResult<()> {
    let standards = &config.accounting_standards;

    if !standards.enabled {
        return Ok(());
    }

    // Validate revenue recognition settings
    if standards.revenue_recognition.enabled {
        let rev = &standards.revenue_recognition;

        if rev.avg_obligations_per_contract < 1.0 {
            return Err(SynthError::validation(
                "avg_obligations_per_contract must be >= 1.0",
            ));
        }

        validate_rate(
            "revenue_recognition.variable_consideration_rate",
            rev.variable_consideration_rate,
        )?;

        validate_rate(
            "revenue_recognition.over_time_recognition_rate",
            rev.over_time_recognition_rate,
        )?;
    }

    // Validate lease accounting settings
    if standards.leases.enabled {
        let lease = &standards.leases;

        if lease.avg_lease_term_months == 0 {
            return Err(SynthError::validation(
                "lease.avg_lease_term_months must be > 0",
            ));
        }

        validate_rate("lease.finance_lease_percent", lease.finance_lease_percent)?;
        validate_rate("lease.real_estate_percent", lease.real_estate_percent)?;
    }

    // Validate fair value settings
    if standards.fair_value.enabled {
        let fv = &standards.fair_value;

        // Level distributions should sum to approximately 1.0
        let level_sum = fv.level1_percent + fv.level2_percent + fv.level3_percent;
        if (level_sum - 1.0).abs() > 0.01 {
            return Err(SynthError::validation(format!(
                "fair_value level percentages must sum to 1.0, got {}",
                level_sum
            )));
        }

        validate_rate("fair_value.level1_percent", fv.level1_percent)?;
        validate_rate("fair_value.level2_percent", fv.level2_percent)?;
        validate_rate("fair_value.level3_percent", fv.level3_percent)?;
    }

    // Validate impairment settings
    if standards.impairment.enabled {
        let imp = &standards.impairment;

        validate_rate("impairment.impairment_rate", imp.impairment_rate)?;
    }

    Ok(())
}

/// Validate audit standards configuration (ISA, PCAOB, SOX).
fn validate_audit_standards(config: &GeneratorConfig) -> SynthResult<()> {
    let standards = &config.audit_standards;

    if !standards.enabled {
        return Ok(());
    }

    // Validate ISA compliance settings
    if standards.isa_compliance.enabled {
        let valid_levels = ["basic", "standard", "comprehensive"];
        if !valid_levels.contains(&standards.isa_compliance.compliance_level.as_str()) {
            return Err(SynthError::validation(format!(
                "isa_compliance.compliance_level must be one of {:?}, got '{}'",
                valid_levels, standards.isa_compliance.compliance_level
            )));
        }

        let valid_frameworks = ["isa", "pcaob", "dual"];
        if !valid_frameworks.contains(&standards.isa_compliance.framework.as_str()) {
            return Err(SynthError::validation(format!(
                "isa_compliance.framework must be one of {:?}, got '{}'",
                valid_frameworks, standards.isa_compliance.framework
            )));
        }
    }

    // Validate analytical procedures settings
    if standards.analytical_procedures.enabled {
        let ap = &standards.analytical_procedures;

        if ap.procedures_per_account == 0 {
            return Err(SynthError::validation(
                "analytical_procedures.procedures_per_account must be > 0",
            ));
        }

        validate_rate(
            "analytical_procedures.variance_probability",
            ap.variance_probability,
        )?;
    }

    // Validate confirmations settings
    if standards.confirmations.enabled {
        let conf = &standards.confirmations;

        validate_rate(
            "confirmations.positive_response_rate",
            conf.positive_response_rate,
        )?;

        validate_rate("confirmations.exception_rate", conf.exception_rate)?;

        // Positive + non-response + exception should make sense
        let total_rate = conf.positive_response_rate + conf.exception_rate;
        if total_rate > 1.0 {
            return Err(SynthError::validation(
                "confirmations: positive_response_rate + exception_rate cannot exceed 1.0",
            ));
        }
    }

    // Validate opinion settings
    if standards.opinion.enabled {
        let op = &standards.opinion;

        if op.generate_kam && op.average_kam_count == 0 {
            return Err(SynthError::validation(
                "opinion.average_kam_count must be > 0 when generate_kam is true",
            ));
        }
    }

    // Validate SOX settings
    if standards.sox.enabled {
        let sox = &standards.sox;

        if sox.materiality_threshold < 0.0 {
            return Err(SynthError::validation(
                "sox.materiality_threshold must be >= 0",
            ));
        }
    }

    // Validate PCAOB settings
    if standards.pcaob.enabled {
        // PCAOB requires ISA dual framework or PCAOB-only
        if standards.isa_compliance.enabled
            && standards.isa_compliance.framework != "pcaob"
            && standards.isa_compliance.framework != "dual"
        {
            return Err(SynthError::validation(
                "When PCAOB is enabled, ISA framework must be 'pcaob' or 'dual'",
            ));
        }
    }

    Ok(())
}

/// Validate advanced distribution configuration.
fn validate_distributions(config: &GeneratorConfig) -> SynthResult<()> {
    let dist = &config.distributions;

    if !dist.enabled {
        return Ok(());
    }

    // Validate mixture model configuration
    validate_mixture_config(&dist.amounts)?;

    // Validate correlation configuration
    validate_correlation_config(&dist.correlations)?;

    // Validate conditional distributions
    for (i, cond) in dist.conditional.iter().enumerate() {
        validate_conditional_config(cond, i)?;
    }

    // Validate regime changes
    validate_regime_changes(&dist.regime_changes)?;

    // Validate statistical validation settings
    validate_statistical_validation(&dist.validation)?;

    Ok(())
}

/// Validate mixture model configuration.
fn validate_mixture_config(
    config: &crate::schema::MixtureDistributionSchemaConfig,
) -> SynthResult<()> {
    if !config.enabled {
        return Ok(());
    }

    if config.components.is_empty() {
        return Err(SynthError::validation(
            "distributions.amounts.components cannot be empty when enabled",
        ));
    }

    // Validate weights sum to 1.0
    let weight_sum: f64 = config.components.iter().map(|c| c.weight).sum();
    if (weight_sum - 1.0).abs() > 0.01 {
        return Err(SynthError::validation(format!(
            "distributions.amounts.components weights must sum to 1.0, got {}",
            weight_sum
        )));
    }

    // Validate individual components
    for (i, comp) in config.components.iter().enumerate() {
        if comp.weight < 0.0 || comp.weight > 1.0 {
            return Err(SynthError::validation(format!(
                "distributions.amounts.components[{}].weight must be between 0.0 and 1.0, got {}",
                i, comp.weight
            )));
        }

        if comp.sigma <= 0.0 {
            return Err(SynthError::validation(format!(
                "distributions.amounts.components[{}].sigma must be positive, got {}",
                i, comp.sigma
            )));
        }
    }

    // Validate min/max values
    if config.min_value < 0.0 {
        return Err(SynthError::validation(
            "distributions.amounts.min_value must be non-negative",
        ));
    }

    if let Some(max) = config.max_value {
        if max <= config.min_value {
            return Err(SynthError::validation(format!(
                "distributions.amounts.max_value ({}) must be greater than min_value ({})",
                max, config.min_value
            )));
        }
    }

    Ok(())
}

/// Validate correlation configuration.
fn validate_correlation_config(config: &crate::schema::CorrelationSchemaConfig) -> SynthResult<()> {
    if !config.enabled {
        return Ok(());
    }

    let n = config.fields.len();
    if n < 2 {
        return Err(SynthError::validation(
            "distributions.correlations.fields must have at least 2 fields",
        ));
    }

    // Check matrix size
    let expected_matrix_size = n * (n - 1) / 2;
    if config.matrix.len() != expected_matrix_size {
        return Err(SynthError::validation(format!(
            "distributions.correlations.matrix must have {} elements for {} fields, got {}",
            expected_matrix_size,
            n,
            config.matrix.len()
        )));
    }

    // Validate correlation values are in [-1, 1]
    for (i, &r) in config.matrix.iter().enumerate() {
        if !(-1.0..=1.0).contains(&r) {
            return Err(SynthError::validation(format!(
                "distributions.correlations.matrix[{}] must be in [-1, 1], got {}",
                i, r
            )));
        }
    }

    // Validate expected correlations
    for expected in &config.expected_correlations {
        if !(-1.0..=1.0).contains(&expected.expected_r) {
            return Err(SynthError::validation(format!(
                "expected_correlation for ({}, {}): expected_r must be in [-1, 1], got {}",
                expected.field1, expected.field2, expected.expected_r
            )));
        }
        if expected.tolerance <= 0.0 || expected.tolerance > 1.0 {
            return Err(SynthError::validation(format!(
                "expected_correlation for ({}, {}): tolerance must be in (0, 1], got {}",
                expected.field1, expected.field2, expected.tolerance
            )));
        }
    }

    Ok(())
}

/// Validate conditional distribution configuration.
fn validate_conditional_config(
    config: &crate::schema::ConditionalDistributionSchemaConfig,
    index: usize,
) -> SynthResult<()> {
    if config.output_field.is_empty() {
        return Err(SynthError::validation(format!(
            "distributions.conditional[{}].output_field cannot be empty",
            index
        )));
    }

    if config.input_field.is_empty() {
        return Err(SynthError::validation(format!(
            "distributions.conditional[{}].input_field cannot be empty",
            index
        )));
    }

    // Validate breakpoints are in ascending order
    for i in 1..config.breakpoints.len() {
        if config.breakpoints[i].threshold <= config.breakpoints[i - 1].threshold {
            return Err(SynthError::validation(format!(
                "distributions.conditional[{}].breakpoints must be in ascending order: {} is not greater than {}",
                index, config.breakpoints[i].threshold, config.breakpoints[i - 1].threshold
            )));
        }
    }

    // Validate min/max constraints
    if let (Some(min), Some(max)) = (config.min_value, config.max_value) {
        if max <= min {
            return Err(SynthError::validation(format!(
                "distributions.conditional[{}].max_value ({}) must be greater than min_value ({})",
                index, max, min
            )));
        }
    }

    Ok(())
}

/// Validate regime change configuration.
fn validate_regime_changes(config: &crate::schema::RegimeChangeSchemaConfig) -> SynthResult<()> {
    if !config.enabled {
        return Ok(());
    }

    // Validate regime change events
    for (i, change) in config.changes.iter().enumerate() {
        // Validate date format (basic check)
        if change.date.is_empty() {
            return Err(SynthError::validation(format!(
                "distributions.regime_changes.changes[{}].date cannot be empty",
                i
            )));
        }

        // Validate effects have positive multipliers
        for (j, effect) in change.effects.iter().enumerate() {
            if effect.multiplier < 0.0 {
                return Err(SynthError::validation(format!(
                    "distributions.regime_changes.changes[{}].effects[{}].multiplier must be non-negative, got {}",
                    i, j, effect.multiplier
                )));
            }
        }
    }

    // Validate economic cycle if present
    if let Some(ref cycle) = config.economic_cycle {
        if cycle.enabled {
            if cycle.period_months == 0 {
                return Err(SynthError::validation(
                    "distributions.regime_changes.economic_cycle.period_months must be > 0",
                ));
            }

            if cycle.amplitude < 0.0 || cycle.amplitude > 1.0 {
                return Err(SynthError::validation(format!(
                    "distributions.regime_changes.economic_cycle.amplitude must be in [0, 1], got {}",
                    cycle.amplitude
                )));
            }

            // Validate recession periods
            for (i, recession) in cycle.recessions.iter().enumerate() {
                if recession.duration_months == 0 {
                    return Err(SynthError::validation(format!(
                        "distributions.regime_changes.economic_cycle.recessions[{}].duration_months must be > 0",
                        i
                    )));
                }

                if recession.severity < 0.0 || recession.severity > 1.0 {
                    return Err(SynthError::validation(format!(
                        "distributions.regime_changes.economic_cycle.recessions[{}].severity must be in [0, 1], got {}",
                        i, recession.severity
                    )));
                }
            }
        }
    }

    // Validate parameter drifts
    for (i, drift) in config.parameter_drifts.iter().enumerate() {
        if drift.parameter.is_empty() {
            return Err(SynthError::validation(format!(
                "distributions.regime_changes.parameter_drifts[{}].parameter cannot be empty",
                i
            )));
        }

        if let Some(end) = drift.end_period {
            if end <= drift.start_period {
                return Err(SynthError::validation(format!(
                    "distributions.regime_changes.parameter_drifts[{}].end_period ({}) must be > start_period ({})",
                    i, end, drift.start_period
                )));
            }
        }
    }

    Ok(())
}

/// Validate statistical validation configuration.
fn validate_statistical_validation(
    config: &crate::schema::StatisticalValidationSchemaConfig,
) -> SynthResult<()> {
    if !config.enabled {
        return Ok(());
    }

    // Validate test configurations
    for (i, test) in config.tests.iter().enumerate() {
        match test {
            crate::schema::StatisticalTestConfig::BenfordFirstDigit {
                threshold_mad,
                warning_mad,
            } => {
                if *threshold_mad <= 0.0 {
                    return Err(SynthError::validation(format!(
                        "distributions.validation.tests[{}].threshold_mad must be positive",
                        i
                    )));
                }
                if *warning_mad <= 0.0 {
                    return Err(SynthError::validation(format!(
                        "distributions.validation.tests[{}].warning_mad must be positive",
                        i
                    )));
                }
            }
            crate::schema::StatisticalTestConfig::DistributionFit {
                ks_significance, ..
            } => {
                if *ks_significance <= 0.0 || *ks_significance >= 1.0 {
                    return Err(SynthError::validation(format!(
                        "distributions.validation.tests[{}].ks_significance must be in (0, 1), got {}",
                        i, ks_significance
                    )));
                }
            }
            crate::schema::StatisticalTestConfig::ChiSquared { bins, significance } => {
                if *bins < 2 {
                    return Err(SynthError::validation(format!(
                        "distributions.validation.tests[{}].bins must be >= 2, got {}",
                        i, bins
                    )));
                }
                if *significance <= 0.0 || *significance >= 1.0 {
                    return Err(SynthError::validation(format!(
                        "distributions.validation.tests[{}].significance must be in (0, 1), got {}",
                        i, significance
                    )));
                }
            }
            crate::schema::StatisticalTestConfig::AndersonDarling { significance, .. } => {
                if *significance <= 0.0 || *significance >= 1.0 {
                    return Err(SynthError::validation(format!(
                        "distributions.validation.tests[{}].significance must be in (0, 1), got {}",
                        i, significance
                    )));
                }
            }
            crate::schema::StatisticalTestConfig::CorrelationCheck {
                expected_correlations,
            } => {
                for expected in expected_correlations {
                    if !(-1.0..=1.0).contains(&expected.expected_r) {
                        return Err(SynthError::validation(format!(
                            "distributions.validation.tests[{}]: expected_r must be in [-1, 1]",
                            i
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets::{create_preset, demo_preset, stress_test_preset};
    use crate::schema::*;
    use datasynth_core::models::{CoAComplexity, IndustrySector};

    /// Helper to create a minimal valid config for testing.
    fn minimal_valid_config() -> GeneratorConfig {
        GeneratorConfig {
            global: GlobalConfig {
                seed: Some(42),
                industry: IndustrySector::Manufacturing,
                start_date: "2024-01-01".to_string(),
                period_months: 3,
                group_currency: "USD".to_string(),
                parallel: true,
                worker_threads: 0,
                memory_limit_mb: 0,
            },
            companies: vec![CompanyConfig {
                code: "TEST".to_string(),
                name: "Test Company".to_string(),
                currency: "USD".to_string(),
                country: "US".to_string(),
                fiscal_year_variant: "K4".to_string(),
                annual_transaction_volume: TransactionVolume::TenK,
                volume_weight: 1.0,
            }],
            chart_of_accounts: ChartOfAccountsConfig {
                complexity: CoAComplexity::Small,
                industry_specific: true,
                custom_accounts: None,
                min_hierarchy_depth: 2,
                max_hierarchy_depth: 5,
            },
            transactions: TransactionConfig::default(),
            output: OutputConfig::default(),
            fraud: FraudConfig::default(),
            internal_controls: InternalControlsConfig::default(),
            business_processes: BusinessProcessConfig::default(),
            user_personas: UserPersonaConfig::default(),
            templates: TemplateConfig::default(),
            approval: ApprovalConfig::default(),
            departments: DepartmentConfig::default(),
            master_data: MasterDataConfig::default(),
            document_flows: DocumentFlowConfig::default(),
            intercompany: IntercompanyConfig::default(),
            balance: BalanceConfig::default(),
            ocpm: OcpmConfig::default(),
            audit: AuditGenerationConfig::default(),
            banking: datasynth_banking::BankingConfig::default(),
            data_quality: DataQualitySchemaConfig::default(),
            scenario: ScenarioConfig::default(),
            temporal: TemporalDriftConfig::default(),
            graph_export: GraphExportConfig::default(),
            streaming: StreamingSchemaConfig::default(),
            rate_limit: RateLimitSchemaConfig::default(),
            temporal_attributes: TemporalAttributeSchemaConfig::default(),
            relationships: RelationshipSchemaConfig::default(),
            accounting_standards: AccountingStandardsConfig::default(),
            audit_standards: AuditStandardsConfig::default(),
            distributions: AdvancedDistributionConfig::default(),
        }
    }

    // ==========================================================================
    // Period Months Validation Tests
    // ==========================================================================

    #[test]
    fn test_valid_period_months() {
        let config = minimal_valid_config();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_zero_period_months_rejected() {
        let mut config = minimal_valid_config();
        config.global.period_months = 0;
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("period_months"));
    }

    #[test]
    fn test_large_period_months_accepted() {
        let mut config = minimal_valid_config();
        config.global.period_months = 120; // 10 years - maximum allowed
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_period_months_exceeds_max_rejected() {
        let mut config = minimal_valid_config();
        config.global.period_months = 121; // Exceeds 10 year max
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("period_months"));
    }

    // ==========================================================================
    // Company Validation Tests
    // ==========================================================================

    #[test]
    fn test_empty_companies_rejected() {
        let mut config = minimal_valid_config();
        config.companies.clear();
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("company"));
    }

    #[test]
    fn test_empty_company_code_rejected() {
        let mut config = minimal_valid_config();
        config.companies[0].code = "".to_string();
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Company code"));
    }

    #[test]
    fn test_invalid_currency_code_rejected() {
        let mut config = minimal_valid_config();
        config.companies[0].currency = "US".to_string(); // 2 chars, not 3
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("currency"));
    }

    #[test]
    fn test_long_currency_code_rejected() {
        let mut config = minimal_valid_config();
        config.companies[0].currency = "USDD".to_string(); // 4 chars
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("currency"));
    }

    #[test]
    fn test_multiple_companies_validated() {
        let mut config = minimal_valid_config();
        config.companies.push(CompanyConfig {
            code: "SUB1".to_string(),
            name: "Subsidiary 1".to_string(),
            currency: "EUR".to_string(),
            country: "DE".to_string(),
            fiscal_year_variant: "K4".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 0.5,
        });
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_second_company_invalid_currency_rejected() {
        let mut config = minimal_valid_config();
        config.companies.push(CompanyConfig {
            code: "SUB1".to_string(),
            name: "Subsidiary 1".to_string(),
            currency: "EU".to_string(), // Invalid
            country: "DE".to_string(),
            fiscal_year_variant: "K4".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 0.5,
        });
        let result = validate_config(&config);
        assert!(result.is_err());
    }

    // ==========================================================================
    // Source Distribution Validation Tests
    // ==========================================================================

    #[test]
    fn test_valid_source_distribution() {
        let config = minimal_valid_config();
        // Default source distribution sums to 1.0
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_source_distribution_not_summing_to_one_rejected() {
        let mut config = minimal_valid_config();
        config.transactions.source_distribution = SourceDistribution {
            manual: 0.5,
            automated: 0.5,
            recurring: 0.5, // Sum = 1.6
            adjustment: 0.1,
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Source distribution"));
    }

    #[test]
    fn test_source_distribution_slightly_off_accepted() {
        let mut config = minimal_valid_config();
        // Within 0.01 tolerance
        config.transactions.source_distribution = SourceDistribution {
            manual: 0.20,
            automated: 0.70,
            recurring: 0.07,
            adjustment: 0.025, // Sum = 0.995, within tolerance
        };
        assert!(validate_config(&config).is_ok());
    }

    // ==========================================================================
    // Business Process Weights Validation Tests
    // ==========================================================================

    #[test]
    fn test_valid_business_process_weights() {
        let config = minimal_valid_config();
        // Default weights sum to 1.0
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_business_process_weights_not_summing_to_one_rejected() {
        let mut config = minimal_valid_config();
        config.business_processes = BusinessProcessConfig {
            o2c_weight: 0.5,
            p2p_weight: 0.5,
            r2r_weight: 0.5, // Sum > 1
            h2r_weight: 0.1,
            a2r_weight: 0.1,
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Business process weights"));
    }

    // ==========================================================================
    // Fraud Configuration Validation Tests
    // ==========================================================================

    #[test]
    fn test_fraud_disabled_invalid_rate_accepted() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = false;
        config.fraud.fraud_rate = 5.0; // Invalid but ignored since disabled
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_fraud_enabled_valid_rate_accepted() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = true;
        config.fraud.fraud_rate = 0.05;
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_fraud_enabled_negative_rate_rejected() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = true;
        config.fraud.fraud_rate = -0.1;
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("fraud_rate"));
    }

    #[test]
    fn test_fraud_enabled_rate_above_one_rejected() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = true;
        config.fraud.fraud_rate = 1.5;
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("fraud_rate"));
    }

    #[test]
    fn test_fraud_rate_zero_accepted() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = true;
        config.fraud.fraud_rate = 0.0;
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_fraud_rate_one_accepted() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = true;
        config.fraud.fraud_rate = 1.0;
        assert!(validate_config(&config).is_ok());
    }

    // ==========================================================================
    // Preset Validation Tests
    // ==========================================================================

    #[test]
    fn test_demo_preset_valid() {
        let config = demo_preset();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_stress_test_preset_valid() {
        let config = stress_test_preset();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_manufacturing_preset_valid() {
        let config = create_preset(
            IndustrySector::Manufacturing,
            2,
            12,
            CoAComplexity::Medium,
            TransactionVolume::HundredK,
        );
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_retail_preset_valid() {
        let config = create_preset(
            IndustrySector::Retail,
            3,
            6,
            CoAComplexity::Large,
            TransactionVolume::OneM,
        );
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_financial_services_preset_valid() {
        let config = create_preset(
            IndustrySector::FinancialServices,
            2,
            12,
            CoAComplexity::Large,
            TransactionVolume::TenM,
        );
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_healthcare_preset_valid() {
        let config = create_preset(
            IndustrySector::Healthcare,
            2,
            6,
            CoAComplexity::Medium,
            TransactionVolume::HundredK,
        );
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_technology_preset_valid() {
        let config = create_preset(
            IndustrySector::Technology,
            3,
            12,
            CoAComplexity::Medium,
            TransactionVolume::HundredK,
        );
        assert!(validate_config(&config).is_ok());
    }

    // ==========================================================================
    // Transaction Volume Tests
    // ==========================================================================

    #[test]
    fn test_transaction_volume_counts() {
        assert_eq!(TransactionVolume::TenK.count(), 10_000);
        assert_eq!(TransactionVolume::HundredK.count(), 100_000);
        assert_eq!(TransactionVolume::OneM.count(), 1_000_000);
        assert_eq!(TransactionVolume::TenM.count(), 10_000_000);
        assert_eq!(TransactionVolume::HundredM.count(), 100_000_000);
        assert_eq!(TransactionVolume::Custom(50_000).count(), 50_000);
    }

    // ==========================================================================
    // Default Value Tests
    // ==========================================================================

    #[test]
    fn test_source_distribution_default_sums_to_one() {
        let dist = SourceDistribution::default();
        let sum = dist.manual + dist.automated + dist.recurring + dist.adjustment;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_business_process_default_sums_to_one() {
        let bp = BusinessProcessConfig::default();
        let sum = bp.o2c_weight + bp.p2p_weight + bp.r2r_weight + bp.h2r_weight + bp.a2r_weight;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_fraud_type_distribution_default_sums_to_one() {
        let dist = FraudTypeDistribution::default();
        let sum = dist.suspense_account_abuse
            + dist.fictitious_transaction
            + dist.revenue_manipulation
            + dist.expense_capitalization
            + dist.split_transaction
            + dist.timing_anomaly
            + dist.unauthorized_access
            + dist.duplicate_payment;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_persona_distribution_default_sums_to_one() {
        let dist = PersonaDistribution::default();
        let sum = dist.junior_accountant
            + dist.senior_accountant
            + dist.controller
            + dist.manager
            + dist.automated_system;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_payment_terms_distribution_default_sums_to_one() {
        let dist = PaymentTermsDistribution::default();
        let sum = dist.net_30
            + dist.net_60
            + dist.net_90
            + dist.two_ten_net_30
            + dist.due_on_receipt
            + dist.end_of_month;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_vendor_behavior_distribution_default_sums_to_one() {
        let dist = VendorBehaviorDistribution::default();
        let sum = dist.reliable
            + dist.sometimes_late
            + dist.inconsistent_quality
            + dist.premium
            + dist.budget;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_credit_rating_distribution_default_sums_to_one() {
        let dist = CreditRatingDistribution::default();
        let sum = dist.aaa + dist.aa + dist.a + dist.bbb + dist.bb + dist.b + dist.below_b;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_payment_behavior_distribution_default_sums_to_one() {
        let dist = PaymentBehaviorDistribution::default();
        let sum = dist.early_payer
            + dist.on_time
            + dist.occasional_late
            + dist.frequent_late
            + dist.discount_taker;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_material_type_distribution_default_sums_to_one() {
        let dist = MaterialTypeDistribution::default();
        let sum = dist.raw_material
            + dist.semi_finished
            + dist.finished_good
            + dist.trading_good
            + dist.operating_supply
            + dist.service;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_valuation_method_distribution_default_sums_to_one() {
        let dist = ValuationMethodDistribution::default();
        let sum = dist.standard_cost + dist.moving_average + dist.fifo + dist.lifo;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_asset_class_distribution_default_sums_to_one() {
        let dist = AssetClassDistribution::default();
        let sum = dist.buildings
            + dist.machinery
            + dist.vehicles
            + dist.it_equipment
            + dist.furniture
            + dist.land
            + dist.leasehold;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_depreciation_method_distribution_default_sums_to_one() {
        let dist = DepreciationMethodDistribution::default();
        let sum = dist.straight_line
            + dist.declining_balance
            + dist.double_declining
            + dist.sum_of_years
            + dist.units_of_production;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_employee_department_distribution_default_sums_to_one() {
        let dist = EmployeeDepartmentDistribution::default();
        let sum = dist.finance
            + dist.procurement
            + dist.sales
            + dist.warehouse
            + dist.it
            + dist.hr
            + dist.operations
            + dist.executive;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ic_transaction_type_distribution_default_sums_to_one() {
        let dist = ICTransactionTypeDistribution::default();
        let sum = dist.goods_sale
            + dist.service_provided
            + dist.loan
            + dist.dividend
            + dist.management_fee
            + dist.royalty
            + dist.cost_sharing;
        assert!((sum - 1.0).abs() < 0.001);
    }

    // ==========================================================================
    // Compression Level Validation Tests
    // ==========================================================================

    #[test]
    fn test_compression_level_valid() {
        let mut config = minimal_valid_config();
        config.output.compression.enabled = true;
        config.output.compression.level = 5;
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_compression_level_zero_rejected() {
        let mut config = minimal_valid_config();
        config.output.compression.enabled = true;
        config.output.compression.level = 0;
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("compression.level"));
    }

    #[test]
    fn test_compression_level_ten_rejected() {
        let mut config = minimal_valid_config();
        config.output.compression.enabled = true;
        config.output.compression.level = 10;
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("compression.level"));
    }

    #[test]
    fn test_compression_disabled_ignores_level() {
        let mut config = minimal_valid_config();
        config.output.compression.enabled = false;
        config.output.compression.level = 0; // Invalid but ignored
        assert!(validate_config(&config).is_ok());
    }

    // ==========================================================================
    // Approval Threshold Ordering Tests
    // ==========================================================================

    #[test]
    fn test_approval_thresholds_ascending_accepted() {
        let mut config = minimal_valid_config();
        config.approval.enabled = true;
        config.approval.thresholds = vec![
            ApprovalThresholdConfig {
                amount: 1000.0,
                level: 1,
                roles: vec!["accountant".to_string()],
            },
            ApprovalThresholdConfig {
                amount: 5000.0,
                level: 2,
                roles: vec!["manager".to_string()],
            },
            ApprovalThresholdConfig {
                amount: 10000.0,
                level: 3,
                roles: vec!["director".to_string()],
            },
        ];
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_approval_thresholds_not_ascending_rejected() {
        let mut config = minimal_valid_config();
        config.approval.enabled = true;
        config.approval.thresholds = vec![
            ApprovalThresholdConfig {
                amount: 5000.0,
                level: 1,
                roles: vec!["accountant".to_string()],
            },
            ApprovalThresholdConfig {
                amount: 1000.0, // Less than previous - invalid
                level: 2,
                roles: vec!["manager".to_string()],
            },
        ];
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ascending"));
    }

    #[test]
    fn test_fraud_approval_thresholds_ascending_accepted() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = true;
        config.fraud.fraud_rate = 0.05;
        config.fraud.approval_thresholds = vec![1000.0, 5000.0, 10000.0];
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_fraud_approval_thresholds_not_ascending_rejected() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = true;
        config.fraud.fraud_rate = 0.05;
        config.fraud.approval_thresholds = vec![5000.0, 1000.0, 10000.0]; // Not ascending
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ascending"));
    }

    // ==========================================================================
    // Rate/Percentage Validation Tests
    // ==========================================================================

    #[test]
    fn test_internal_controls_rates_valid() {
        let mut config = minimal_valid_config();
        config.internal_controls.enabled = true;
        config.internal_controls.exception_rate = 0.05;
        config.internal_controls.sod_violation_rate = 0.02;
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_internal_controls_exception_rate_invalid() {
        let mut config = minimal_valid_config();
        config.internal_controls.enabled = true;
        config.internal_controls.exception_rate = 1.5; // > 1.0
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exception_rate"));
    }

    #[test]
    fn test_approval_rejection_plus_revision_exceeds_one_rejected() {
        let mut config = minimal_valid_config();
        config.approval.enabled = true;
        config.approval.rejection_rate = 0.6;
        config.approval.revision_rate = 0.6; // Sum > 1.0
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must not exceed 1.0"));
    }

    #[test]
    fn test_master_data_intercompany_percent_invalid() {
        let mut config = minimal_valid_config();
        config.master_data.vendors.intercompany_percent = 1.5; // > 1.0
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("intercompany_percent"));
    }

    #[test]
    fn test_balance_gross_margin_invalid() {
        let mut config = minimal_valid_config();
        config.balance.target_gross_margin = 1.5; // > 1.0
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("target_gross_margin"));
    }

    #[test]
    fn test_negative_volume_weight_rejected() {
        let mut config = minimal_valid_config();
        config.companies[0].volume_weight = -0.5;
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("volume_weight"));
    }

    #[test]
    fn test_benford_tolerance_invalid() {
        let mut config = minimal_valid_config();
        config.transactions.benford.tolerance = 1.5; // > 1.0
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("benford.tolerance"));
    }

    #[test]
    fn test_batch_size_zero_rejected() {
        let mut config = minimal_valid_config();
        config.output.batch_size = 0;
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("batch_size"));
    }
}
