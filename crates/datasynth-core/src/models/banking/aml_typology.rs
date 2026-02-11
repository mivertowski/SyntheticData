//! AML typology definitions for money laundering patterns.

use serde::{Deserialize, Serialize};

/// AML typology classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmlTypology {
    // Structuring patterns
    /// Structuring deposits below reporting threshold
    Structuring,
    /// Smurfing (multiple people making small deposits)
    Smurfing,
    /// Cuckoo smurfing (using legitimate account holders)
    CuckooSmurfing,

    // Funnel patterns
    /// Funnel account (many in, few out)
    FunnelAccount,
    /// Concentration account abuse
    ConcentrationAccount,
    /// Pouch activity (cash collected and deposited in bulk)
    PouchActivity,

    // Layering patterns
    /// Layering through multiple transfers
    Layering,
    /// Rapid movement of funds
    RapidMovement,
    /// Shell company transactions
    ShellCompany,

    // Round-tripping
    /// Round-tripping through foreign accounts
    RoundTripping,
    /// Trade-based money laundering
    TradeBasedML,
    /// Invoice manipulation
    InvoiceManipulation,

    // Mule patterns
    /// Money mule recruitment and use
    MoneyMule,
    /// Romance scam / social engineering
    RomanceScam,
    /// Advance fee fraud
    AdvanceFeeFraud,

    // Integration patterns
    /// Real estate integration
    RealEstateIntegration,
    /// Luxury goods purchase
    LuxuryGoods,
    /// Casino integration
    CasinoIntegration,
    /// Cryptocurrency integration
    CryptoIntegration,

    // Fraud patterns
    /// Account takeover
    AccountTakeover,
    /// Synthetic identity
    SyntheticIdentity,
    /// First-party fraud
    FirstPartyFraud,
    /// Authorized push payment fraud
    AuthorizedPushPayment,
    /// Business email compromise
    BusinessEmailCompromise,
    /// Fake vendor
    FakeVendor,

    // Other typologies
    /// Terrorist financing
    TerroristFinancing,
    /// Sanctions evasion
    SanctionsEvasion,
    /// Tax evasion
    TaxEvasion,
    /// Human trafficking
    HumanTrafficking,
    /// Drug trafficking
    DrugTrafficking,
    /// Corruption / PEP
    Corruption,

    /// Custom / other typology
    Custom(u16),
}

