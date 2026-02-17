//! Pluggable country-pack architecture.
//!
//! Loads country-specific configuration (holidays, names, tax rates, phone
//! formats, address data, payroll rules, etc.) from JSON files.  Built-in
//! packs are compiled in via `include_str!`; external packs can be loaded
//! from a directory at runtime.

pub mod easter;
pub mod error;
pub mod lunar;
pub mod merge;
pub mod schema;

pub use error::CountryPackError;
pub use schema::CountryPack;

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use merge::{apply_override, deep_merge};

// ---------------------------------------------------------------------------
// Embedded packs
// ---------------------------------------------------------------------------

const DEFAULT_PACK_JSON: &str = include_str!("../../country-packs/_default.json");
const US_PACK_JSON: &str = include_str!("../../country-packs/US.json");
const DE_PACK_JSON: &str = include_str!("../../country-packs/DE.json");
const GB_PACK_JSON: &str = include_str!("../../country-packs/GB.json");

// ---------------------------------------------------------------------------
// CountryCode
// ---------------------------------------------------------------------------

/// Validated ISO 3166-1 alpha-2 country code (or `_DEFAULT`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CountryCode(String);

impl CountryCode {
    /// Create a `CountryCode`, validating that it is exactly 2 uppercase ASCII
    /// letters or the special `_DEFAULT` sentinel.
    pub fn new(code: &str) -> Result<Self, CountryPackError> {
        let code = code.trim().to_uppercase();
        if code == "_DEFAULT" {
            return Ok(Self(code));
        }
        if code.len() == 2 && code.chars().all(|c| c.is_ascii_uppercase()) {
            Ok(Self(code))
        } else {
            Err(CountryPackError::InvalidCountryCode(code))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CountryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ---------------------------------------------------------------------------
// CountryPackRegistry
// ---------------------------------------------------------------------------

/// Central registry that owns all loaded country packs and provides merged
/// access keyed by country code.
pub struct CountryPackRegistry {
    default_pack: CountryPack,
    packs: HashMap<CountryCode, CountryPack>,
}

impl CountryPackRegistry {
    /// Build a registry from embedded packs, optionally loading additional
    /// packs from `external_dir` and applying per-country `overrides`.
    pub fn new(
        external_dir: Option<&Path>,
        overrides: &HashMap<String, serde_json::Value>,
    ) -> Result<Self, CountryPackError> {
        let mut registry = Self::builtin_only()?;

        // Load external packs if a directory is provided.
        if let Some(dir) = external_dir {
            registry.load_external_dir(dir)?;
        }

        // Apply per-country overrides.
        for (code_str, value) in overrides {
            let code = CountryCode::new(code_str)?;
            if let Some(pack) = registry.packs.get_mut(&code) {
                apply_override(pack, value)?;
            } else {
                // Create a new pack by merging default + override.
                let mut pack = registry.default_pack.clone();
                pack.country_code = code_str.to_uppercase();
                apply_override(&mut pack, value)?;
                registry.packs.insert(code, pack);
            }
        }

        Ok(registry)
    }

    /// Build a registry from the embedded (built-in) packs only.
    pub fn builtin_only() -> Result<Self, CountryPackError> {
        let default_pack: CountryPack = serde_json::from_str(DEFAULT_PACK_JSON)
            .map_err(|e| CountryPackError::parse(format!("_default.json: {e}")))?;

        let mut packs = HashMap::new();

        for (json, label) in [
            (US_PACK_JSON, "US.json"),
            (DE_PACK_JSON, "DE.json"),
            (GB_PACK_JSON, "GB.json"),
        ] {
            let pack = Self::parse_and_merge(&default_pack, json, label)?;
            let code = CountryCode::new(&pack.country_code)?;
            packs.insert(code, pack);
        }

        Ok(Self {
            default_pack,
            packs,
        })
    }

    /// Look up a pack by `CountryCode`. Falls back to the default pack for
    /// unknown codes.
    pub fn get(&self, code: &CountryCode) -> &CountryPack {
        self.packs.get(code).unwrap_or(&self.default_pack)
    }

    /// Convenience: look up by a raw country-code string (case-insensitive).
    /// Returns the default pack if the code is invalid or unknown.
    pub fn get_by_str(&self, code: &str) -> &CountryPack {
        match CountryCode::new(code) {
            Ok(cc) => self.get(&cc),
            Err(_) => &self.default_pack,
        }
    }

    /// List all country codes that have explicit packs (excludes `_DEFAULT`).
    pub fn available_countries(&self) -> Vec<&CountryCode> {
        self.packs.keys().collect()
    }

    /// Access the default (fallback) pack.
    pub fn default_pack(&self) -> &CountryPack {
        &self.default_pack
    }

    // -- internal helpers ---------------------------------------------------

    /// Parse a country JSON string and deep-merge it on top of the default.
    fn parse_and_merge(
        default: &CountryPack,
        json: &str,
        label: &str,
    ) -> Result<CountryPack, CountryPackError> {
        let country_value: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| CountryPackError::parse(format!("{label}: {e}")))?;

        let mut base_value = serde_json::to_value(default)
            .map_err(|e| CountryPackError::parse(format!("serialize default: {e}")))?;

        deep_merge(&mut base_value, &country_value);

        serde_json::from_value(base_value)
            .map_err(|e| CountryPackError::parse(format!("{label} merge: {e}")))
    }

    /// Scan an external directory for `*.json` files and load each as a
    /// country pack, merging on top of the default.
    fn load_external_dir(&mut self, dir: &Path) -> Result<(), CountryPackError> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| CountryPackError::directory(format!("{}: {e}", dir.display())))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| CountryPackError::directory(format!("{}: {e}", dir.display())))?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            // Skip _default.json in external dirs (only embedded default is used).
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem == "_default" {
                    continue;
                }
            }

            let json = std::fs::read_to_string(&path).map_err(|e| {
                CountryPackError::directory(format!("{}: {e}", path.display()))
            })?;

            let label = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let pack = Self::parse_and_merge(&self.default_pack, &json, label)?;
            let code = CountryCode::new(&pack.country_code)?;
            self.packs.insert(code, pack);
        }

        Ok(())
    }
}

