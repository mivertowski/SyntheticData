#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz DSF fingerprint file loading - should never panic
    let cursor = std::io::Cursor::new(data);
    let reader = datasynth_fingerprint::io::FingerprintReader::new();
    let _ = reader.read(cursor);
});
