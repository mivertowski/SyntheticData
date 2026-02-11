//! Timezone handling for multi-region synthetic data generation.
//!
//! This module provides timezone-aware datetime handling for:
//! - Entity-specific timezone assignments (by company code pattern)
//! - UTC to local time conversions
//! - Consolidation timezone for group reporting

use chrono::{DateTime, NaiveDateTime, Offset, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Timezone configuration for multi-region entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezoneConfig {
    /// Default timezone for entities without a specific mapping.
    pub default_timezone: String,
    /// Entity-to-timezone mappings (supports patterns like "EU_*" -> "Europe/London").
    pub entity_timezones: HashMap<String, String>,
    /// Consolidation timezone for group reporting.
    pub consolidation_timezone: String,
}

impl Default for TimezoneConfig {
    fn default() -> Self {
        Self {
            default_timezone: "America/New_York".to_string(),
            entity_timezones: HashMap::new(),
            consolidation_timezone: "UTC".to_string(),
        }
    }
}

impl TimezoneConfig {
    /// Creates a new timezone config with the specified default timezone.
    pub fn new(default_tz: &str) -> Self {
        Self {
            default_timezone: default_tz.to_string(),
            entity_timezones: HashMap::new(),
            consolidation_timezone: "UTC".to_string(),
        }
    }

    /// Sets the consolidation timezone.
    pub fn with_consolidation(mut self, tz: &str) -> Self {
        self.consolidation_timezone = tz.to_string();
        self
    }

    /// Adds an entity-to-timezone mapping.
    ///
    /// Supports pattern matching:
    /// - Exact match: "1000" -> "America/New_York"
    /// - Prefix match: "EU_*" -> "Europe/London"
    /// - Suffix match: "*_APAC" -> "Asia/Singapore"
    pub fn add_mapping(mut self, entity_pattern: &str, timezone: &str) -> Self {
        self.entity_timezones
            .insert(entity_pattern.to_string(), timezone.to_string());
        self
    }
}

/// Handler for timezone operations.
#[derive(Debug, Clone)]
pub struct TimezoneHandler {
    config: TimezoneConfig,
    /// Parsed default timezone.
    default_tz: Tz,
    /// Parsed consolidation timezone.
    consolidation_tz: Tz,
    /// Cached parsed entity timezones.
    entity_tz_cache: HashMap<String, Tz>,
}

impl TimezoneHandler {
    /// Creates a new timezone handler from configuration.
    pub fn new(config: TimezoneConfig) -> Result<Self, TimezoneError> {
        let default_tz: Tz = config
            .default_timezone
            .parse()
            .map_err(|_| TimezoneError::InvalidTimezone(config.default_timezone.clone()))?;

        let consolidation_tz: Tz = config
            .consolidation_timezone
            .parse()
            .map_err(|_| TimezoneError::InvalidTimezone(config.consolidation_timezone.clone()))?;

        // Pre-parse all entity timezone mappings
        let mut entity_tz_cache = HashMap::new();
        for (pattern, tz_name) in &config.entity_timezones {
            let tz: Tz = tz_name
                .parse()
                .map_err(|_| TimezoneError::InvalidTimezone(tz_name.clone()))?;
            entity_tz_cache.insert(pattern.clone(), tz);
        }

        Ok(Self {
            config,
            default_tz,
            consolidation_tz,
            entity_tz_cache,
        })
    }

    /// Creates a handler with default US Eastern timezone.
    pub fn us_eastern() -> Self {
        Self::new(TimezoneConfig::default()).expect("Default timezone config should be valid")
    }

    /// Gets the timezone for a specific entity code.
    ///
    /// Matches entity code against patterns in this order:
    /// 1. Exact match
    /// 2. Prefix patterns (e.g., "EU_*")
    /// 3. Suffix patterns (e.g., "*_APAC")
    /// 4. Default timezone
    pub fn get_entity_timezone(&self, entity_code: &str) -> Tz {
        // Check exact match first
        if let Some(tz) = self.entity_tz_cache.get(entity_code) {
            return *tz;
        }

        // Check patterns
        for (pattern, tz) in &self.entity_tz_cache {
            if let Some(prefix) = pattern.strip_suffix('*') {
                // Prefix pattern (e.g., "EU_*")
                if entity_code.starts_with(prefix) {
                    return *tz;
                }
            } else if let Some(suffix) = pattern.strip_prefix('*') {
                // Suffix pattern (e.g., "*_APAC")
                if entity_code.ends_with(suffix) {
                    return *tz;
                }
            }
        }

        self.default_tz
    }

    /// Converts a local datetime to UTC for a specific entity.
    pub fn to_utc(&self, local: NaiveDateTime, entity_code: &str) -> DateTime<Utc> {
        let tz = self.get_entity_timezone(entity_code);
        tz.from_local_datetime(&local)
            .single()
            .unwrap_or_else(|| {
                tz.from_local_datetime(&local)
                    .earliest()
                    .expect("valid time components")
            })
            .with_timezone(&Utc)
    }

    /// Converts a UTC datetime to local time for a specific entity.
    pub fn to_local(&self, utc: DateTime<Utc>, entity_code: &str) -> NaiveDateTime {
        let tz = self.get_entity_timezone(entity_code);
        utc.with_timezone(&tz).naive_local()
    }

    /// Converts a local datetime to the consolidation timezone.
    pub fn to_consolidation(&self, local: NaiveDateTime, entity_code: &str) -> DateTime<Tz> {
        let utc = self.to_utc(local, entity_code);
        utc.with_timezone(&self.consolidation_tz)
    }

    /// Returns the consolidation timezone.
    pub fn consolidation_timezone(&self) -> Tz {
        self.consolidation_tz
    }

    /// Returns the default timezone.
    pub fn default_timezone(&self) -> Tz {
        self.default_tz
    }

    /// Returns the timezone name for an entity.
    pub fn get_timezone_name(&self, entity_code: &str) -> String {
        self.get_entity_timezone(entity_code).name().to_string()
    }

    /// Calculates the UTC offset in hours for an entity at a given time.
    pub fn get_utc_offset_hours(&self, entity_code: &str, at: NaiveDateTime) -> f64 {
        let tz = self.get_entity_timezone(entity_code);
        let offset = tz.offset_from_local_datetime(&at).single();
        match offset {
            Some(o) => o.fix().local_minus_utc() as f64 / 3600.0,
            None => 0.0,
        }
    }

    /// Returns a reference to the underlying configuration.
    pub fn config(&self) -> &TimezoneConfig {
        &self.config
    }
}

/// Errors that can occur during timezone operations.
#[derive(Debug, Clone)]
pub enum TimezoneError {
    /// Invalid timezone name.
    InvalidTimezone(String),
    /// Ambiguous local time (e.g., during DST transition).
    AmbiguousTime(NaiveDateTime),
}

impl std::fmt::Display for TimezoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimezoneError::InvalidTimezone(tz) => {
                write!(f, "Invalid timezone: '{}'. Use IANA timezone names.", tz)
            }
            TimezoneError::AmbiguousTime(dt) => {
                write!(f, "Ambiguous local time: {}", dt)
            }
        }
    }
}

