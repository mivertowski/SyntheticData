#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz YAML config parsing - should never panic
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(config) = serde_yaml::from_str::<datasynth_config::schema::GeneratorConfig>(s) {
            // If parsing succeeds, validation should not panic
            let _ = datasynth_config::validate_config(&config);
        }
    }
});
