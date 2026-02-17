//! Deep-merge logic for country pack JSON overrides.

use serde_json::Value;

use super::error::CountryPackError;
use super::schema::CountryPack;

/// Recursively merge `overlay` into `base`.
///
/// - Objects merge recursively (overlay keys win).
/// - Arrays and scalars in `overlay` replace entirely.
/// - Keys present only in `base` are preserved.
pub fn deep_merge(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_val) in overlay_map {
                let entry = base_map
                    .entry(key.clone())
                    .or_insert_with(|| Value::Null);
                deep_merge(entry, overlay_val);
            }
        }
        (base, overlay) => {
            *base = overlay.clone();
        }
    }
}

/// Apply a JSON override blob to a `CountryPack`.
///
/// Serializes the pack to a JSON `Value`, deep-merges the override,
/// then deserializes back.
pub fn apply_override(
    pack: &mut CountryPack,
    override_value: &Value,
) -> Result<(), CountryPackError> {
    let mut base = serde_json::to_value(&*pack).map_err(|e| CountryPackError::merge(e.to_string()))?;
    deep_merge(&mut base, override_value);
    *pack =
        serde_json::from_value(base).map_err(|e| CountryPackError::merge(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deep_merge_objects() {
        let mut base = json!({"a": 1, "b": {"c": 2, "d": 3}});
        let overlay = json!({"b": {"c": 99}, "e": 5});
        deep_merge(&mut base, &overlay);
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"]["c"], 99);
        assert_eq!(base["b"]["d"], 3); // preserved from base
        assert_eq!(base["e"], 5);
    }

    #[test]
    fn test_deep_merge_array_replaces() {
        let mut base = json!({"arr": [1, 2, 3]});
        let overlay = json!({"arr": [10, 20]});
        deep_merge(&mut base, &overlay);
        assert_eq!(base["arr"], json!([10, 20]));
    }

    #[test]
    fn test_deep_merge_scalar_replaces() {
        let mut base = json!({"x": "old"});
        let overlay = json!({"x": "new"});
        deep_merge(&mut base, &overlay);
        assert_eq!(base["x"], "new");
    }

    #[test]
    fn test_apply_override_to_pack() {
        let mut pack = CountryPack {
            country_code: "US".to_string(),
            ..Default::default()
        };
        let ovr = json!({"country_name": "United States of America"});
        apply_override(&mut pack, &ovr).expect("merge should succeed");
        assert_eq!(pack.country_name, "United States of America");
        assert_eq!(pack.country_code, "US"); // unchanged
    }
}