impl std::error::Error for TimezoneError {}

/// Common timezone presets for different regions.
pub struct TimezonePresets;

impl TimezonePresets {
    /// US-centric configuration with NY as default.
    pub fn us_centric() -> TimezoneConfig {
        TimezoneConfig::new("America/New_York")
            .with_consolidation("America/New_York")
            .add_mapping("*_WEST", "America/Los_Angeles")
            .add_mapping("*_CENTRAL", "America/Chicago")
            .add_mapping("*_MOUNTAIN", "America/Denver")
    }

    /// Europe-centric configuration with London as default.
    pub fn eu_centric() -> TimezoneConfig {
        TimezoneConfig::new("Europe/London")
            .with_consolidation("Europe/London")
            .add_mapping("DE_*", "Europe/Berlin")
            .add_mapping("FR_*", "Europe/Paris")
            .add_mapping("CH_*", "Europe/Zurich")
    }

    /// Asia-Pacific configuration with Singapore as default.
    pub fn apac_centric() -> TimezoneConfig {
        TimezoneConfig::new("Asia/Singapore")
            .with_consolidation("Asia/Singapore")
            .add_mapping("JP_*", "Asia/Tokyo")
            .add_mapping("CN_*", "Asia/Shanghai")
            .add_mapping("IN_*", "Asia/Kolkata")
            .add_mapping("AU_*", "Australia/Sydney")
    }