impl AmlTypology {
    /// Returns the category name.
    pub fn category(&self) -> &'static str {
        match self {
            Self::Structuring | Self::Smurfing | Self::CuckooSmurfing => "Structuring",
            Self::FunnelAccount | Self::ConcentrationAccount | Self::PouchActivity => "Funnel",
            Self::Layering | Self::RapidMovement | Self::ShellCompany => "Layering",
            Self::RoundTripping | Self::TradeBasedML | Self::InvoiceManipulation => {
                "Round-Tripping"
            }
            Self::MoneyMule | Self::RomanceScam | Self::AdvanceFeeFraud => "Mule/Scam",
            Self::RealEstateIntegration
            | Self::LuxuryGoods
            | Self::CasinoIntegration
            | Self::CryptoIntegration => "Integration",
            Self::AccountTakeover
            | Self::SyntheticIdentity
            | Self::FirstPartyFraud
            | Self::AuthorizedPushPayment
            | Self::BusinessEmailCompromise
            | Self::FakeVendor => "Fraud",
            Self::TerroristFinancing
            | Self::SanctionsEvasion
            | Self::TaxEvasion
            | Self::HumanTrafficking
            | Self::DrugTrafficking
            | Self::Corruption => "Predicate Crime",
            Self::Custom(_) => "Custom",
        }
    }

    /// Risk severity (1-10, 10 being most severe).
    pub fn severity(&self) -> u8 {
        match self {
            Self::TerroristFinancing | Self::SanctionsEvasion | Self::HumanTrafficking => 10,
            Self::DrugTrafficking | Self::Corruption => 9,
            Self::AccountTakeover | Self::BusinessEmailCompromise => 8,
            Self::MoneyMule | Self::SyntheticIdentity | Self::ShellCompany => 7,
            Self::Structuring | Self::Layering | Self::RoundTripping => 6,
            Self::FunnelAccount | Self::RapidMovement => 5,
            Self::TaxEvasion | Self::FirstPartyFraud => 5,
            Self::Smurfing | Self::CuckooSmurfing => 5,
            Self::TradeBasedML | Self::InvoiceManipulation => 6,
            Self::CryptoIntegration | Self::CasinoIntegration => 5,
            Self::RealEstateIntegration | Self::LuxuryGoods => 4,
            Self::RomanceScam | Self::AdvanceFeeFraud => 6,
            Self::AuthorizedPushPayment | Self::FakeVendor => 7,
            Self::ConcentrationAccount | Self::PouchActivity => 5,
            Self::Custom(_) => 5,
        }
    }

    /// Whether this is primarily a fraud pattern (vs AML).
    pub fn is_fraud(&self) -> bool {
        matches!(
            self,
            Self::AccountTakeover
                | Self::SyntheticIdentity
                | Self::FirstPartyFraud
                | Self::AuthorizedPushPayment
                | Self::BusinessEmailCompromise
                | Self::FakeVendor
                | Self::RomanceScam
                | Self::AdvanceFeeFraud
        )
    }

    /// Typical duration in days for the pattern to complete.
    pub fn typical_duration_days(&self) -> (u32, u32) {
        match self {
            Self::Structuring => (1, 30),
            Self::Smurfing | Self::CuckooSmurfing => (1, 7),
            Self::FunnelAccount => (7, 90),
            Self::ConcentrationAccount => (30, 180),
            Self::PouchActivity => (1, 3),
            Self::Layering => (3, 14),
            Self::RapidMovement => (1, 3),
            Self::ShellCompany => (30, 365),
            Self::RoundTripping => (7, 60),
            Self::TradeBasedML | Self::InvoiceManipulation => (30, 180),
            Self::MoneyMule => (1, 30),
            Self::RomanceScam => (30, 180),
            Self::AdvanceFeeFraud => (7, 60),
            Self::RealEstateIntegration => (60, 180),
            Self::LuxuryGoods => (1, 7),
            Self::CasinoIntegration => (1, 30),
            Self::CryptoIntegration => (1, 14),
            Self::AccountTakeover => (1, 7),
            Self::SyntheticIdentity => (30, 365),
            Self::FirstPartyFraud => (30, 180),
            Self::AuthorizedPushPayment => (1, 3),
            Self::BusinessEmailCompromise => (1, 14),
            Self::FakeVendor => (30, 180),
            _ => (7, 90),
        }
    }

    /// Number of entities typically involved.
    pub fn typical_entity_count(&self) -> (u32, u32) {
        match self {
            Self::Structuring => (1, 1),
            Self::Smurfing => (3, 20),
            Self::CuckooSmurfing => (5, 50),
            Self::FunnelAccount => (10, 100),
            Self::MoneyMule => (3, 10),
            Self::Layering => (3, 20),
            Self::ShellCompany => (2, 10),
            Self::RoundTripping => (2, 5),
            _ => (1, 5),
        }
    }
}

/// Money laundering stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunderingStage {
    /// Placement - introducing illicit funds into financial system
    Placement,
    /// Layering - disguising the trail
    Layering,
    /// Integration - making funds appear legitimate
    Integration,
    /// Not applicable (e.g., for fraud patterns)
    NotApplicable,
}

impl LaunderingStage {
    /// Description of the stage.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Placement => "Introducing illicit funds into the financial system",
            Self::Layering => "Disguising the source through complex transactions",
            Self::Integration => "Making funds appear legitimate through business",
            Self::NotApplicable => "Not a laundering pattern",
        }
    }

    /// Typical detection difficulty (1-10).
    pub fn detection_difficulty(&self) -> u8 {
        match self {
            Self::Placement => 4,
            Self::Layering => 7,
            Self::Integration => 9,
            Self::NotApplicable => 5,
        }
    }
}

/// Sophistication level of the AML pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Sophistication {
    /// Basic - easily detectable patterns
    Basic,
    /// Standard - some evasion tactics
    #[default]
    Standard,
    /// Professional - deliberate evasion tactics
    Professional,
    /// Advanced - complex multi-stage schemes
    Advanced,
    /// State-level - nation-state sophistication
    StateLevel,
}

impl Sophistication {
    /// Detectability modifier (0.0-1.0, lower = harder to detect).
    pub fn detectability_modifier(&self) -> f64 {
        match self {
            Self::Basic => 1.0,
            Self::Standard => 0.7,
            Self::Professional => 0.4,
            Self::Advanced => 0.2,
            Self::StateLevel => 0.1,
        }
    }

