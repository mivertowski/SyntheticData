//! Tax Code Generator.
//!
//! Generates tax jurisdictions and tax codes with built-in rate tables
//! for common countries. Supports VAT/GST (EU, UK, SG, AU, JP, IN, BR, CA),
//! sales tax (US states), and config-driven rate overrides.

use chrono::NaiveDate;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_config::schema::TaxConfig;
use datasynth_core::models::{JurisdictionType, TaxCode, TaxJurisdiction, TaxType};

// ---------------------------------------------------------------------------
// Built-in rate tables
// ---------------------------------------------------------------------------

/// US state sales tax rates (top 10 nexus states).
const US_STATE_RATES: &[(&str, &str, &str)] = &[
    ("CA", "California", "0.0725"),
    ("NY", "New York", "0.08"),
    ("TX", "Texas", "0.0625"),
    ("FL", "Florida", "0.06"),
    ("WA", "Washington", "0.065"),
    ("IL", "Illinois", "0.0625"),
    ("PA", "Pennsylvania", "0.06"),
    ("OH", "Ohio", "0.0575"),
    ("NJ", "New Jersey", "0.06625"),
    ("GA", "Georgia", "0.04"),
];

/// Country-level VAT/GST rate table.
///
/// Tuple: (country_code, country_name, tax_type, standard_rate, reduced_rate_or_none)
const COUNTRY_RATES: &[(&str, &str, &str, &str, Option<&str>)] = &[
    ("DE", "Germany", "vat", "0.19", Some("0.07")),
    ("GB", "United Kingdom", "vat", "0.20", Some("0.05")),
    ("FR", "France", "vat", "0.20", Some("0.055")),
    ("IT", "Italy", "vat", "0.22", Some("0.10")),
    ("ES", "Spain", "vat", "0.21", Some("0.10")),
    ("NL", "Netherlands", "vat", "0.21", Some("0.09")),
    ("SG", "Singapore", "gst", "0.09", None),
    ("AU", "Australia", "gst", "0.10", None),
    ("JP", "Japan", "gst", "0.10", Some("0.08")),
    ("IN", "India", "gst", "0.18", Some("0.05")),
    ("BR", "Brazil", "vat", "0.17", None),
    ("CA", "Canada", "gst", "0.05", None),
];

/// Indian state names for GST sub-jurisdictions.
const INDIA_STATES: &[(&str, &str)] = &[
    ("MH", "Maharashtra"),
    ("DL", "Delhi"),
    ("KA", "Karnataka"),
    ("TN", "Tamil Nadu"),
    ("GJ", "Gujarat"),
    ("UP", "Uttar Pradesh"),
    ("WB", "West Bengal"),
    ("RJ", "Rajasthan"),
    ("TG", "Telangana"),
    ("KL", "Kerala"),
];

/// German Bundeslaender (states).
const GERMANY_STATES: &[(&str, &str)] = &[
    ("BW", "Baden-Wuerttemberg"),
    ("BY", "Bavaria"),
    ("BE", "Berlin"),
    ("BB", "Brandenburg"),
    ("HB", "Bremen"),
    ("HH", "Hamburg"),
    ("HE", "Hesse"),
    ("MV", "Mecklenburg-Vorpommern"),
    ("NI", "Lower Saxony"),
    ("NW", "North Rhine-Westphalia"),
    ("RP", "Rhineland-Palatinate"),
    ("SL", "Saarland"),
    ("SN", "Saxony"),
    ("ST", "Saxony-Anhalt"),
    ("SH", "Schleswig-Holstein"),
    ("TH", "Thuringia"),
];

/// Canadian provinces for GST/HST sub-jurisdictions.
const CANADA_PROVINCES: &[(&str, &str, &str)] = &[
    ("ON", "Ontario", "0.13"),
    ("BC", "British Columbia", "0.12"),
    ("QC", "Quebec", "0.14975"),
    ("AB", "Alberta", "0.05"),
    ("NS", "Nova Scotia", "0.15"),
    ("NB", "New Brunswick", "0.15"),
    ("MB", "Manitoba", "0.12"),
    ("SK", "Saskatchewan", "0.11"),
    ("NL", "Newfoundland and Labrador", "0.15"),
    ("PE", "Prince Edward Island", "0.15"),
];

/// India GST slabs beyond the standard 18%.
const INDIA_GST_SLABS: &[(&str, &str)] = &[
    ("0.05", "GST 5% slab"),
    ("0.12", "GST 12% slab"),
    ("0.18", "GST 18% slab"),
    ("0.28", "GST 28% slab"),
];

/// Default effective date for all generated tax codes.
fn default_effective_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2020, 1, 1).expect("valid date")
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates tax jurisdictions and tax codes from built-in rate tables.
///
/// The generator reads `TaxConfig` to determine which countries to produce
/// jurisdictions for, whether to include sub-national jurisdictions (US states,
/// Canadian provinces, etc.), and whether config-provided rate overrides should
/// replace the built-in defaults.
///
/// # Examples
///
/// ```
/// use datasynth_generators::tax::TaxCodeGenerator;
///
/// let mut gen = TaxCodeGenerator::new(42);
/// let (jurisdictions, codes) = gen.generate();
/// assert!(!jurisdictions.is_empty());
/// assert!(!codes.is_empty());
/// ```
pub struct TaxCodeGenerator {
    rng: ChaCha8Rng,
    config: TaxConfig,
}

