#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz individual YAML config subsections
    if let Ok(s) = std::str::from_utf8(data) {
        // Try parsing as various config subsections
        let _ = serde_yaml::from_str::<datasynth_config::schema::GlobalConfig>(s);
        let _ = serde_yaml::from_str::<datasynth_config::schema::CompanyConfig>(s);
        let _ = serde_yaml::from_str::<datasynth_config::schema::OutputConfig>(s);
        let _ = serde_yaml::from_str::<datasynth_config::schema::FraudConfig>(s);
        let _ = serde_yaml::from_str::<datasynth_config::schema::TransactionConfig>(s);
    }
});
