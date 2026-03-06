//! Graph property mapping trait and types for node export.
//!
//! Provides a `ToNodeProperties` trait that each model struct implements
//! to map typed Rust fields to camelCase graph properties expected by
//! downstream consumers (e.g. AssureTwin).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Property value for graph node export.
///
/// Mirrors `datasynth-graph` `NodeProperty` but lives in `datasynth-core`
/// to avoid circular dependencies.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphPropertyValue {
    String(String),
    Int(i64),
    Float(f64),
    Decimal(Decimal),
    Bool(bool),
    Date(NaiveDate),
    StringList(Vec<String>),
}

impl GraphPropertyValue {
    /// Convert any variant to a string representation.
    pub fn to_string_value(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Int(i) => i.to_string(),
            Self::Float(f) => format!("{f:.6}"),
            Self::Decimal(d) => d.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Date(d) => d.to_string(),
            Self::StringList(v) => v.join(";"),
        }
    }

    /// Try to extract a string reference.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to extract a bool value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to extract a Decimal value.
    pub fn as_decimal(&self) -> Option<Decimal> {
        match self {
            Self::Decimal(d) => Some(*d),
            _ => None,
        }
    }

    /// Try to extract an i64 value.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to extract an f64 value.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Try to extract a date value.
    pub fn as_date(&self) -> Option<NaiveDate> {
        match self {
            Self::Date(d) => Some(*d),
            _ => None,
        }
    }
}

/// Trait for converting typed model structs to graph node property maps.
///
/// Implementations map struct fields to camelCase property keys matching
/// downstream consumer (AssureTwin) DTO expectations.
pub trait ToNodeProperties {
    /// Entity type name (snake_case), e.g. `"uncertain_tax_position"`.
    fn node_type_name(&self) -> &'static str;

    /// Numeric entity type code for registry, e.g. `416`.
    fn node_type_code(&self) -> u16;

    /// Convert all fields to a property map with camelCase keys.
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_property_value_to_string() {
        assert_eq!(GraphPropertyValue::Bool(true).to_string_value(), "true");
        assert_eq!(GraphPropertyValue::Bool(false).to_string_value(), "false");
        assert_eq!(GraphPropertyValue::Int(42).to_string_value(), "42");
        assert_eq!(GraphPropertyValue::Int(-7).to_string_value(), "-7");
        assert_eq!(
            GraphPropertyValue::String("hello".into()).to_string_value(),
            "hello"
        );
        assert_eq!(
            GraphPropertyValue::Float(3.14).to_string_value(),
            "3.140000"
        );
        assert_eq!(
            GraphPropertyValue::Decimal(Decimal::new(1234, 2)).to_string_value(),
            "12.34"
        );
        assert_eq!(
            GraphPropertyValue::Date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
                .to_string_value(),
            "2024-01-15"
        );
        assert_eq!(
            GraphPropertyValue::StringList(vec!["a".into(), "b".into(), "c".into()])
                .to_string_value(),
            "a;b;c"
        );
    }

    #[test]
    fn test_accessor_methods() {
        assert_eq!(
            GraphPropertyValue::String("test".into()).as_str(),
            Some("test")
        );
        assert_eq!(GraphPropertyValue::Int(42).as_str(), None);
        assert_eq!(GraphPropertyValue::Bool(true).as_bool(), Some(true));
        assert_eq!(GraphPropertyValue::String("x".into()).as_bool(), None);
        assert_eq!(
            GraphPropertyValue::Decimal(Decimal::new(100, 0)).as_decimal(),
            Some(Decimal::new(100, 0))
        );
        assert_eq!(GraphPropertyValue::Bool(true).as_decimal(), None);
        assert_eq!(GraphPropertyValue::Int(99).as_int(), Some(99));
        assert_eq!(GraphPropertyValue::Float(1.5).as_float(), Some(1.5));
        let d = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        assert_eq!(GraphPropertyValue::Date(d).as_date(), Some(d));
    }

    #[test]
    fn test_empty_string_list() {
        assert_eq!(GraphPropertyValue::StringList(vec![]).to_string_value(), "");
    }
}