    /// Global configuration with UTC as consolidation.
    pub fn global_utc() -> TimezoneConfig {
        TimezoneConfig::new("America/New_York")
            .with_consolidation("UTC")
            .add_mapping("US_*", "America/New_York")
            .add_mapping("EU_*", "Europe/London")
            .add_mapping("APAC_*", "Asia/Singapore")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_default_timezone() {
        let handler = TimezoneHandler::us_eastern();
        let tz = handler.get_entity_timezone("UNKNOWN_COMPANY");
        assert_eq!(tz.name(), "America/New_York");
    }

    #[test]
    fn test_exact_match() {
        let config = TimezoneConfig::new("America/New_York").add_mapping("1000", "Europe/London");
        let handler = TimezoneHandler::new(config).unwrap();

        assert_eq!(handler.get_entity_timezone("1000").name(), "Europe/London");
        assert_eq!(
            handler.get_entity_timezone("2000").name(),
            "America/New_York"
        );
    }

    #[test]
    fn test_prefix_pattern() {
        let config = TimezoneConfig::new("America/New_York").add_mapping("EU_*", "Europe/Berlin");
        let handler = TimezoneHandler::new(config).unwrap();

        assert_eq!(
            handler.get_entity_timezone("EU_1000").name(),
            "Europe/Berlin"
        );
        assert_eq!(
            handler.get_entity_timezone("EU_SUBSIDIARY").name(),
            "Europe/Berlin"
        );
        assert_eq!(
            handler.get_entity_timezone("US_1000").name(),
            "America/New_York"
        );
    }

    #[test]
    fn test_suffix_pattern() {
        let config = TimezoneConfig::new("America/New_York").add_mapping("*_APAC", "Asia/Tokyo");
        let handler = TimezoneHandler::new(config).unwrap();

        assert_eq!(
            handler.get_entity_timezone("1000_APAC").name(),
            "Asia/Tokyo"
        );
        assert_eq!(
            handler.get_entity_timezone("CORP_APAC").name(),
            "Asia/Tokyo"
        );
        assert_eq!(
            handler.get_entity_timezone("1000_US").name(),
            "America/New_York"
        );
    }

    #[test]
    fn test_to_utc() {
        let handler = TimezoneHandler::new(TimezoneConfig::new("America/New_York")).unwrap();

        // 10 AM New York time
        let local =
            NaiveDateTime::parse_from_str("2024-06-15 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let utc = handler.to_utc(local, "US_1000");

        // In June, NY is UTC-4 (EDT)
        assert_eq!(utc.hour(), 14);
    }

    #[test]
    fn test_to_local() {
        let config = TimezoneConfig::new("America/New_York").add_mapping("EU_*", "Europe/London");
        let handler = TimezoneHandler::new(config).unwrap();

        // 12:00 UTC
        let utc = DateTime::parse_from_rfc3339("2024-06-15T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        // London in June is UTC+1 (BST)
        let london_local = handler.to_local(utc, "EU_1000");
        assert_eq!(london_local.hour(), 13);

        // New York in June is UTC-4 (EDT)
        let ny_local = handler.to_local(utc, "US_1000");
        assert_eq!(ny_local.hour(), 8);
    }

    #[test]
    fn test_presets() {
        // Test that presets can be created without errors
        let _ = TimezoneHandler::new(TimezonePresets::us_centric()).unwrap();
        let _ = TimezoneHandler::new(TimezonePresets::eu_centric()).unwrap();
        let _ = TimezoneHandler::new(TimezonePresets::apac_centric()).unwrap();
        let _ = TimezoneHandler::new(TimezonePresets::global_utc()).unwrap();
    }

    #[test]
    fn test_invalid_timezone() {
        let config = TimezoneConfig::new("Invalid/Timezone");
        let result = TimezoneHandler::new(config);
        assert!(result.is_err());
    }
}
