//! Configuration validation.

use crate::schema::GeneratorConfig;
use datasynth_core::error::{SynthError, SynthResult};

/// Maximum allowed period in months (10 years).
const MAX_PERIOD_MONTHS: u32 = 120;

/// Check if a string is in valid HH:MM time format.
fn is_valid_time_format(s: &str) -> bool {
    if s.len() != 5 {
        return false;
    }
    let chars: Vec<char> = s.chars().collect();
    if chars[2] != ':' {
        return false;
    }
    // Check hours (00-23)
    let hours: Option<u8> = s[0..2].parse().ok();
    let minutes: Option<u8> = s[3..5].parse().ok();
    match (hours, minutes) {
        (Some(h), Some(m)) => h <= 23 && m <= 59,
        _ => false,
    }
}

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
    validate_temporal_patterns(config)?;
    validate_vendor_network(config)?;
    validate_customer_segmentation(config)?;
    validate_relationship_strength(config)?;
    validate_cross_process_links(config)?;
    validate_anomaly_injection(config)?;
    validate_hypergraph(config)?;
    validate_fingerprint_privacy(config)?;
    validate_quality_gates(config)?;
    validate_compliance(config)?;
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
    validate_sum_to_one(
        "Source distribution",
        &[
            config.transactions.source_distribution.manual,
            config.transactions.source_distribution.automated,
            config.transactions.source_distribution.recurring,
            config.transactions.source_distribution.adjustment,
        ],
    )?;

    // Validate business process weights
    validate_sum_to_one(
        "Business process weights",
        &[
            config.business_processes.o2c_weight,
            config.business_processes.p2p_weight,
            config.business_processes.r2r_weight,
            config.business_processes.h2r_weight,
            config.business_processes.a2r_weight,
        ],
    )?;

    // Validate Benford tolerance
    validate_rate("benford.tolerance", config.transactions.benford.tolerance)?;

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

    validate_rate("fraud_rate", config.fraud.fraud_rate)?;

    if config.fraud.clustering_factor < 0.0 {
        return Err(SynthError::validation(
            "clustering_factor must be non-negative",
        ));
    }

    // Validate approval thresholds are in ascending order
    validate_ascending(
        "fraud.approval_thresholds",
        &config.fraud.approval_thresholds,
    )?;

    // Validate fraud type distribution sums to ~1.0
    let dist = &config.fraud.fraud_type_distribution;
    validate_sum_to_one(
        "fraud_type_distribution",
        &[
            dist.suspense_account_abuse,
            dist.fictitious_transaction,
            dist.revenue_manipulation,
            dist.expense_capitalization,
            dist.split_transaction,
            dist.timing_anomaly,
            dist.unauthorized_access,
            dist.duplicate_payment,
        ],
    )?;

    Ok(())
}

/// Validate internal controls configuration.
fn validate_internal_controls(config: &GeneratorConfig) -> SynthResult<()> {
    if !config.internal_controls.enabled {
        return Ok(());
    }

    validate_rate("exception_rate", config.internal_controls.exception_rate)?;

    validate_rate(
        "sod_violation_rate",
        config.internal_controls.sod_violation_rate,
    )?;

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
    validate_rate("rejection_rate", rejection_rate)?;

    let revision_rate = config.approval.revision_rate;
    validate_rate("revision_rate", revision_rate)?;

    // rejection + revision should not exceed 1.0
    if rejection_rate + revision_rate > 1.0 {
        return Err(SynthError::validation(format!(
            "rejection_rate + revision_rate must not exceed 1.0, got {}",
            rejection_rate + revision_rate
        )));
    }

    // Validate approval thresholds are in ascending order by amount
    let threshold_amounts: Vec<f64> = config
        .approval
        .thresholds
        .iter()
        .map(|t| t.amount)
        .collect();
    validate_ascending("approval.thresholds", &threshold_amounts)?;

    Ok(())
}

/// Validate master data configuration.
fn validate_master_data(config: &GeneratorConfig) -> SynthResult<()> {
    // Vendor config
    validate_rate(
        "vendors.intercompany_percent",
        config.master_data.vendors.intercompany_percent,
    )?;

    // Customer config
    validate_rate(
        "customers.intercompany_percent",
        config.master_data.customers.intercompany_percent,
    )?;

    // Material config
    validate_rate(
        "materials.bom_percent",
        config.master_data.materials.bom_percent,
    )?;

    // Fixed asset config
    validate_rate(
        "fixed_assets.fully_depreciated_percent",
        config.master_data.fixed_assets.fully_depreciated_percent,
    )?;

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
    validate_sum_to_one(
        "p2p.payment_behavior.late_payment_days_distribution",
        &[
            late_dist.slightly_late_1_to_7,
            late_dist.late_8_to_14,
            late_dist.very_late_15_to_30,
            late_dist.severely_late_31_to_60,
            late_dist.extremely_late_over_60,
        ],
    )?;

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
        validate_sum_to_one(
            "dunning.payment_after_dunning_rates",
            &[
                rates.after_level_1,
                rates.after_level_2,
                rates.after_level_3,
                rates.during_collection,
                rates.never_pay,
            ],
        )?;
    }

    // Validate partial payments config
    let partial = &config.partial_payments;
    validate_rate("o2c.payment_behavior.partial_payments.rate", partial.rate)?;
    let partial_dist = &partial.percentage_distribution;
    validate_sum_to_one(
        "partial_payments.percentage_distribution",
        &[
            partial_dist.pay_25_percent,
            partial_dist.pay_50_percent,
            partial_dist.pay_75_percent,
            partial_dist.pay_random_percent,
        ],
    )?;

    // Validate short payments config
    let short = &config.short_payments;
    validate_rate("o2c.payment_behavior.short_payments.rate", short.rate)?;
    validate_rate(
        "o2c.payment_behavior.short_payments.max_short_percent",
        short.max_short_percent,
    )?;
    let short_dist = &short.reason_distribution;
    validate_sum_to_one(
        "short_payments.reason_distribution",
        &[
            short_dist.pricing_dispute,
            short_dist.quality_issue,
            short_dist.quantity_discrepancy,
            short_dist.unauthorized_deduction,
            short_dist.incorrect_discount,
        ],
    )?;

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
    validate_sum_to_one(
        "payment_corrections.type_distribution",
        &[
            corr_dist.nsf,
            corr_dist.chargeback,
            corr_dist.wrong_amount,
            corr_dist.wrong_customer,
            corr_dist.duplicate_payment,
        ],
    )?;

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
    validate_sum_to_one(
        "intercompany.transaction_type_distribution",
        &[
            dist.goods_sale,
            dist.service_provided,
            dist.loan,
            dist.dividend,
            dist.management_fee,
            dist.royalty,
            dist.cost_sharing,
        ],
    )?;

    Ok(())
}

