use serde_json::Value;

pub const FRAUD_PACKS: &[&str] = &[
    "revenue_fraud",
    "payroll_ghost",
    "vendor_kickback",
    "management_override",
    "comprehensive",
];

pub fn load_fraud_pack(name: &str) -> Option<Value> {
    let yaml_str = match name {
        "revenue_fraud" => include_str!("revenue_fraud.yaml"),
        "payroll_ghost" => include_str!("payroll_ghost.yaml"),
        "vendor_kickback" => include_str!("vendor_kickback.yaml"),
        "management_override" => include_str!("management_override.yaml"),
        "comprehensive" => include_str!("comprehensive.yaml"),
        _ => return None,
    };
    serde_yaml::from_str(yaml_str).ok()
}

pub fn merge_fraud_pack(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_val) in overlay_map {
                let entry = base_map.entry(key.clone()).or_insert(Value::Null);
                merge_fraud_pack(entry, overlay_val);
            }
        }
        (base, overlay) => {
            *base = overlay.clone();
        }
    }
}

pub fn apply_fraud_packs(
    config: &crate::GeneratorConfig,
    pack_names: &[String],
) -> Result<crate::GeneratorConfig, String> {
    let mut config_json =
        serde_json::to_value(config).map_err(|e| format!("Failed to serialize config: {}", e))?;

    for name in pack_names {
        let pack = load_fraud_pack(name).ok_or_else(|| {
            format!(
                "Unknown fraud pack: '{}'. Available: {:?}",
                name, FRAUD_PACKS
            )
        })?;
        merge_fraud_pack(&mut config_json, &pack);
    }

    strip_nulls(&mut config_json);

    serde_json::from_value(config_json)
        .map_err(|e| format!("Failed to deserialize merged config: {}", e))
}

fn strip_nulls(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, v| !v.is_null());
            for v in map.values_mut() {
                strip_nulls(v);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                strip_nulls(v);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_packs() {
        for name in FRAUD_PACKS {
            let pack = load_fraud_pack(name);
            assert!(pack.is_some(), "Failed to load pack: {}", name);
        }
    }

    #[test]
    fn test_load_unknown_pack_returns_none() {
        assert!(load_fraud_pack("nonexistent").is_none());
    }

    #[test]
    fn test_merge_fraud_pack_overwrites() {
        let mut base = serde_json::json!({"fraud": {"enabled": false, "fraud_rate": 0.01}});
        let overlay = serde_json::json!({"fraud": {"enabled": true, "fraud_rate": 0.05}});
        merge_fraud_pack(&mut base, &overlay);
        assert_eq!(base["fraud"]["enabled"], true);
        assert_eq!(base["fraud"]["fraud_rate"], 0.05);
    }

    #[test]
    fn test_merge_preserves_non_overlapping() {
        let mut base = serde_json::json!({"fraud": {"enabled": false}, "other": "keep"});
        let overlay = serde_json::json!({"fraud": {"enabled": true}});
        merge_fraud_pack(&mut base, &overlay);
        assert_eq!(base["other"], "keep");
    }
}