impl fmt::Debug for CountryPackRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CountryPackRegistry")
            .field("default", &self.default_pack.country_code)
            .field(
                "countries",
                &self.packs.keys().map(|c| c.as_str()).collect::<Vec<_>>(),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_country_code_valid() {
        assert!(CountryCode::new("US").is_ok());
        assert!(CountryCode::new("de").is_ok()); // lowercased → DE
        assert!(CountryCode::new("GB").is_ok());
        assert!(CountryCode::new("_DEFAULT").is_ok());
    }

    #[test]
    fn test_country_code_invalid() {
        assert!(CountryCode::new("").is_err());
        assert!(CountryCode::new("USA").is_err());
        assert!(CountryCode::new("1A").is_err());
        assert!(CountryCode::new("A").is_err());
    }

    #[test]
    fn test_builtin_only() {
        let reg = CountryPackRegistry::builtin_only().expect("should load");
        assert!(reg.available_countries().len() >= 3);

        let us = reg.get_by_str("US");
        assert_eq!(us.country_code, "US");
        assert!(!us.holidays.fixed.is_empty());

        let de = reg.get_by_str("DE");
        assert_eq!(de.country_code, "DE");

        let gb = reg.get_by_str("GB");
        assert_eq!(gb.country_code, "GB");
    }

    #[test]
    fn test_fallback_to_default() {
        let reg = CountryPackRegistry::builtin_only().expect("should load");
        let unknown = reg.get_by_str("ZZ");
        assert_eq!(unknown.country_code, "_DEFAULT");
    }

    #[test]
    fn test_default_pack_parses() {
        let pack: CountryPack =
            serde_json::from_str(DEFAULT_PACK_JSON).expect("default pack should parse");
        assert_eq!(pack.country_code, "_DEFAULT");
        assert_eq!(pack.schema_version, "1.0");
    }

    #[test]
    fn test_registry_with_overrides() {
        let mut overrides = HashMap::new();
        overrides.insert(
            "US".to_string(),
            serde_json::json!({"country_name": "USA Override"}),
        );
        let reg = CountryPackRegistry::new(None, &overrides).expect("should load");
        let us = reg.get_by_str("US");
        assert_eq!(us.country_name, "USA Override");
    }
}