    /// Spoofing intensity for mimicking normal behavior.
    pub fn spoofing_intensity(&self) -> f64 {
        match self {
            Self::Basic => 0.0,
            Self::Standard => 0.2,
            Self::Professional => 0.5,
            Self::Advanced => 0.8,
            Self::StateLevel => 0.95,
        }
    }
}

/// Evasion tactics used to avoid detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvasionTactic {
    /// Using amounts just below thresholds
    ThresholdAvoidance,
    /// Adding time delays between transactions
    TimeJitter,
    /// Splitting transactions across multiple accounts
    AccountSplitting,
    /// Using multiple channels
    ChannelDiversification,
    /// Mimicking normal spending patterns
    PatternMimicry,
    /// Using cover transactions
    CoverTraffic,
    /// Mixing legitimate and illicit funds
    Commingling,
    /// Using nested correspondent relationships
    NestedCorrespondent,
    /// Exploiting regulatory gaps
    RegulatoryArbitrage,
    /// Using privacy-enhancing technology
    PrivacyTechnology,
}

impl EvasionTactic {
    /// Detection difficulty modifier (1.0 = standard).
    pub fn difficulty_modifier(&self) -> f64 {
        match self {
            Self::ThresholdAvoidance => 1.2,
            Self::TimeJitter => 1.3,
            Self::AccountSplitting => 1.4,
            Self::ChannelDiversification => 1.3,
            Self::PatternMimicry => 1.8,
            Self::CoverTraffic => 1.6,
            Self::Commingling => 1.5,
            Self::NestedCorrespondent => 1.7,
            Self::RegulatoryArbitrage => 1.4,
            Self::PrivacyTechnology => 2.0,
        }
    }
}

/// Turnover band for expected activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TurnoverBand {
    /// Very low (<$1K/month)
    VeryLow,
    /// Low ($1K-$5K/month)
    #[default]
    Low,
    /// Medium ($5K-$25K/month)
    Medium,
    /// High ($25K-$100K/month)
    High,
    /// Very high ($100K-$500K/month)
    VeryHigh,
    /// Ultra high (>$500K/month)
    UltraHigh,
}

impl TurnoverBand {
    /// Expected monthly turnover range.
    pub fn range(&self) -> (u64, u64) {
        match self {
            Self::VeryLow => (0, 1_000),
            Self::Low => (1_000, 5_000),
            Self::Medium => (5_000, 25_000),
            Self::High => (25_000, 100_000),
            Self::VeryHigh => (100_000, 500_000),
            Self::UltraHigh => (500_000, 10_000_000),
        }
    }
}

/// Frequency band for expected transaction count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FrequencyBand {
    /// Very low (< 10 transactions/month)
    VeryLow,
    /// Low (10-30 transactions/month)
    #[default]
    Low,
    /// Medium (30-100 transactions/month)
    Medium,
    /// High (100-300 transactions/month)
    High,
    /// Very high (> 300 transactions/month)
    VeryHigh,
}

impl FrequencyBand {
    /// Expected monthly transaction count range.
    pub fn range(&self) -> (u32, u32) {
        match self {
            Self::VeryLow => (0, 10),
            Self::Low => (10, 30),
            Self::Medium => (30, 100),
            Self::High => (100, 300),
            Self::VeryHigh => (300, 10_000),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_aml_typology_severity() {
        assert!(AmlTypology::TerroristFinancing.severity() >= 10);
        assert!(AmlTypology::Structuring.severity() < AmlTypology::TerroristFinancing.severity());
    }

    #[test]
    fn test_aml_typology_category() {
        assert_eq!(AmlTypology::Structuring.category(), "Structuring");
        assert_eq!(AmlTypology::Smurfing.category(), "Structuring");
        assert_eq!(AmlTypology::FunnelAccount.category(), "Funnel");
        assert_eq!(AmlTypology::AccountTakeover.category(), "Fraud");
    }

    #[test]
    fn test_laundering_stage() {
        assert!(
            LaunderingStage::Integration.detection_difficulty()
                > LaunderingStage::Placement.detection_difficulty()
        );
    }

    #[test]
    fn test_sophistication_detectability() {
        assert!(
            Sophistication::Basic.detectability_modifier()
                > Sophistication::Professional.detectability_modifier()
        );
    }

    #[test]
    fn test_turnover_band_range() {
        let (min, max) = TurnoverBand::Medium.range();
        assert!(min < max);
        assert!(min >= 5_000);
        assert!(max <= 25_000);
    }
}