impl TaxCodeGenerator {
    /// Creates a new generator with default configuration.
    ///
    /// Default config generates jurisdictions for US, DE, and GB.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config: TaxConfig::default(),
        }
    }

    /// Creates a new generator with custom configuration.
    pub fn with_config(seed: u64, config: TaxConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
        }
    }

    /// Generates tax jurisdictions and tax codes.
    ///
    /// Returns a tuple of `(Vec<TaxJurisdiction>, Vec<TaxCode>)`.
    pub fn generate(&mut self) -> (Vec<TaxJurisdiction>, Vec<TaxCode>) {
        let countries = self.resolve_countries();
        let include_subnational = self.config.jurisdictions.include_subnational;

        let mut jurisdictions = Vec::new();
        let mut codes = Vec::new();
        let mut code_counter: u32 = 1;

        for country in &countries {
            let cc = country.as_str();
            match cc {
                "US" => self.generate_us(
                    include_subnational,
                    &mut jurisdictions,
                    &mut codes,
                    &mut code_counter,
                ),
                _ => self.generate_country(
                    cc,
                    include_subnational,
                    &mut jurisdictions,
                    &mut codes,
                    &mut code_counter,
                ),
            }
        }

        (jurisdictions, codes)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Resolves the list of country codes to generate.
    fn resolve_countries(&self) -> Vec<String> {
        if self.config.jurisdictions.countries.is_empty() {
            vec!["US".into(), "DE".into(), "GB".into()]
        } else {
            self.config.jurisdictions.countries.clone()
        }
    }

    /// Generates US federal + state jurisdictions and sales tax codes.
    fn generate_us(
        &mut self,
        include_subnational: bool,
        jurisdictions: &mut Vec<TaxJurisdiction>,
        codes: &mut Vec<TaxCode>,
        counter: &mut u32,
    ) {
        let federal_id = "JUR-US".to_string();

        // Federal jurisdiction (no federal sales tax, but anchor the hierarchy)
        jurisdictions.push(TaxJurisdiction::new(
            &federal_id,
            "United States - Federal",
            "US",
            JurisdictionType::Federal,
        ));

        if !include_subnational {
            return;
        }

        // Determine which states to generate
        let nexus_states = &self.config.sales_tax.nexus_states;

        for &(state_code, state_name, rate_str) in US_STATE_RATES {
            // If nexus_states is non-empty, only generate those states
            if !nexus_states.is_empty()
                && !nexus_states
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(state_code))
            {
                continue;
            }

            let jur_id = format!("JUR-US-{state_code}");

            jurisdictions.push(
                TaxJurisdiction::new(&jur_id, state_name, "US", JurisdictionType::State)
                    .with_region_code(state_code)
                    .with_parent_jurisdiction_id(&federal_id),
            );

            let rate: Decimal = rate_str.parse().expect("valid decimal");
            let code_id = format!("TC-{counter:04}");
            let code_mnemonic = format!("ST-{state_code}");
            let description = format!("{state_name} Sales Tax {}", format_rate_pct(rate));

            codes.push(TaxCode::new(
                code_id,
                code_mnemonic,
                description,
                TaxType::SalesTax,
                rate,
                &jur_id,
                default_effective_date(),
            ));
            *counter += 1;
        }
    }

    /// Generates a non-US country's federal jurisdiction, tax codes, and
    /// optionally sub-national jurisdictions.
    fn generate_country(
        &mut self,
        country_code: &str,
        include_subnational: bool,
        jurisdictions: &mut Vec<TaxJurisdiction>,
        codes: &mut Vec<TaxCode>,
        counter: &mut u32,
    ) {
        // Look up country in the built-in rate table
        let entry = COUNTRY_RATES
            .iter()
            .find(|(cc, _, _, _, _)| *cc == country_code);

        let (country_name, tax_type_str, default_std_rate_str, default_reduced_str) = match entry {
            Some((_, name, tt, std_rate, reduced)) => (*name, *tt, *std_rate, *reduced),
            None => {
                // Unknown country - skip silently (config may list countries
                // for which we don't have built-in rates yet)
                return;
            }
        };

        let tax_type = match tax_type_str {
            "gst" => TaxType::Gst,
            _ => TaxType::Vat,
        };

        let is_vat_gst = matches!(tax_type, TaxType::Vat | TaxType::Gst);

        let federal_id = format!("JUR-{country_code}");

        // Federal jurisdiction
        jurisdictions.push(
            TaxJurisdiction::new(
                &federal_id,
                format!("{country_name} - Federal"),
                country_code,
                JurisdictionType::Federal,
            )
            .with_vat_registered(is_vat_gst),
        );

        // Resolve standard rate (config override > built-in)
        let std_rate = self.resolve_standard_rate(country_code, default_std_rate_str);
        let reduced_rate = self.resolve_reduced_rate(country_code, default_reduced_str);

        // Standard-rate code
        let std_code_id = format!("TC-{counter:04}");
        let std_mnemonic = format!(
            "{}-STD-{}",
            if tax_type == TaxType::Gst {
                "GST"
            } else {
                "VAT"
            },
            country_code
        );
        let std_desc = format!(
            "{country_name} {} Standard {}",
            if tax_type == TaxType::Gst {
                "GST"
            } else {
                "VAT"
            },
            format_rate_pct(std_rate)
        );

        let mut std_code = TaxCode::new(
            std_code_id,
            std_mnemonic,
            std_desc,
            tax_type,
            std_rate,
            &federal_id,
            default_effective_date(),
        );

        // For EU countries, enable reverse charge on the standard code
        if is_eu_country(country_code) && self.config.vat_gst.reverse_charge {
            std_code = std_code.with_reverse_charge(true);
        }

        codes.push(std_code);
        *counter += 1;

        // Reduced-rate code (if applicable)
        if let Some(red_rate) = reduced_rate {
            let red_code_id = format!("TC-{counter:04}");
            let red_mnemonic = format!(
                "{}-RED-{}",
                if tax_type == TaxType::Gst {
                    "GST"
                } else {
                    "VAT"
                },
                country_code
            );
            let red_desc = format!(
                "{country_name} {} Reduced {}",
                if tax_type == TaxType::Gst {
                    "GST"
                } else {
                    "VAT"
                },
                format_rate_pct(red_rate)
            );

            codes.push(TaxCode::new(
                red_code_id,
                red_mnemonic,
                red_desc,
                tax_type,
                red_rate,
                &federal_id,
                default_effective_date(),
            ));
            *counter += 1;
        }

        // Zero-rate code for GB (food, children's clothing)
        if country_code == "GB" {
            let zero_code_id = format!("TC-{counter:04}");
            codes.push(TaxCode::new(
                zero_code_id,
                format!("VAT-ZERO-{country_code}"),
                format!("{country_name} VAT Zero Rate"),
                TaxType::Vat,
                dec!(0),
                &federal_id,
                default_effective_date(),
            ));
            *counter += 1;
        }

        // Exempt code
        let exempt_code_id = format!("TC-{counter:04}");
        let exempt_mnemonic = format!(
            "{}-EX-{}",
            if tax_type == TaxType::Gst {
                "GST"
            } else {
                "VAT"
            },
            country_code
        );
        codes.push(
            TaxCode::new(
                exempt_code_id,
                exempt_mnemonic,
                format!("{country_name} Tax Exempt"),
                tax_type,
                dec!(0),
                &federal_id,
                default_effective_date(),
            )
            .with_exempt(true),
        );
        *counter += 1;

        // Sub-national jurisdictions
        if include_subnational {
            self.generate_subnational(
                country_code,
                &federal_id,
                tax_type,
                jurisdictions,
                codes,
                counter,
            );
        }
    }

    /// Generates sub-national jurisdictions for countries that have them.
    fn generate_subnational(
        &mut self,
        country_code: &str,
        federal_id: &str,
        _tax_type: TaxType,
        jurisdictions: &mut Vec<TaxJurisdiction>,
        codes: &mut Vec<TaxCode>,
        counter: &mut u32,
    ) {
        match country_code {
            "IN" => {
                // India: state-level GST jurisdictions + slab codes
                for &(state_code, state_name) in INDIA_STATES {
                    let jur_id = format!("JUR-IN-{state_code}");
                    jurisdictions.push(
                        TaxJurisdiction::new(&jur_id, state_name, "IN", JurisdictionType::State)
                            .with_region_code(state_code)
                            .with_parent_jurisdiction_id(federal_id)
                            .with_vat_registered(true),
                    );
                }

                // India GST slab codes (attached to federal jurisdiction)
                for &(rate_str, label) in INDIA_GST_SLABS {
                    let rate: Decimal = rate_str.parse().expect("valid decimal");
                    let code_id = format!("TC-{counter:04}");
                    let pct = format_rate_pct(rate);
                    codes.push(TaxCode::new(
                        code_id,
                        format!("GST-SLAB-{pct}"),
                        label,
                        TaxType::Gst,
                        rate,
                        federal_id,
                        default_effective_date(),
                    ));
                    *counter += 1;
                }
            }
            "DE" => {
                // Germany: Bundeslaender (no separate tax rates, but jurisdiction hierarchy)
                for &(state_code, state_name) in GERMANY_STATES {
                    let jur_id = format!("JUR-DE-{state_code}");
                    jurisdictions.push(
                        TaxJurisdiction::new(&jur_id, state_name, "DE", JurisdictionType::State)
                            .with_region_code(state_code)
                            .with_parent_jurisdiction_id(federal_id)
                            .with_vat_registered(true),
                    );
                }
            }
            "CA" => {
                // Canada: provincial HST/PST combined rates
                for &(prov_code, prov_name, combined_rate_str) in CANADA_PROVINCES {
                    let jur_id = format!("JUR-CA-{prov_code}");
                    jurisdictions.push(
                        TaxJurisdiction::new(&jur_id, prov_name, "CA", JurisdictionType::State)
                            .with_region_code(prov_code)
                            .with_parent_jurisdiction_id(federal_id)
                            .with_vat_registered(true),
                    );

                    let combined_rate: Decimal = combined_rate_str.parse().expect("valid decimal");
                    let code_id = format!("TC-{counter:04}");
                    codes.push(TaxCode::new(
                        code_id,
                        format!("HST-{prov_code}"),
                        format!("{prov_name} HST/GST+PST {}", format_rate_pct(combined_rate)),
                        TaxType::Gst,
                        combined_rate,
                        &jur_id,
                        default_effective_date(),
                    ));
                    *counter += 1;
                }
            }
            _ => {
                // No sub-national jurisdictions for other countries
            }
        }
    }

    // -----------------------------------------------------------------------
    // Country-pack-driven generation
    // -----------------------------------------------------------------------

    /// Generates tax jurisdictions and tax codes from a [`CountryPack`].
    ///
    /// This is an **alternative** to [`generate()`](Self::generate) that reads
    /// tax rates and sub-national jurisdictions from a country pack instead of
    /// using the hardcoded constants. If the pack carries no meaningful tax data
    /// (e.g. `standard_rate == 0.0` and no sub-national entries), the method
    /// returns empty vectors so the caller can fall back to `generate()`.
    ///
    /// # Arguments
    ///
    /// * `pack` - The country pack whose tax data should drive generation.
    /// * `company_code` - Company code used to prefix generated IDs.
    /// * `fiscal_year` - Fiscal year; used to derive the effective date
    ///   (January 1 of that year).
    pub fn generate_from_country_pack(
        &mut self,
        pack: &datasynth_core::CountryPack,
        company_code: &str,
        fiscal_year: i32,
    ) -> (Vec<TaxJurisdiction>, Vec<TaxCode>) {
        let tax = &pack.tax;
        let country_code = pack.country_code.as_str();
        let country_name = if pack.country_name.is_empty() {
            country_code
        } else {
            pack.country_name.as_str()
        };

        // Guard: if the pack has no meaningful tax data, return empty.
        let has_vat = tax.vat.standard_rate > 0.0;
        let has_cit = tax.corporate_income_tax.standard_rate > 0.0;
        let has_subnational = !tax.subnational.is_empty();

        if !has_vat && !has_cit && !has_subnational {
            return (Vec::new(), Vec::new());
        }

        let effective_date = NaiveDate::from_ymd_opt(fiscal_year, 1, 1)
            .unwrap_or_else(default_effective_date);

        let mut jurisdictions = Vec::new();
        let mut codes = Vec::new();
        let mut counter: u32 = 1;

        // -------------------------------------------------------------------
        // Federal jurisdiction
        // -------------------------------------------------------------------
        let federal_id = format!("JUR-{company_code}-{country_code}");

        jurisdictions.push(
            TaxJurisdiction::new(
                &federal_id,
                format!("{country_name} - Federal"),
                country_code,
                JurisdictionType::Federal,
            )
            .with_vat_registered(has_vat),
        );

        // -------------------------------------------------------------------
        // VAT/GST codes from pack
        // -------------------------------------------------------------------
        if has_vat {
            let std_rate = Decimal::try_from(tax.vat.standard_rate)
                .unwrap_or_else(|_| dec!(0));

            // Determine tax type: treat country packs as VAT by default,
            // but use GST for known GST countries.
            let tax_type = if is_gst_country(country_code) {
                TaxType::Gst
            } else {
                TaxType::Vat
            };

            let type_label = if tax_type == TaxType::Gst {
                "GST"
            } else {
                "VAT"
            };

            // Standard rate code
            let std_code_id = format!("TC-{company_code}-{counter:04}");
            let std_mnemonic = format!("{type_label}-STD-{country_code}");
            let std_desc = format!(
                "{country_name} {type_label} Standard {}",
                format_rate_pct(std_rate)
            );

            let mut std_code = TaxCode::new(
                std_code_id,
                std_mnemonic,
                std_desc,
                tax_type,
                std_rate,
                &federal_id,
                effective_date,
            );

            if tax.vat.reverse_charge_applicable {
                std_code = std_code.with_reverse_charge(true);
            }

            codes.push(std_code);
            counter += 1;

            // Reduced rate codes
            for reduced in &tax.vat.reduced_rates {
                if reduced.rate <= 0.0 {
                    continue;
                }
                let red_rate = Decimal::try_from(reduced.rate)
                    .unwrap_or_else(|_| dec!(0));

                let label_suffix = if reduced.label.is_empty() {
                    format_rate_pct(red_rate)
                } else {
                    reduced.label.clone()
                };

                let red_code_id = format!("TC-{company_code}-{counter:04}");
                let red_mnemonic = format!("{type_label}-RED-{country_code}-{counter}");
                let red_desc = format!(
                    "{country_name} {type_label} Reduced {label_suffix} {}",
                    format_rate_pct(red_rate)
                );

                codes.push(TaxCode::new(
                    red_code_id,
                    red_mnemonic,
                    red_desc,
                    tax_type,
                    red_rate,
                    &federal_id,
                    effective_date,
                ));
                counter += 1;
            }

            // Zero-rated code (if the pack lists zero-rated categories)
            if !tax.vat.zero_rated.is_empty() {
                let zero_code_id = format!("TC-{company_code}-{counter:04}");
                codes.push(TaxCode::new(
                    zero_code_id,
                    format!("{type_label}-ZERO-{country_code}"),
                    format!("{country_name} {type_label} Zero Rate"),
                    tax_type,
                    dec!(0),
                    &federal_id,
                    effective_date,
                ));
                counter += 1;
            }

            // Exempt code (if the pack lists exempt categories)
            if !tax.vat.exempt.is_empty() {
                let exempt_code_id = format!("TC-{company_code}-{counter:04}");
                codes.push(
                    TaxCode::new(
                        exempt_code_id,
                        format!("{type_label}-EX-{country_code}"),
                        format!("{country_name} Tax Exempt"),
                        tax_type,
                        dec!(0),
                        &federal_id,
                        effective_date,
                    )
                    .with_exempt(true),
                );
                counter += 1;
            }
        }

        // -------------------------------------------------------------------
        // Corporate income tax code
        // -------------------------------------------------------------------
        if has_cit {
            let cit_rate = Decimal::try_from(tax.corporate_income_tax.standard_rate)
                .unwrap_or_else(|_| dec!(0));

            let cit_code_id = format!("TC-{company_code}-{counter:04}");
            codes.push(TaxCode::new(
                cit_code_id,
                format!("CIT-{country_code}"),
                format!(
                    "{country_name} Corporate Income Tax {}",
                    format_rate_pct(cit_rate)
                ),
                TaxType::IncomeTax,
                cit_rate,
                &federal_id,
                effective_date,
            ));
            counter += 1;
        }

        // -------------------------------------------------------------------
        // Sub-national jurisdictions from pack
        // -------------------------------------------------------------------
        for sub in &tax.subnational {
            if sub.code.is_empty() {
                continue;
            }

            let jur_id = format!("JUR-{company_code}-{country_code}-{}", sub.code);

            let sub_name = if sub.name.is_empty() {
                &sub.code
            } else {
                &sub.name
            };

            jurisdictions.push(
                TaxJurisdiction::new(
                    &jur_id,
                    sub_name,
                    country_code,
                    JurisdictionType::State,
                )
                .with_region_code(&sub.code)
                .with_parent_jurisdiction_id(&federal_id)
                .with_vat_registered(has_vat),
            );

            // Generate a tax code for this sub-national jurisdiction if it has a rate
            if sub.rate > 0.0 {
                let sub_rate = Decimal::try_from(sub.rate)
                    .unwrap_or_else(|_| dec!(0));

                let sub_tax_type = match sub.tax_type.as_str() {
                    "sales_tax" | "SalesTax" => TaxType::SalesTax,
                    "gst" | "Gst" | "GST" => TaxType::Gst,
                    "vat" | "Vat" | "VAT" => TaxType::Vat,
                    "income_tax" | "IncomeTax" => TaxType::IncomeTax,
                    _ => {
                        // Infer from country: US → SalesTax, else VAT/GST
                        if country_code == "US" {
                            TaxType::SalesTax
                        } else if is_gst_country(country_code) {
                            TaxType::Gst
                        } else {
                            TaxType::Vat
                        }
                    }
                };

                let type_label = match sub_tax_type {
                    TaxType::SalesTax => "ST",
                    TaxType::Gst => "GST",
                    TaxType::Vat => "VAT",
                    TaxType::IncomeTax => "CIT",
                    _ => "TAX",
                };

                let sub_code_id = format!("TC-{company_code}-{counter:04}");
                let sub_mnemonic = format!("{type_label}-{}", sub.code);
                let sub_desc = format!(
                    "{sub_name} {} {}",
                    type_label,
                    format_rate_pct(sub_rate)
                );

                codes.push(TaxCode::new(
                    sub_code_id,
                    sub_mnemonic,
                    sub_desc,
                    sub_tax_type,
                    sub_rate,
                    &jur_id,
                    effective_date,
                ));
                counter += 1;
            }
        }

        // Suppress unused-variable warning for the RNG (deterministic but unused
        // in this path; kept for future jitter / randomised selection).
        let _ = self.rng.gen::<u32>();

        (jurisdictions, codes)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Resolves the standard rate for a country, applying config overrides.
    fn resolve_standard_rate(&self, country_code: &str, default_str: &str) -> Decimal {
        if let Some(&override_rate) = self.config.vat_gst.standard_rates.get(country_code) {
            Decimal::try_from(override_rate)
                .unwrap_or_else(|_| default_str.parse().expect("valid decimal"))
        } else {
            default_str.parse().expect("valid decimal")
        }
    }

    /// Resolves the reduced rate for a country, applying config overrides.
    fn resolve_reduced_rate(
        &self,
        country_code: &str,
        default_opt: Option<&str>,
    ) -> Option<Decimal> {
        if let Some(&override_rate) = self.config.vat_gst.reduced_rates.get(country_code) {
            Some(Decimal::try_from(override_rate).unwrap_or_else(|_| {
                default_opt
                    .map(|s| s.parse().expect("valid decimal"))
                    .unwrap_or(dec!(0))
            }))
        } else {
            default_opt.map(|s| s.parse().expect("valid decimal"))
        }
    }
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

/// Returns `true` for EU member state country codes.
fn is_eu_country(cc: &str) -> bool {
    matches!(
        cc,
        "DE" | "FR"
            | "IT"
            | "ES"
            | "NL"
            | "BE"
            | "AT"
            | "PT"
            | "IE"
            | "FI"
            | "SE"
            | "DK"
            | "PL"
            | "CZ"
            | "RO"
            | "HU"
            | "BG"
            | "HR"
            | "SK"
            | "SI"
            | "LT"
            | "LV"
            | "EE"
            | "CY"
            | "LU"
            | "MT"
            | "EL"
            | "GR"
    )
}

/// Returns `true` for countries that use GST rather than VAT.
fn is_gst_country(cc: &str) -> bool {
    matches!(cc, "SG" | "AU" | "NZ" | "IN" | "CA" | "MY" | "JP")
}

/// Formats a decimal rate as a percentage string (e.g., 0.19 -> "19%").
fn format_rate_pct(rate: Decimal) -> String {
    let pct = rate * dec!(100);
    // Strip trailing zeros for cleaner display
    let s = pct.normalize().to_string();
    format!("{s}%")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn test_generate_default_countries() {
        let mut gen = TaxCodeGenerator::new(42);
        let (jurisdictions, codes) = gen.generate();

        // Default countries: US, DE, GB
        let countries: Vec<&str> = jurisdictions
            .iter()
            .map(|j| j.country_code.as_str())
            .collect();
        assert!(countries.contains(&"US"), "Should contain US");
        assert!(countries.contains(&"DE"), "Should contain DE");
        assert!(countries.contains(&"GB"), "Should contain GB");

        // Should have at least one jurisdiction per country
        assert!(
            jurisdictions
                .iter()
                .any(|j| j.country_code == "US" && j.jurisdiction_type == JurisdictionType::Federal),
            "US should have a federal jurisdiction"
        );
        assert!(
            jurisdictions
                .iter()
                .any(|j| j.country_code == "DE" && j.jurisdiction_type == JurisdictionType::Federal),
            "DE should have a federal jurisdiction"
        );
        assert!(
            jurisdictions
                .iter()
                .any(|j| j.country_code == "GB" && j.jurisdiction_type == JurisdictionType::Federal),
            "GB should have a federal jurisdiction"
        );

        // Should have tax codes generated
        assert!(!codes.is_empty(), "Should produce tax codes");
    }

    #[test]
    fn test_generate_specific_countries() {
        let mut config = TaxConfig::default();
        config.jurisdictions.countries = vec!["SG".into(), "JP".into()];

        let mut gen = TaxCodeGenerator::with_config(42, config);
        let (jurisdictions, codes) = gen.generate();

        let country_codes: Vec<&str> = jurisdictions
            .iter()
            .map(|j| j.country_code.as_str())
            .collect();

        assert!(country_codes.contains(&"SG"), "Should contain SG");
        assert!(country_codes.contains(&"JP"), "Should contain JP");
        assert!(!country_codes.contains(&"US"), "Should NOT contain US");
        assert!(!country_codes.contains(&"DE"), "Should NOT contain DE");

        // SG should have GST codes
        let sg_codes: Vec<&TaxCode> = codes
            .iter()
            .filter(|c| c.jurisdiction_id == "JUR-SG")
            .collect();
        assert!(!sg_codes.is_empty(), "SG should have tax codes");
        assert!(
            sg_codes.iter().any(|c| c.tax_type == TaxType::Gst),
            "SG codes should be GST type"
        );

        // JP should have standard and reduced rates
        let jp_codes: Vec<&TaxCode> = codes
            .iter()
            .filter(|c| c.jurisdiction_id == "JUR-JP")
            .collect();
        let jp_rates: Vec<Decimal> = jp_codes
            .iter()
            .filter(|c| !c.is_exempt)
            .map(|c| c.rate)
            .collect();
        assert!(
            jp_rates.contains(&dec!(0.10)),
            "JP should have standard rate 10%"
        );
        assert!(
            jp_rates.contains(&dec!(0.08)),
            "JP should have reduced rate 8%"
        );
    }

    #[test]
    fn test_us_sales_tax_codes() {
        let mut config = TaxConfig::default();
        config.jurisdictions.countries = vec!["US".into()];
        config.jurisdictions.include_subnational = true;

        let mut gen = TaxCodeGenerator::with_config(42, config);
        let (jurisdictions, codes) = gen.generate();

        // Should have federal + state jurisdictions
        let federal = jurisdictions
            .iter()
            .find(|j| j.id == "JUR-US")
            .expect("US federal jurisdiction");
        assert_eq!(federal.jurisdiction_type, JurisdictionType::Federal);

        let state_jurs: Vec<&TaxJurisdiction> = jurisdictions
            .iter()
            .filter(|j| j.country_code == "US" && j.jurisdiction_type == JurisdictionType::State)
            .collect();
        assert_eq!(
            state_jurs.len(),
            10,
            "Should have 10 US state jurisdictions"
        );

        // Check specific state rates
        let ca_code = codes
            .iter()
            .find(|c| c.code == "ST-CA")
            .expect("California sales tax code");
        assert_eq!(ca_code.rate, dec!(0.0725));
        assert_eq!(ca_code.tax_type, TaxType::SalesTax);

        let ny_code = codes
            .iter()
            .find(|c| c.code == "ST-NY")
            .expect("New York sales tax code");
        assert_eq!(ny_code.rate, dec!(0.08));

        let tx_code = codes
            .iter()
            .find(|c| c.code == "ST-TX")
            .expect("Texas sales tax code");
        assert_eq!(tx_code.rate, dec!(0.0625));
    }

    #[test]
    fn test_eu_vat_codes() {
        let mut config = TaxConfig::default();
        config.jurisdictions.countries = vec!["DE".into(), "GB".into(), "FR".into()];

        let mut gen = TaxCodeGenerator::with_config(42, config);
        let (_jurisdictions, codes) = gen.generate();

        // DE: standard 19%, reduced 7%
        let de_std = codes
            .iter()
            .find(|c| c.code == "VAT-STD-DE")
            .expect("DE standard VAT code");
        assert_eq!(de_std.rate, dec!(0.19));
        assert_eq!(de_std.tax_type, TaxType::Vat);
        assert!(de_std.is_reverse_charge, "DE should have reverse charge");

        let de_red = codes
            .iter()
            .find(|c| c.code == "VAT-RED-DE")
            .expect("DE reduced VAT code");
        assert_eq!(de_red.rate, dec!(0.07));

        // GB: standard 20%, reduced 5%, zero rate
        let gb_std = codes
            .iter()
            .find(|c| c.code == "VAT-STD-GB")
            .expect("GB standard VAT code");
        assert_eq!(gb_std.rate, dec!(0.20));
        assert!(
            !gb_std.is_reverse_charge,
            "GB should NOT have reverse charge (not EU)"
        );

        let gb_red = codes
            .iter()
            .find(|c| c.code == "VAT-RED-GB")
            .expect("GB reduced VAT code");
        assert_eq!(gb_red.rate, dec!(0.05));

        let gb_zero = codes
            .iter()
            .find(|c| c.code == "VAT-ZERO-GB")
            .expect("GB zero-rate VAT code");
        assert_eq!(gb_zero.rate, dec!(0));

        // FR: standard 20%, reduced 5.5%
        let fr_std = codes
            .iter()
            .find(|c| c.code == "VAT-STD-FR")
            .expect("FR standard VAT code");
        assert_eq!(fr_std.rate, dec!(0.20));
        assert!(fr_std.is_reverse_charge, "FR should have reverse charge");

        let fr_red = codes
            .iter()
            .find(|c| c.code == "VAT-RED-FR")
            .expect("FR reduced VAT code");
        assert_eq!(fr_red.rate, dec!(0.055));
    }

    #[test]
    fn test_deterministic() {
        let mut gen1 = TaxCodeGenerator::new(12345);
        let (jur1, codes1) = gen1.generate();

        let mut gen2 = TaxCodeGenerator::new(12345);
        let (jur2, codes2) = gen2.generate();

        assert_eq!(jur1.len(), jur2.len(), "Same number of jurisdictions");
        assert_eq!(codes1.len(), codes2.len(), "Same number of codes");

        for (j1, j2) in jur1.iter().zip(jur2.iter()) {
            assert_eq!(j1.id, j2.id);
            assert_eq!(j1.name, j2.name);
            assert_eq!(j1.country_code, j2.country_code);
            assert_eq!(j1.jurisdiction_type, j2.jurisdiction_type);
            assert_eq!(j1.vat_registered, j2.vat_registered);
        }

        for (c1, c2) in codes1.iter().zip(codes2.iter()) {
            assert_eq!(c1.id, c2.id);
            assert_eq!(c1.code, c2.code);
            assert_eq!(c1.rate, c2.rate);
            assert_eq!(c1.tax_type, c2.tax_type);
        }
    }

    #[test]
    fn test_config_rate_override() {
        let mut config = TaxConfig::default();
        config.jurisdictions.countries = vec!["DE".into()];
        config.vat_gst.standard_rates.insert("DE".into(), 0.25);

        let mut gen = TaxCodeGenerator::with_config(42, config);
        let (_jurisdictions, codes) = gen.generate();

        let de_std = codes
            .iter()
            .find(|c| c.code == "VAT-STD-DE")
            .expect("DE standard VAT code");
        assert_eq!(
            de_std.rate,
            dec!(0.25),
            "Config override should replace built-in rate"
        );
    }

    #[test]
    fn test_subnational_generation() {
        let mut config = TaxConfig::default();
        config.jurisdictions.countries = vec!["US".into(), "IN".into(), "CA".into()];
        config.jurisdictions.include_subnational = true;

        let mut gen = TaxCodeGenerator::with_config(42, config);
        let (jurisdictions, codes) = gen.generate();

        // US: 1 federal + 10 states
        let us_jurs: Vec<&TaxJurisdiction> = jurisdictions
            .iter()
            .filter(|j| j.country_code == "US")
            .collect();
        assert_eq!(us_jurs.len(), 11, "US: 1 federal + 10 states");

        // IN: 1 federal + 10 states
        let in_jurs: Vec<&TaxJurisdiction> = jurisdictions
            .iter()
            .filter(|j| j.country_code == "IN")
            .collect();
        assert_eq!(in_jurs.len(), 11, "IN: 1 federal + 10 states");

        // IN state jurisdictions should be VAT-registered
        let in_states: Vec<&TaxJurisdiction> = in_jurs
            .iter()
            .filter(|j| j.jurisdiction_type == JurisdictionType::State)
            .copied()
            .collect();
        assert!(
            in_states.iter().all(|j| j.vat_registered),
            "IN states should be VAT-registered"
        );

        // India should have GST slab codes
        let in_slab_codes: Vec<&TaxCode> = codes
            .iter()
            .filter(|c| c.code.starts_with("GST-SLAB-"))
            .collect();
        assert_eq!(in_slab_codes.len(), 4, "India should have 4 GST slab codes");

        // CA: 1 federal + 10 provinces
        let ca_jurs: Vec<&TaxJurisdiction> = jurisdictions
            .iter()
            .filter(|j| j.country_code == "CA")
            .collect();
        assert_eq!(ca_jurs.len(), 11, "CA: 1 federal + 10 provinces");

        // CA should have HST codes per province
        let ca_hst_codes: Vec<&TaxCode> = codes
            .iter()
            .filter(|c| c.code.starts_with("HST-"))
            .collect();
        assert_eq!(
            ca_hst_codes.len(),
            10,
            "CA should have 10 provincial HST codes"
        );

        // Ontario HST should be 13%
        let on_code = ca_hst_codes
            .iter()
            .find(|c| c.code == "HST-ON")
            .expect("Ontario HST code");
        assert_eq!(on_code.rate, dec!(0.13));
    }

    #[test]
    fn test_nexus_states_filter() {
        let mut config = TaxConfig::default();
        config.jurisdictions.countries = vec!["US".into()];
        config.jurisdictions.include_subnational = true;
        config.sales_tax.nexus_states = vec!["CA".into(), "NY".into()];

        let mut gen = TaxCodeGenerator::with_config(42, config);
        let (jurisdictions, codes) = gen.generate();

        let state_jurs: Vec<&TaxJurisdiction> = jurisdictions
            .iter()
            .filter(|j| j.country_code == "US" && j.jurisdiction_type == JurisdictionType::State)
            .collect();
        assert_eq!(state_jurs.len(), 2, "Should only generate nexus states");

        let state_codes: Vec<String> = state_jurs
            .iter()
            .filter_map(|j| j.region_code.clone())
            .collect();
        assert!(state_codes.contains(&"CA".to_string()));
        assert!(state_codes.contains(&"NY".to_string()));

        // Sales tax codes should only be for CA and NY
        let sales_codes: Vec<&TaxCode> = codes
            .iter()
            .filter(|c| c.tax_type == TaxType::SalesTax)
            .collect();
        assert_eq!(sales_codes.len(), 2);
    }

    #[test]
    fn test_vat_registered_flag() {
        let mut config = TaxConfig::default();
        config.jurisdictions.countries = vec!["DE".into(), "SG".into(), "US".into()];

        let mut gen = TaxCodeGenerator::with_config(42, config);
        let (jurisdictions, _codes) = gen.generate();

        let de_federal = jurisdictions
            .iter()
            .find(|j| j.id == "JUR-DE")
            .expect("DE federal");
        assert!(de_federal.vat_registered, "DE should be VAT-registered");

        let sg_federal = jurisdictions
            .iter()
            .find(|j| j.id == "JUR-SG")
            .expect("SG federal");
        assert!(
            sg_federal.vat_registered,
            "SG should be VAT-registered (GST)"
        );

        let us_federal = jurisdictions
            .iter()
            .find(|j| j.id == "JUR-US")
            .expect("US federal");
        assert!(
            !us_federal.vat_registered,
            "US should NOT be VAT-registered (sales tax)"
        );
    }

    #[test]
    fn test_exempt_codes_generated() {
        let mut config = TaxConfig::default();
        config.jurisdictions.countries = vec!["DE".into()];

        let mut gen = TaxCodeGenerator::with_config(42, config);
        let (_jurisdictions, codes) = gen.generate();

        let exempt = codes
            .iter()
            .find(|c| c.code == "VAT-EX-DE")
            .expect("DE exempt code");
        assert!(exempt.is_exempt);
        assert_eq!(exempt.rate, dec!(0));
        assert_eq!(exempt.tax_amount(dec!(10000)), dec!(0));
    }

    #[test]
    fn test_effective_dates() {
        let mut gen = TaxCodeGenerator::new(42);
        let (_jurisdictions, codes) = gen.generate();

        let expected_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        for code in &codes {
            assert_eq!(
                code.effective_date, expected_date,
                "All codes should have effective date 2020-01-01, got {} for {}",
                code.effective_date, code.code
            );
            assert!(
                code.expiry_date.is_none(),
                "Codes should not have an expiry date"
            );
        }
    }

    #[test]
    fn test_reduced_rate_override() {
        let mut config = TaxConfig::default();
        config.jurisdictions.countries = vec!["JP".into()];
        config.vat_gst.reduced_rates.insert("JP".into(), 0.03);

        let mut gen = TaxCodeGenerator::with_config(42, config);
        let (_jurisdictions, codes) = gen.generate();

        let jp_red = codes
            .iter()
            .find(|c| c.code == "GST-RED-JP")
            .expect("JP reduced GST code");
        assert_eq!(
            jp_red.rate,
            dec!(0.03),
            "Reduced rate override should apply"
        );
    }

    #[test]
    fn test_germany_subnational() {
        let mut config = TaxConfig::default();
        config.jurisdictions.countries = vec!["DE".into()];
        config.jurisdictions.include_subnational = true;

        let mut gen = TaxCodeGenerator::with_config(42, config);
        let (jurisdictions, _codes) = gen.generate();

        let de_states: Vec<&TaxJurisdiction> = jurisdictions
            .iter()
            .filter(|j| j.country_code == "DE" && j.jurisdiction_type == JurisdictionType::State)
            .collect();
        assert_eq!(de_states.len(), 16, "Germany should have 16 Bundeslaender");

        // All states should reference the federal parent
        for state in &de_states {
            assert_eq!(
                state.parent_jurisdiction_id,
                Some("JUR-DE".to_string()),
                "State {} should have federal parent",
                state.name
            );
            assert!(state.vat_registered);
        }
    }

    #[test]
    fn test_format_rate_pct() {
        assert_eq!(format_rate_pct(dec!(0.19)), "19%");
        assert_eq!(format_rate_pct(dec!(0.055)), "5.5%");
        assert_eq!(format_rate_pct(dec!(0.0725)), "7.25%");
        assert_eq!(format_rate_pct(dec!(0)), "0%");
    }
}