/// Validate balance configuration.
fn validate_balance(config: &GeneratorConfig) -> SynthResult<()> {
    let balance = &config.balance;

    validate_rate("target_gross_margin", balance.target_gross_margin)?;

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

/// Helper to validate that a slice of f64 values sums to approximately 1.0 (within 0.01 tolerance).
fn validate_sum_to_one(name: &str, values: &[f64]) -> SynthResult<()> {
    let sum: f64 = values.iter().sum();
    if (sum - 1.0).abs() > 0.01 {
        return Err(SynthError::validation(format!(
            "{} must sum to 1.0, got {}",
            name, sum
        )));
    }
    Ok(())
}

/// Helper to validate that a f64 value is within a given range [min, max].
fn validate_range_f64(name: &str, value: f64, min: f64, max: f64) -> SynthResult<()> {
    if value < min || value > max {
        return Err(SynthError::validation(format!(
            "{} must be between {} and {}, got {}",
            name, min, max, value
        )));
    }
    Ok(())
}

/// Helper to validate that a slice of f64 values is in strictly ascending order.
fn validate_ascending(name: &str, values: &[f64]) -> SynthResult<()> {
    for i in 1..values.len() {
        if values[i] <= values[i - 1] {
            return Err(SynthError::validation(format!(
                "{} must be in ascending order",
                name
            )));
        }
    }
    Ok(())
}

/// Helper to validate that a f64 value is strictly positive (> 0.0).
fn validate_positive(name: &str, value: f64) -> SynthResult<()> {
    if value <= 0.0 {
        return Err(SynthError::validation(format!(
            "{} must be positive, got {}",
            name, value
        )));
    }
    Ok(())
}

/// Helper to validate a rate field is between 0.0 and 1.0.
fn validate_rate(field_name: &str, value: f64) -> SynthResult<()> {
    validate_range_f64(field_name, value, 0.0, 1.0)
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
        validate_sum_to_one(
            "fair_value level percentages",
            &[fv.level1_percent, fv.level2_percent, fv.level3_percent],
        )?;

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
    let weights: Vec<f64> = config.components.iter().map(|c| c.weight).collect();
    validate_sum_to_one("distributions.amounts.components weights", &weights)?;

    // Validate individual components
    for (i, comp) in config.components.iter().enumerate() {
        validate_rate(
            &format!("distributions.amounts.components[{}].weight", i),
            comp.weight,
        )?;

        validate_positive(
            &format!("distributions.amounts.components[{}].sigma", i),
            comp.sigma,
        )?;
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
    let thresholds: Vec<f64> = config.breakpoints.iter().map(|b| b.threshold).collect();
    validate_ascending(
        &format!("distributions.conditional[{}].breakpoints", index),
        &thresholds,
    )?;

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

            validate_rate(
                "distributions.regime_changes.economic_cycle.amplitude",
                cycle.amplitude,
            )?;

            // Validate recession periods
            for (i, recession) in cycle.recessions.iter().enumerate() {
                if recession.duration_months == 0 {
                    return Err(SynthError::validation(format!(
                        "distributions.regime_changes.economic_cycle.recessions[{}].duration_months must be > 0",
                        i
                    )));
                }

                validate_rate(
                    &format!(
                        "distributions.regime_changes.economic_cycle.recessions[{}].severity",
                        i
                    ),
                    recession.severity,
                )?;
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
                validate_positive(
                    &format!("distributions.validation.tests[{}].threshold_mad", i),
                    *threshold_mad,
                )?;
                validate_positive(
                    &format!("distributions.validation.tests[{}].warning_mad", i),
                    *warning_mad,
                )?;
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

/// Validate temporal patterns configuration.
fn validate_temporal_patterns(config: &GeneratorConfig) -> SynthResult<()> {
    let temporal = &config.temporal_patterns;

    if !temporal.enabled {
        return Ok(());
    }

    // Validate business day configuration
    validate_business_day_config(&temporal.business_days)?;

    // Validate period-end configuration
    validate_period_end_config(&temporal.period_end)?;

    // Validate processing lag configuration
    validate_processing_lag_config(&temporal.processing_lags)?;

    // Validate calendar regions
    validate_calendar_config(&temporal.calendars)?;

    // Validate fiscal calendar configuration (P2)
    validate_fiscal_calendar_config(&temporal.fiscal_calendar)?;

    // Validate intra-day patterns configuration (P2)
    validate_intraday_config(&temporal.intraday)?;

    // Validate timezone configuration (P2)
    validate_timezone_config(&temporal.timezones)?;

    Ok(())
}

/// Validate business day configuration.
fn validate_business_day_config(
    config: &crate::schema::BusinessDaySchemaConfig,
) -> SynthResult<()> {
    if !config.enabled {
        return Ok(());
    }

    // Validate half-day policy
    let valid_policies = ["full_day", "half_day", "non_business_day"];
    if !valid_policies.contains(&config.half_day_policy.as_str()) {
        return Err(SynthError::validation(format!(
            "temporal_patterns.business_days.half_day_policy must be one of {:?}, got '{}'",
            valid_policies, config.half_day_policy
        )));
    }

    // Validate month-end convention
    let valid_conventions = [
        "modified_following",
        "preceding",
        "following",
        "end_of_month",
    ];
    if !valid_conventions.contains(&config.month_end_convention.as_str()) {
        return Err(SynthError::validation(format!(
            "temporal_patterns.business_days.month_end_convention must be one of {:?}, got '{}'",
            valid_conventions, config.month_end_convention
        )));
    }

    // Validate settlement rules
    let rules = &config.settlement_rules;
    if rules.equity_days < 0 {
        return Err(SynthError::validation(
            "temporal_patterns.business_days.settlement_rules.equity_days must be non-negative",
        ));
    }
    if rules.government_bonds_days < 0 {
        return Err(SynthError::validation(
            "temporal_patterns.business_days.settlement_rules.government_bonds_days must be non-negative",
        ));
    }
    if rules.fx_spot_days < 0 {
        return Err(SynthError::validation(
            "temporal_patterns.business_days.settlement_rules.fx_spot_days must be non-negative",
        ));
    }

    // Validate wire cutoff time format (HH:MM)
    if !rules.wire_cutoff_time.contains(':') {
        return Err(SynthError::validation(format!(
            "temporal_patterns.business_days.settlement_rules.wire_cutoff_time must be in HH:MM format, got '{}'",
            rules.wire_cutoff_time
        )));
    }

    // Validate weekend days if provided
    if let Some(ref weekend_days) = config.weekend_days {
        let valid_days = [
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
            "sunday",
        ];
        for day in weekend_days {
            if !valid_days.contains(&day.to_lowercase().as_str()) {
                return Err(SynthError::validation(format!(
                    "temporal_patterns.business_days.weekend_days contains invalid day '{}', must be one of {:?}",
                    day, valid_days
                )));
            }
        }
    }

    Ok(())
}

/// Validate period-end configuration.
fn validate_period_end_config(config: &crate::schema::PeriodEndSchemaConfig) -> SynthResult<()> {
    // Validate model type if specified
    if let Some(ref model) = config.model {
        let valid_models = ["flat", "exponential", "extended_crunch", "daily_profile"];
        if !valid_models.contains(&model.as_str()) {
            return Err(SynthError::validation(format!(
                "temporal_patterns.period_end.model must be one of {:?}, got '{}'",
                valid_models, model
            )));
        }
    }

    // Validate month-end config if present
    if let Some(ref month_end) = config.month_end {
        validate_period_end_model_config(month_end, "month_end")?;
    }

    // Validate quarter-end config if present
    if let Some(ref quarter_end) = config.quarter_end {
        validate_period_end_model_config(quarter_end, "quarter_end")?;
    }

    // Validate year-end config if present
    if let Some(ref year_end) = config.year_end {
        validate_period_end_model_config(year_end, "year_end")?;
    }

    Ok(())
}

/// Validate a period-end model configuration.
fn validate_period_end_model_config(
    config: &crate::schema::PeriodEndModelSchemaConfig,
    name: &str,
) -> SynthResult<()> {
    // Validate multipliers are positive
    if let Some(mult) = config.additional_multiplier {
        validate_positive(
            &format!(
                "temporal_patterns.period_end.{}.additional_multiplier",
                name
            ),
            mult,
        )?;
    }

    if let Some(mult) = config.base_multiplier {
        validate_positive(
            &format!("temporal_patterns.period_end.{}.base_multiplier", name),
            mult,
        )?;
    }

    if let Some(mult) = config.peak_multiplier {
        validate_positive(
            &format!("temporal_patterns.period_end.{}.peak_multiplier", name),
            mult,
        )?;
    }

    // Validate decay_rate is in valid range (0, 1] for exponential model
    if let Some(rate) = config.decay_rate {
        if rate <= 0.0 || rate > 1.0 {
            return Err(SynthError::validation(format!(
                "temporal_patterns.period_end.{}.decay_rate must be in (0, 1], got {}",
                name, rate
            )));
        }
    }

    // Validate start_day is negative or zero
    if let Some(day) = config.start_day {
        if day > 0 {
            return Err(SynthError::validation(format!(
                "temporal_patterns.period_end.{}.start_day must be <= 0 (days before period end), got {}",
                name, day
            )));
        }
    }

    // Validate sustained_high_days is positive
    if let Some(days) = config.sustained_high_days {
        if days <= 0 {
            return Err(SynthError::validation(format!(
                "temporal_patterns.period_end.{}.sustained_high_days must be positive, got {}",
                name, days
            )));
        }
    }

    Ok(())
}

/// Validate processing lag configuration.
fn validate_processing_lag_config(
    config: &crate::schema::ProcessingLagSchemaConfig,
) -> SynthResult<()> {
    if !config.enabled {
        return Ok(());
    }

    // Validate lag distributions
    let lag_configs = [
        (&config.sales_order_lag, "sales_order_lag"),
        (&config.purchase_order_lag, "purchase_order_lag"),
        (&config.goods_receipt_lag, "goods_receipt_lag"),
        (&config.invoice_receipt_lag, "invoice_receipt_lag"),
        (&config.invoice_issue_lag, "invoice_issue_lag"),
        (&config.payment_lag, "payment_lag"),
        (&config.journal_entry_lag, "journal_entry_lag"),
    ];

    for (lag_opt, name) in lag_configs {
        if let Some(lag) = lag_opt {
            validate_lag_distribution(lag, name)?;
        }
    }

    // Validate cross-day posting config
    if let Some(ref cross_day) = config.cross_day_posting {
        for (hour, prob) in &cross_day.probability_by_hour {
            if *hour > 23 {
                return Err(SynthError::validation(format!(
                    "temporal_patterns.processing_lags.cross_day_posting.probability_by_hour contains invalid hour {}, must be 0-23",
                    hour
                )));
            }
            validate_rate(
                &format!(
                    "temporal_patterns.processing_lags.cross_day_posting.probability_by_hour[{}]",
                    hour
                ),
                *prob,
            )?;
        }
    }

    Ok(())
}

/// Validate a lag distribution configuration.
fn validate_lag_distribution(
    config: &crate::schema::LagDistributionSchemaConfig,
    name: &str,
) -> SynthResult<()> {
    // Sigma must be positive for log-normal
    validate_positive(
        &format!("temporal_patterns.processing_lags.{}.sigma", name),
        config.sigma,
    )?;

    // Min/max hours must be non-negative and ordered correctly
    if let Some(min) = config.min_hours {
        if min < 0.0 {
            return Err(SynthError::validation(format!(
                "temporal_patterns.processing_lags.{}.min_hours must be non-negative, got {}",
                name, min
            )));
        }
    }

    if let Some(max) = config.max_hours {
        if max < 0.0 {
            return Err(SynthError::validation(format!(
                "temporal_patterns.processing_lags.{}.max_hours must be non-negative, got {}",
                name, max
            )));
        }

        if let Some(min) = config.min_hours {
            if max < min {
                return Err(SynthError::validation(format!(
                    "temporal_patterns.processing_lags.{}.max_hours ({}) must be >= min_hours ({})",
                    name, max, min
                )));
            }
        }
    }

    Ok(())
}

/// Validate calendar configuration.
fn validate_calendar_config(config: &crate::schema::CalendarSchemaConfig) -> SynthResult<()> {
    // Validate region codes
    let valid_regions = [
        "US", "DE", "GB", "CN", "JP", "IN", "BR", "MX", "AU", "SG", "KR",
    ];
    for region in &config.regions {
        let region_upper = region.to_uppercase();
        if !valid_regions.contains(&region_upper.as_str()) {
            return Err(SynthError::validation(format!(
                "temporal_patterns.calendars.regions contains invalid region '{}', must be one of {:?}",
                region, valid_regions
            )));
        }
    }

    // Validate custom holidays
    for (i, holiday) in config.custom_holidays.iter().enumerate() {
        if holiday.name.is_empty() {
            return Err(SynthError::validation(format!(
                "temporal_patterns.calendars.custom_holidays[{}].name cannot be empty",
                i
            )));
        }

        if holiday.month < 1 || holiday.month > 12 {
            return Err(SynthError::validation(format!(
                "temporal_patterns.calendars.custom_holidays[{}].month must be 1-12, got {}",
                i, holiday.month
            )));
        }

        if holiday.day < 1 || holiday.day > 31 {
            return Err(SynthError::validation(format!(
                "temporal_patterns.calendars.custom_holidays[{}].day must be 1-31, got {}",
                i, holiday.day
            )));
        }

        validate_rate(
            &format!(
                "temporal_patterns.calendars.custom_holidays[{}].activity_multiplier",
                i
            ),
            holiday.activity_multiplier,
        )?;
    }

    Ok(())
}

/// Validate fiscal calendar configuration.
fn validate_fiscal_calendar_config(
    config: &crate::schema::FiscalCalendarSchemaConfig,
) -> SynthResult<()> {
    if !config.enabled {
        return Ok(());
    }

    // Validate calendar type
    let valid_types = [
        "calendar_year",
        "custom",
        "four_four_five",
        "thirteen_period",
    ];
    if !valid_types.contains(&config.calendar_type.as_str()) {
        return Err(SynthError::validation(format!(
            "temporal_patterns.fiscal_calendar.calendar_type must be one of {:?}, got '{}'",
            valid_types, config.calendar_type
        )));
    }

    // Validate custom year start
    if config.calendar_type == "custom" {
        if let Some(month) = config.year_start_month {
            if !(1..=12).contains(&month) {
                return Err(SynthError::validation(format!(
                    "temporal_patterns.fiscal_calendar.year_start_month must be 1-12, got {}",
                    month
                )));
            }
        } else {
            return Err(SynthError::validation(
                "temporal_patterns.fiscal_calendar.year_start_month is required for custom calendar type",
            ));
        }

        if let Some(day) = config.year_start_day {
            if !(1..=31).contains(&day) {
                return Err(SynthError::validation(format!(
                    "temporal_patterns.fiscal_calendar.year_start_day must be 1-31, got {}",
                    day
                )));
            }
        }
    }

    // Validate 4-4-5 configuration
    if config.calendar_type == "four_four_five" {
        if let Some(ref cfg) = config.four_four_five {
            let valid_patterns = ["four_four_five", "four_five_four", "five_four_four"];
            if !valid_patterns.contains(&cfg.pattern.as_str()) {
                return Err(SynthError::validation(format!(
                    "temporal_patterns.fiscal_calendar.four_four_five.pattern must be one of {:?}, got '{}'",
                    valid_patterns, cfg.pattern
                )));
            }

            let valid_anchors = ["first_sunday", "last_saturday", "nearest_saturday"];
            if !valid_anchors.contains(&cfg.anchor_type.as_str()) {
                return Err(SynthError::validation(format!(
                    "temporal_patterns.fiscal_calendar.four_four_five.anchor_type must be one of {:?}, got '{}'",
                    valid_anchors, cfg.anchor_type
                )));
            }

            if !(1..=12).contains(&cfg.anchor_month) {
                return Err(SynthError::validation(format!(
                    "temporal_patterns.fiscal_calendar.four_four_five.anchor_month must be 1-12, got {}",
                    cfg.anchor_month
                )));
            }

            let valid_placements = ["q4_period3", "q1_period1"];
            if !valid_placements.contains(&cfg.leap_week_placement.as_str()) {
                return Err(SynthError::validation(format!(
                    "temporal_patterns.fiscal_calendar.four_four_five.leap_week_placement must be one of {:?}, got '{}'",
                    valid_placements, cfg.leap_week_placement
                )));
            }
        }
    }

    Ok(())
}

/// Validate intra-day patterns configuration.
fn validate_intraday_config(config: &crate::schema::IntraDaySchemaConfig) -> SynthResult<()> {
    if !config.enabled {
        return Ok(());
    }

    // Validate each segment
    for (i, segment) in config.segments.iter().enumerate() {
        // Validate segment name is not empty
        if segment.name.is_empty() {
            return Err(SynthError::validation(format!(
                "temporal_patterns.intraday.segments[{}].name cannot be empty",
                i
            )));
        }

        // Validate time format (HH:MM) - simple check without regex
        if !is_valid_time_format(&segment.start) {
            return Err(SynthError::validation(format!(
                "temporal_patterns.intraday.segments[{}].start must be in HH:MM format, got '{}'",
                i, segment.start
            )));
        }
        if !is_valid_time_format(&segment.end) {
            return Err(SynthError::validation(format!(
                "temporal_patterns.intraday.segments[{}].end must be in HH:MM format, got '{}'",
                i, segment.end
            )));
        }

        // Validate multiplier is positive
        if segment.multiplier < 0.0 {
            return Err(SynthError::validation(format!(
                "temporal_patterns.intraday.segments[{}].multiplier must be non-negative, got {}",
                i, segment.multiplier
            )));
        }

        // Validate posting type
        let valid_posting_types = ["human", "system", "both"];
        if !valid_posting_types.contains(&segment.posting_type.as_str()) {
            return Err(SynthError::validation(format!(
                "temporal_patterns.intraday.segments[{}].posting_type must be one of {:?}, got '{}'",
                i, valid_posting_types, segment.posting_type
            )));
        }
    }

    Ok(())
}

/// Validate timezone configuration.
fn validate_timezone_config(config: &crate::schema::TimezoneSchemaConfig) -> SynthResult<()> {
    if !config.enabled {
        return Ok(());
    }

    // Validate default timezone is a valid IANA timezone name
    if !is_valid_iana_timezone(&config.default_timezone) {
        return Err(SynthError::validation(format!(
            "temporal_patterns.timezones.default_timezone '{}' is not a valid IANA timezone name",
            config.default_timezone
        )));
    }

    // Validate consolidation timezone
    if !is_valid_iana_timezone(&config.consolidation_timezone) {
        return Err(SynthError::validation(format!(
            "temporal_patterns.timezones.consolidation_timezone '{}' is not a valid IANA timezone name",
            config.consolidation_timezone
        )));
    }

    // Validate entity mappings
    for (i, mapping) in config.entity_mappings.iter().enumerate() {
        if mapping.pattern.is_empty() {
            return Err(SynthError::validation(format!(
                "temporal_patterns.timezones.entity_mappings[{}].pattern cannot be empty",
                i
            )));
        }

        if !is_valid_iana_timezone(&mapping.timezone) {
            return Err(SynthError::validation(format!(
                "temporal_patterns.timezones.entity_mappings[{}].timezone '{}' is not a valid IANA timezone name",
                i, mapping.timezone
            )));
        }
    }

    Ok(())
}

/// Check if a string is a valid IANA timezone name.
/// This is a simplified check that validates common timezone patterns.
fn is_valid_iana_timezone(tz: &str) -> bool {
    // Common valid timezones - check against a set of patterns
    // IANA timezones are typically in format "Continent/City" or "Etc/GMT+N"
    if tz == "UTC" || tz == "GMT" {
        return true;
    }

    // Check for Continent/City format
    let parts: Vec<&str> = tz.split('/').collect();
    if parts.len() >= 2 {
        let valid_continents = [
            "Africa",
            "America",
            "Antarctica",
            "Arctic",
            "Asia",
            "Atlantic",
            "Australia",
            "Europe",
            "Indian",
            "Pacific",
            "Etc",
        ];
        if valid_continents.contains(&parts[0]) {
            return true;
        }
    }

    false
}

/// Validate vendor network configuration.
fn validate_vendor_network(config: &GeneratorConfig) -> SynthResult<()> {
    let vn = &config.vendor_network;

    if !vn.enabled {
        return Ok(());
    }

    // Validate tier depth is 1-3
    if vn.depth == 0 || vn.depth > 3 {
        return Err(SynthError::validation(format!(
            "vendor_network.depth must be between 1 and 3, got {}",
            vn.depth
        )));
    }

    // Validate tier1 count
    if vn.tier1.min > vn.tier1.max {
        return Err(SynthError::validation(format!(
            "vendor_network.tier1.min ({}) must be <= max ({})",
            vn.tier1.min, vn.tier1.max
        )));
    }

    // Validate tier2_per_parent count
    if vn.tier2_per_parent.min > vn.tier2_per_parent.max {
        return Err(SynthError::validation(format!(
            "vendor_network.tier2_per_parent.min ({}) must be <= max ({})",
            vn.tier2_per_parent.min, vn.tier2_per_parent.max
        )));
    }

    // Validate tier3_per_parent count
    if vn.tier3_per_parent.min > vn.tier3_per_parent.max {
        return Err(SynthError::validation(format!(
            "vendor_network.tier3_per_parent.min ({}) must be <= max ({})",
            vn.tier3_per_parent.min, vn.tier3_per_parent.max
        )));
    }

    // Validate cluster distribution sums to ~1.0
    let clusters = &vn.clusters;
    validate_sum_to_one(
        "vendor_network.clusters distribution",
        &[
            clusters.reliable_strategic,
            clusters.standard_operational,
            clusters.transactional,
            clusters.problematic,
        ],
    )?;

    // Validate concentration limits are in valid range
    let deps = &vn.dependencies;
    validate_rate(
        "vendor_network.dependencies.max_single_vendor_concentration",
        deps.max_single_vendor_concentration,
    )?;
    validate_rate(
        "vendor_network.dependencies.top_5_concentration",
        deps.top_5_concentration,
    )?;
    validate_rate(
        "vendor_network.dependencies.single_source_percent",
        deps.single_source_percent,
    )?;

    // Max single vendor should be less than top 5 (logical constraint)
    if deps.max_single_vendor_concentration > deps.top_5_concentration {
        return Err(SynthError::validation(format!(
            "vendor_network.dependencies.max_single_vendor_concentration ({}) should be <= top_5_concentration ({})",
            deps.max_single_vendor_concentration, deps.top_5_concentration
        )));
    }

    Ok(())
}

/// Validate customer segmentation configuration.
fn validate_customer_segmentation(config: &GeneratorConfig) -> SynthResult<()> {
    let cs = &config.customer_segmentation;

    if !cs.enabled {
        return Ok(());
    }

    // Validate value segments
    let segments = &cs.value_segments;

    // Validate revenue shares sum to ~1.0
    validate_sum_to_one(
        "customer_segmentation.value_segments revenue_share",
        &[
            segments.enterprise.revenue_share,
            segments.mid_market.revenue_share,
            segments.smb.revenue_share,
            segments.consumer.revenue_share,
        ],
    )?;

    // Validate customer shares sum to ~1.0
    validate_sum_to_one(
        "customer_segmentation.value_segments customer_share",
        &[
            segments.enterprise.customer_share,
            segments.mid_market.customer_share,
            segments.smb.customer_share,
            segments.consumer.customer_share,
        ],
    )?;

    // Validate each segment's shares are in valid range
    for (name, seg) in [
        ("enterprise", &segments.enterprise),
        ("mid_market", &segments.mid_market),
        ("smb", &segments.smb),
        ("consumer", &segments.consumer),
    ] {
        validate_rate(
            &format!(
                "customer_segmentation.value_segments.{}.revenue_share",
                name
            ),
            seg.revenue_share,
        )?;
        validate_rate(
            &format!(
                "customer_segmentation.value_segments.{}.customer_share",
                name
            ),
            seg.customer_share,
        )?;
    }

    // Validate lifecycle distribution sums to ~1.0
    let lifecycle = &cs.lifecycle;
    validate_sum_to_one(
        "customer_segmentation.lifecycle distribution",
        &[
            lifecycle.prospect_rate,
            lifecycle.new_rate,
            lifecycle.growth_rate,
            lifecycle.mature_rate,
            lifecycle.at_risk_rate,
            lifecycle.churned_rate,
        ],
    )?;

    // Validate network config
    let networks = &cs.networks;
    validate_rate(
        "customer_segmentation.networks.referrals.referral_rate",
        networks.referrals.referral_rate,
    )?;
    validate_rate(
        "customer_segmentation.networks.corporate_hierarchies.probability",
        networks.corporate_hierarchies.probability,
    )?;

    Ok(())
}

/// Validate relationship strength configuration.
fn validate_relationship_strength(config: &GeneratorConfig) -> SynthResult<()> {
    let rs = &config.relationship_strength;

    if !rs.enabled {
        return Ok(());
    }

    // Validate calculation weights sum to ~1.0
    let calc = &rs.calculation;
    validate_sum_to_one(
        "relationship_strength.calculation weights",
        &[
            calc.transaction_volume_weight,
            calc.transaction_count_weight,
            calc.relationship_duration_weight,
            calc.recency_weight,
            calc.mutual_connections_weight,
        ],
    )?;

    // Validate individual weights are in valid range
    validate_rate(
        "relationship_strength.calculation.transaction_volume_weight",
        calc.transaction_volume_weight,
    )?;
    validate_rate(
        "relationship_strength.calculation.transaction_count_weight",
        calc.transaction_count_weight,
    )?;
    validate_rate(
        "relationship_strength.calculation.relationship_duration_weight",
        calc.relationship_duration_weight,
    )?;
    validate_rate(
        "relationship_strength.calculation.recency_weight",
        calc.recency_weight,
    )?;
    validate_rate(
        "relationship_strength.calculation.mutual_connections_weight",
        calc.mutual_connections_weight,
    )?;

    // Validate recency half-life is positive
    if calc.recency_half_life_days == 0 {
        return Err(SynthError::validation(
            "relationship_strength.calculation.recency_half_life_days must be positive",
        ));
    }

    // Validate thresholds are in valid range and descending order
    let thresh = &rs.thresholds;
    validate_rate("relationship_strength.thresholds.strong", thresh.strong)?;
    validate_rate("relationship_strength.thresholds.moderate", thresh.moderate)?;
    validate_rate("relationship_strength.thresholds.weak", thresh.weak)?;

    // Thresholds should be in descending order: strong > moderate > weak
    if thresh.strong <= thresh.moderate {
        return Err(SynthError::validation(format!(
            "relationship_strength.thresholds.strong ({}) must be > moderate ({})",
            thresh.strong, thresh.moderate
        )));
    }
    if thresh.moderate <= thresh.weak {
        return Err(SynthError::validation(format!(
            "relationship_strength.thresholds.moderate ({}) must be > weak ({})",
            thresh.moderate, thresh.weak
        )));
    }

    Ok(())
}

/// Validate cross-process links configuration.
fn validate_cross_process_links(config: &GeneratorConfig) -> SynthResult<()> {
    let cpl = &config.cross_process_links;

    if !cpl.enabled {
        return Ok(());
    }

    // Cross-process links are boolean flags, so there's not much to validate
    // beyond ensuring they're consistent with other config settings

    // If inventory P2P-O2C links are enabled, ensure both document flows are enabled
    if cpl.inventory_p2p_o2c {
        if !config.document_flows.p2p.enabled {
            return Err(SynthError::validation(
                "cross_process_links.inventory_p2p_o2c requires document_flows.p2p to be enabled",
            ));
        }
        if !config.document_flows.o2c.enabled {
            return Err(SynthError::validation(
                "cross_process_links.inventory_p2p_o2c requires document_flows.o2c to be enabled",
            ));
        }
    }

    // If intercompany bilateral links are enabled, ensure intercompany is enabled
    if cpl.intercompany_bilateral && !config.intercompany.enabled {
        return Err(SynthError::validation(
            "cross_process_links.intercompany_bilateral requires intercompany to be enabled",
        ));
    }

    Ok(())
}

/// Validate enhanced anomaly injection configuration.
fn validate_anomaly_injection(config: &GeneratorConfig) -> SynthResult<()> {
    let ai = &config.anomaly_injection;

    if !ai.enabled {
        return Ok(());
    }

    // Validate rates are within bounds
    validate_rate("anomaly_injection.rates.total_rate", ai.rates.total_rate)?;
    validate_rate("anomaly_injection.rates.fraud_rate", ai.rates.fraud_rate)?;
    validate_rate("anomaly_injection.rates.error_rate", ai.rates.error_rate)?;
    validate_rate(
        "anomaly_injection.rates.process_rate",
        ai.rates.process_rate,
    )?;

    // Validate sub-rates don't exceed total rate
    let sub_rate_sum = ai.rates.fraud_rate + ai.rates.error_rate + ai.rates.process_rate;
    if sub_rate_sum > ai.rates.total_rate + 0.001 {
        return Err(SynthError::validation(format!(
            "anomaly_injection sub-rates sum ({}) exceeds total_rate ({})",
            sub_rate_sum, ai.rates.total_rate
        )));
    }

    // Validate multi-stage scheme probabilities
    if ai.multi_stage_schemes.enabled {
        let emb = &ai.multi_stage_schemes.embezzlement;
        validate_rate("embezzlement.probability", emb.probability)?;

        let rev = &ai.multi_stage_schemes.revenue_manipulation;
        validate_rate("revenue_manipulation.probability", rev.probability)?;

        let kick = &ai.multi_stage_schemes.kickback;
        validate_rate("kickback.probability", kick.probability)?;
        if kick.inflation_min > kick.inflation_max {
            return Err(SynthError::validation(
                "kickback.inflation_min must be less than or equal to inflation_max",
            ));
        }
    }

    // Validate near-miss configuration
    if ai.near_miss.enabled {
        validate_rate("near_miss.proportion", ai.near_miss.proportion)?;
        if ai.near_miss.near_duplicate_days.min > ai.near_miss.near_duplicate_days.max {
            return Err(SynthError::validation(
                "near_miss.near_duplicate_days.min must be less than or equal to max",
            ));
        }
        if ai.near_miss.threshold_proximity_range.min > ai.near_miss.threshold_proximity_range.max {
            return Err(SynthError::validation(
                "near_miss.threshold_proximity_range.min must be less than or equal to max",
            ));
        }
        if ai.near_miss.corrected_error_lag.min > ai.near_miss.corrected_error_lag.max {
            return Err(SynthError::validation(
                "near_miss.corrected_error_lag.min must be less than or equal to max",
            ));
        }
    }

    // Validate difficulty distribution sums to ~1.0
    if ai.difficulty_classification.enabled {
        let dist = &ai.difficulty_classification.target_distribution;
        validate_sum_to_one(
            "difficulty_classification.target_distribution",
            &[
                dist.trivial,
                dist.easy,
                dist.moderate,
                dist.hard,
                dist.expert,
            ],
        )?;
    }

    // Validate context-aware configuration
    if ai.context_aware.enabled {
        let vendor = &ai.context_aware.vendor_rules;
        if vendor.new_vendor_error_multiplier < 1.0 {
            return Err(SynthError::validation(
                "vendor_rules.new_vendor_error_multiplier must be >= 1.0",
            ));
        }

        let emp = &ai.context_aware.employee_rules;
        validate_rate(
            "employee_rules.new_employee_error_rate",
            emp.new_employee_error_rate,
        )?;

        let baseline = &ai.context_aware.behavioral_baseline;
        if baseline.enabled {
            validate_positive(
                "behavioral_baseline.deviation_threshold_std",
                baseline.deviation_threshold_std,
            )?;
        }
    }

    // Validate materiality thresholds are ascending
    let mat = &ai.labeling.materiality_thresholds;
    validate_ascending(
        "materiality_thresholds",
        &[
            mat.trivial,
            mat.immaterial,
            mat.material,
            mat.highly_material,
        ],
    )?;

    Ok(())
}

/// Validate hypergraph export configuration.
fn validate_hypergraph(config: &GeneratorConfig) -> SynthResult<()> {
    let hg = &config.graph_export.hypergraph;

    if !hg.enabled {
        return Ok(());
    }

    if hg.max_nodes == 0 || hg.max_nodes > 150_000 {
        return Err(SynthError::validation(
            "hypergraph.max_nodes must be between 1 and 150000",
        ));
    }

    let valid_strategies = [
        "truncate",
        "pool_by_counterparty",
        "pool_by_time_period",
        "importance_sample",
    ];
    if !valid_strategies.contains(&hg.aggregation_strategy.as_str()) {
        return Err(SynthError::validation(format!(
            "hypergraph.aggregation_strategy must be one of: {}",
            valid_strategies.join(", ")
        )));
    }

    if hg.process_layer.docs_per_counterparty_threshold == 0 {
        return Err(SynthError::validation(
            "hypergraph.process_layer.docs_per_counterparty_threshold must be >= 1",
        ));
    }

    Ok(())
}

/// Validate fingerprint privacy configuration.
fn validate_fingerprint_privacy(config: &GeneratorConfig) -> SynthResult<()> {
    let fp = &config.fingerprint_privacy;

    if fp.epsilon <= 0.0 {
        return Err(SynthError::validation(
            "fingerprint_privacy.epsilon must be positive",
        ));
    }

    if fp.delta < 0.0 || fp.delta >= 1.0 {
        return Err(SynthError::validation(
            "fingerprint_privacy.delta must be in [0.0, 1.0)",
        ));
    }

    let valid_methods = ["naive", "advanced", "renyi_dp", "zcdp", ""];
    if !valid_methods.contains(&fp.composition_method.as_str()) {
        return Err(SynthError::validation(format!(
            "fingerprint_privacy.composition_method must be one of: naive, advanced, renyi_dp, zcdp (got '{}')",
            fp.composition_method
        )));
    }

    let valid_levels = ["minimal", "standard", "high", "maximum", "custom", ""];
    if !valid_levels.contains(&fp.level.as_str()) {
        return Err(SynthError::validation(format!(
            "fingerprint_privacy.level must be one of: minimal, standard, high, maximum, custom (got '{}')",
            fp.level
        )));
    }

    Ok(())
}

/// Validate quality gates configuration.
fn validate_quality_gates(config: &GeneratorConfig) -> SynthResult<()> {
    let qg = &config.quality_gates;

    let valid_profiles = ["strict", "default", "lenient", "custom"];
    if !valid_profiles.contains(&qg.profile.as_str()) {
        return Err(SynthError::validation(format!(
            "quality_gates.profile must be one of: strict, default, lenient, custom (got '{}')",
            qg.profile
        )));
    }

    let valid_comparisons = ["gte", "lte", "eq", "between"];
    for gate in &qg.custom_gates {
        if gate.name.is_empty() {
            return Err(SynthError::validation(
                "quality_gates.custom_gates[].name must not be empty",
            ));
        }
        if !valid_comparisons.contains(&gate.comparison.as_str()) {
            return Err(SynthError::validation(format!(
                "quality_gates.custom_gates[{}].comparison must be one of: gte, lte, eq, between (got '{}')",
                gate.name, gate.comparison
            )));
        }
        if gate.comparison == "between" && gate.upper_threshold.is_none() {
            return Err(SynthError::validation(format!(
                "quality_gates.custom_gates[{}].upper_threshold is required for 'between' comparison",
                gate.name
            )));
        }
    }

    Ok(())
}

/// Validate compliance configuration.
fn validate_compliance(config: &GeneratorConfig) -> SynthResult<()> {
    let valid_formats = ["embedded", "sidecar", "both"];
    if !valid_formats.contains(&config.compliance.content_marking.format.as_str()) {
        return Err(SynthError::validation(format!(
            "compliance.content_marking.format must be one of: embedded, sidecar, both (got '{}')",
            config.compliance.content_marking.format
        )));
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
            temporal_patterns: TemporalPatternsConfig::default(),
            vendor_network: VendorNetworkSchemaConfig::default(),
            customer_segmentation: CustomerSegmentationSchemaConfig::default(),
            relationship_strength: RelationshipStrengthSchemaConfig::default(),
            cross_process_links: CrossProcessLinksSchemaConfig::default(),
            organizational_events: OrganizationalEventsSchemaConfig::default(),
            behavioral_drift: BehavioralDriftSchemaConfig::default(),
            market_drift: MarketDriftSchemaConfig::default(),
            drift_labeling: DriftLabelingSchemaConfig::default(),
            anomaly_injection: EnhancedAnomalyConfig::default(),
            industry_specific: IndustrySpecificConfig::default(),
            fingerprint_privacy: FingerprintPrivacyConfig::default(),
            quality_gates: QualityGatesSchemaConfig::default(),
            compliance: ComplianceSchemaConfig::default(),
            webhooks: WebhookSchemaConfig::default(),
            llm: LlmSchemaConfig::default(),
            diffusion: DiffusionSchemaConfig::default(),
            causal: CausalSchemaConfig::default(),
            source_to_pay: SourceToPayConfig::default(),
            financial_reporting: FinancialReportingConfig::default(),
            hr: HrConfig::default(),
            manufacturing: ManufacturingProcessConfig::default(),
            sales_quotes: SalesQuoteConfig::default(),
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

    // ==========================================================================
    // Temporal Patterns Validation Tests
    // ==========================================================================

    #[test]
    fn test_temporal_patterns_disabled_passes() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = false;
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_temporal_patterns_enabled_with_defaults_passes() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_business_day_invalid_half_day_policy() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.business_days.enabled = true;
        config.temporal_patterns.business_days.half_day_policy = "invalid_policy".to_string();
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("half_day_policy"));
    }

    #[test]
    fn test_business_day_valid_half_day_policies() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.business_days.enabled = true;

        for policy in ["full_day", "half_day", "non_business_day"] {
            config.temporal_patterns.business_days.half_day_policy = policy.to_string();
            assert!(
                validate_config(&config).is_ok(),
                "Expected '{}' to be valid",
                policy
            );
        }
    }

    #[test]
    fn test_business_day_invalid_month_end_convention() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.business_days.enabled = true;
        config.temporal_patterns.business_days.month_end_convention = "invalid".to_string();
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("month_end_convention"));
    }

    #[test]
    fn test_period_end_invalid_model() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.period_end.model = Some("invalid_model".to_string());
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("model"));
    }

    #[test]
    fn test_period_end_valid_models() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;

        for model in ["flat", "exponential", "daily_profile", "extended_crunch"] {
            config.temporal_patterns.period_end.model = Some(model.to_string());
            assert!(
                validate_config(&config).is_ok(),
                "Expected model '{}' to be valid",
                model
            );
        }
    }

    #[test]
    fn test_period_end_invalid_decay_rate() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.period_end.model = Some("exponential".to_string());
        config.temporal_patterns.period_end.month_end = Some(PeriodEndModelSchemaConfig {
            inherit_from: None,
            additional_multiplier: None,
            start_day: Some(-10),
            base_multiplier: Some(1.0),
            peak_multiplier: Some(3.5),
            decay_rate: Some(1.5), // Invalid: > 1.0
            sustained_high_days: None,
        });
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("decay_rate"));
    }

    #[test]
    fn test_period_end_negative_multiplier() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.period_end.month_end = Some(PeriodEndModelSchemaConfig {
            inherit_from: None,
            additional_multiplier: None,
            start_day: Some(-10),
            base_multiplier: Some(1.0),
            peak_multiplier: Some(-1.0), // Invalid: negative
            decay_rate: Some(0.3),
            sustained_high_days: None,
        });
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("multiplier"));
    }

    #[test]
    fn test_processing_lag_negative_mu_allowed() {
        // Note: For log-normal distributions, mu (log-scale mean) can be any real number
        // including negative values. This test verifies that negative mu is allowed.
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.processing_lags.enabled = true;
        config.temporal_patterns.processing_lags.sales_order_lag =
            Some(LagDistributionSchemaConfig {
                mu: -1.0, // Valid: log-normal mu can be negative
                sigma: 0.8,
                min_hours: None,
                max_hours: None,
            });
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_processing_lag_negative_sigma() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.processing_lags.enabled = true;
        config.temporal_patterns.processing_lags.goods_receipt_lag =
            Some(LagDistributionSchemaConfig {
                mu: 1.5,
                sigma: -0.5, // Invalid: negative
                min_hours: None,
                max_hours: None,
            });
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("sigma"));
    }

    #[test]
    fn test_fiscal_calendar_invalid_year_start_month() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.fiscal_calendar.enabled = true;
        config.temporal_patterns.fiscal_calendar.calendar_type = "custom".to_string();
        config.temporal_patterns.fiscal_calendar.year_start_month = Some(13); // Invalid
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("year_start_month"));
    }

    #[test]
    fn test_fiscal_calendar_invalid_year_start_day() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.fiscal_calendar.enabled = true;
        config.temporal_patterns.fiscal_calendar.calendar_type = "custom".to_string();
        config.temporal_patterns.fiscal_calendar.year_start_month = Some(2);
        config.temporal_patterns.fiscal_calendar.year_start_day = Some(32); // Invalid
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("year_start_day"));
    }

    #[test]
    fn test_intraday_invalid_time_format() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.intraday.enabled = true;
        config.temporal_patterns.intraday.segments = vec![IntraDaySegmentSchemaConfig {
            name: "test".to_string(),
            start: "25:00".to_string(), // Invalid hour
            end: "10:00".to_string(),
            multiplier: 1.5,
            posting_type: "both".to_string(),
        }];
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HH:MM format"));
    }

    #[test]
    fn test_intraday_invalid_posting_type() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.intraday.enabled = true;
        config.temporal_patterns.intraday.segments = vec![IntraDaySegmentSchemaConfig {
            name: "test".to_string(),
            start: "08:00".to_string(),
            end: "10:00".to_string(),
            multiplier: 1.5,
            posting_type: "invalid".to_string(),
        }];
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("posting_type"));
    }

    #[test]
    fn test_intraday_negative_multiplier() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.intraday.enabled = true;
        config.temporal_patterns.intraday.segments = vec![IntraDaySegmentSchemaConfig {
            name: "test".to_string(),
            start: "08:00".to_string(),
            end: "10:00".to_string(),
            multiplier: -1.0, // Invalid
            posting_type: "both".to_string(),
        }];
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("multiplier"));
    }

    #[test]
    fn test_timezone_invalid_default() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.timezones.enabled = true;
        config.temporal_patterns.timezones.default_timezone = "Invalid/Timezone".to_string();
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timezone"));
    }

    #[test]
    fn test_timezone_valid_iana_names() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.timezones.enabled = true;
        config.temporal_patterns.timezones.consolidation_timezone = "UTC".to_string();

        for tz in [
            "America/New_York",
            "Europe/London",
            "Asia/Tokyo",
            "UTC",
            "Pacific/Auckland",
        ] {
            config.temporal_patterns.timezones.default_timezone = tz.to_string();
            let result = validate_config(&config);
            assert!(
                result.is_ok(),
                "Expected timezone '{}' to be valid, got error: {:?}",
                tz,
                result.err()
            );
        }
    }

    #[test]
    fn test_timezone_invalid_entity_mapping() {
        let mut config = minimal_valid_config();
        config.temporal_patterns.enabled = true;
        config.temporal_patterns.timezones.enabled = true;
        config.temporal_patterns.timezones.entity_mappings = vec![EntityTimezoneMapping {
            pattern: "EU_*".to_string(),
            timezone: "Invalid/TZ".to_string(),
        }];
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timezone"));
    }
}
