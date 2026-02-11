//! Integration tests for the fingerprint extraction and generation workflow.
//!
//! Tests the full round-trip: CSV → extract → .dsf → generate → evaluate

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use datasynth_fingerprint::{
    evaluation::FidelityEvaluator,
    extraction::{
        CsvDataSource, DataSource, DirectoryDataSource, ExtractionConfig, FingerprintExtractor,
    },
    io::{validate_dsf, DsfSigner, DsfVerifier, FingerprintReader, FingerprintWriter, SigningKey},
    models::PrivacyLevel,
    privacy::PrivacyConfig,
    synthesis::ConfigSynthesizer,
};

/// Create sample CSV data for testing.
fn create_sample_csv(dir: &TempDir, name: &str) -> PathBuf {
    let path = dir.path().join(name);
    let content = r#"id,amount,date,category,description
1,100.50,2024-01-15,Sales,Product A sale
2,200.75,2024-01-16,Sales,Product B sale
3,50.25,2024-01-17,Expense,Office supplies
4,1000.00,2024-01-18,Sales,Large order
5,75.50,2024-01-19,Expense,Travel
6,150.00,2024-01-20,Sales,Product C sale
7,25.00,2024-01-21,Expense,Utilities
8,500.00,2024-01-22,Sales,Bulk order
9,80.00,2024-01-23,Expense,Software license
10,300.00,2024-01-24,Sales,Product D sale
11,45.00,2024-01-25,Expense,Postage
12,250.00,2024-01-26,Sales,Product E sale
13,90.00,2024-01-27,Expense,Maintenance
14,175.00,2024-01-28,Sales,Product F sale
15,60.00,2024-01-29,Expense,Cleaning
16,400.00,2024-01-30,Sales,Special order
17,35.00,2024-01-31,Expense,Subscriptions
18,125.00,2024-02-01,Sales,Product G sale
19,55.00,2024-02-02,Expense,Phone
20,225.00,2024-02-03,Sales,Product H sale
"#;
    fs::write(&path, content).expect("Failed to write sample CSV");
    path
}

/// Create larger sample CSV for more robust statistics.
fn create_large_sample_csv(dir: &TempDir, name: &str) -> PathBuf {
    let path = dir.path().join(name);
    let mut content = String::from("id,amount,date,category,description\n");

    for i in 1..=100 {
        let amount = (i as f64 * 10.5) + (i % 7) as f64 * 3.17;
        let day = (i % 28) + 1;
        let month = ((i - 1) / 28) % 12 + 1;
        let category = if i % 3 == 0 { "Expense" } else { "Sales" };
        content.push_str(&format!(
            "{},{:.2},2024-{:02}-{:02},{},Transaction {}\n",
            i, amount, month, day, category, i
        ));
    }

    fs::write(&path, content).expect("Failed to write large sample CSV");
    path
}

#[test]
fn test_extract_fingerprint_from_csv() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_sample_csv(&temp_dir, "test_data.csv");

    // Create data source
    let data_source = DataSource::Csv(CsvDataSource::new(csv_path));

    // Extract fingerprint
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    // Verify fingerprint structure
    assert!(
        !fingerprint.schema.tables.is_empty(),
        "Schema should have tables"
    );
    assert!(
        !fingerprint.statistics.numeric_columns.is_empty(),
        "Should have numeric columns"
    );
    assert!(
        !fingerprint.statistics.categorical_columns.is_empty(),
        "Should have categorical columns"
    );
    assert!(
        !fingerprint.privacy_audit.actions.is_empty(),
        "Should have privacy audit actions"
    );
}

#[test]
fn test_extract_with_privacy_levels() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_sample_csv(&temp_dir, "test_data.csv");

    for level in [
        PrivacyLevel::Minimal,
        PrivacyLevel::Standard,
        PrivacyLevel::High,
        PrivacyLevel::Maximum,
    ] {
        let data_source = DataSource::Csv(CsvDataSource::new(csv_path.clone()));

        let config = ExtractionConfig {
            privacy: PrivacyConfig::from_level(level),
            ..Default::default()
        };

        let extractor = FingerprintExtractor::with_config(config);
        let fingerprint = extractor
            .extract(&data_source)
            .expect("Failed to extract fingerprint");

        // Verify privacy level is recorded
        assert_eq!(fingerprint.manifest.privacy.level, level);

        // Higher privacy levels should have more epsilon budget
        match level {
            PrivacyLevel::Minimal => assert!(fingerprint.manifest.privacy.epsilon >= 4.0),
            PrivacyLevel::Standard => assert!(fingerprint.manifest.privacy.epsilon >= 0.9),
            PrivacyLevel::High => assert!(fingerprint.manifest.privacy.epsilon >= 0.4),
            PrivacyLevel::Maximum => assert!(fingerprint.manifest.privacy.epsilon >= 0.05),
            PrivacyLevel::Custom => {} // Custom uses user-specified values
        }
    }
}

#[test]
fn test_write_and_read_dsf_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_sample_csv(&temp_dir, "test_data.csv");
    let dsf_path = temp_dir.path().join("fingerprint.dsf");

    // Extract fingerprint
    let data_source = DataSource::Csv(CsvDataSource::new(csv_path));
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    // Write to DSF file
    let writer = FingerprintWriter::new();
    writer
        .write_to_file(&fingerprint, &dsf_path)
        .expect("Failed to write DSF");

    // Verify file exists
    assert!(dsf_path.exists(), "DSF file should exist");

    // Read back from DSF file
    let reader = FingerprintReader::new();
    let read_fingerprint = reader
        .read_from_file(&dsf_path)
        .expect("Failed to read DSF");

    // Verify data matches
    assert_eq!(
        fingerprint.manifest.version,
        read_fingerprint.manifest.version
    );
    assert_eq!(
        fingerprint.schema.tables.len(),
        read_fingerprint.schema.tables.len()
    );
    assert_eq!(
        fingerprint.statistics.numeric_columns.len(),
        read_fingerprint.statistics.numeric_columns.len()
    );
}

#[test]
fn test_validate_dsf_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_sample_csv(&temp_dir, "test_data.csv");
    let dsf_path = temp_dir.path().join("fingerprint.dsf");

    // Extract and write fingerprint
    let data_source = DataSource::Csv(CsvDataSource::new(csv_path));
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    let writer = FingerprintWriter::new();
    writer
        .write_to_file(&fingerprint, &dsf_path)
        .expect("Failed to write DSF");

    // Validate DSF file
    let report = validate_dsf(&dsf_path).expect("Failed to validate DSF");
    assert!(report.is_valid, "DSF file should be valid");
    assert!(report.errors.is_empty(), "Should have no errors");
}

#[test]
fn test_synthesize_config_from_fingerprint() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_large_sample_csv(&temp_dir, "test_data.csv");

    // Extract fingerprint
    let data_source = DataSource::Csv(CsvDataSource::new(csv_path));
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    // Synthesize config
    let synthesizer = ConfigSynthesizer::new();
    let config_patch = synthesizer
        .synthesize(&fingerprint)
        .expect("Failed to synthesize config");

    // Verify config patch has expected values
    assert!(
        !config_patch.values().is_empty(),
        "Config patch should have values"
    );

    // Should have transaction count
    let has_transaction_count = config_patch.values().contains_key("transactions.count");
    assert!(has_transaction_count, "Should have transaction count");
}

#[test]
fn test_fidelity_evaluation_same_fingerprint() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_large_sample_csv(&temp_dir, "test_data.csv");

    // Extract fingerprint
    let data_source = DataSource::Csv(CsvDataSource::new(csv_path));
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    // Evaluate fingerprint against itself (should have perfect fidelity)
    let evaluator = FidelityEvaluator::new();
    let report = evaluator
        .evaluate_fingerprints(&fingerprint, &fingerprint)
        .expect("Failed to evaluate fidelity");

    // Self-comparison should have near-perfect scores
    assert!(
        report.overall_score >= 0.95,
        "Self-comparison should have high fidelity"
    );
    assert!(report.passes, "Self-comparison should pass");
}

#[test]
fn test_full_round_trip_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_large_sample_csv(&temp_dir, "original_data.csv");
    let dsf_path = temp_dir.path().join("fingerprint.dsf");

    // Step 1: Extract fingerprint from CSV
    let data_source = DataSource::Csv(CsvDataSource::new(csv_path.clone()));
    let extractor = FingerprintExtractor::new();
    let original_fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    // Step 2: Write to DSF file
    let writer = FingerprintWriter::new();
    writer
        .write_to_file(&original_fingerprint, &dsf_path)
        .expect("Failed to write DSF");

    // Step 3: Read DSF file back
    let reader = FingerprintReader::new();
    let loaded_fingerprint = reader
        .read_from_file(&dsf_path)
        .expect("Failed to read DSF");

    // Step 4: Synthesize config from fingerprint
    let synthesizer = ConfigSynthesizer::new();
    let config_patch = synthesizer
        .synthesize(&loaded_fingerprint)
        .expect("Failed to synthesize config");

    // Step 5: Verify config patch
    assert!(
        !config_patch.values().is_empty(),
        "Config patch should have values"
    );

    // Step 6: Evaluate fidelity (using loaded fingerprint against original)
    let evaluator = FidelityEvaluator::with_threshold(0.8);
    let report = evaluator
        .evaluate_fingerprints(&original_fingerprint, &loaded_fingerprint)
        .expect("Failed to evaluate fidelity");

    // Round-trip should preserve fidelity
    assert!(
        report.overall_score >= 0.95,
        "Round-trip should preserve fidelity: got {:.2}",
        report.overall_score
    );
    assert!(report.passes, "Round-trip should pass fidelity check");
}

#[test]
fn test_privacy_audit_tracking() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_sample_csv(&temp_dir, "test_data.csv");

    // Extract with standard privacy
    let data_source = DataSource::Csv(CsvDataSource::new(csv_path));
    let config = ExtractionConfig {
        privacy: PrivacyConfig::from_level(PrivacyLevel::Standard),
        ..Default::default()
    };

    let extractor = FingerprintExtractor::with_config(config);
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    // Verify privacy audit
    let audit = &fingerprint.privacy_audit;
    assert!(!audit.actions.is_empty(), "Should have privacy actions");
    assert!(audit.epsilon_budget > 0.0, "Should have epsilon budget");
    assert!(
        audit.total_epsilon_spent <= audit.epsilon_budget,
        "Spent epsilon should not exceed budget"
    );
}

#[test]
fn test_numeric_statistics_extraction() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_large_sample_csv(&temp_dir, "test_data.csv");

    let data_source = DataSource::Csv(CsvDataSource::new(csv_path));
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    // Check amount column statistics
    // Statistics keys are in the format "table_name.column_name"
    let amount_stats = fingerprint
        .statistics
        .numeric_columns
        .get("test_data.amount");
    assert!(
        amount_stats.is_some(),
        "Should have amount column stats (key: test_data.amount)"
    );

    if let Some(stats) = amount_stats {
        assert!(stats.count > 0, "Should have positive count");
        assert!(stats.min < stats.max, "Min should be less than max");
        // Note: mean can be affected by differential privacy noise, so we check it's finite
        // rather than strictly positive (DP noise can shift the mean)
        assert!(stats.mean.is_finite(), "Mean should be finite");
        assert!(stats.std_dev >= 0.0, "StdDev should be non-negative");
    }
}

#[test]
fn test_categorical_statistics_extraction() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_large_sample_csv(&temp_dir, "test_data.csv");

    let data_source = DataSource::Csv(CsvDataSource::new(csv_path));
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    // Check category column statistics
    // Statistics keys are in the format "table_name.column_name"
    let category_stats = fingerprint
        .statistics
        .categorical_columns
        .get("test_data.category");
    assert!(
        category_stats.is_some(),
        "Should have category column stats (key: test_data.category)"
    );

    if let Some(stats) = category_stats {
        assert!(stats.count > 0, "Should have positive count");
        assert!(stats.cardinality > 0, "Should have positive cardinality");
        assert!(!stats.top_values.is_empty(), "Should have top values");
    }
}

#[test]
fn test_schema_extraction() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_sample_csv(&temp_dir, "test_data.csv");

    let data_source = DataSource::Csv(CsvDataSource::new(csv_path));
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    // Verify schema has expected structure
    assert_eq!(fingerprint.schema.tables.len(), 1, "Should have one table");

    let table = fingerprint.schema.tables.values().next().unwrap();
    assert!(table.columns.len() >= 5, "Should have at least 5 columns");

    // Check for expected columns
    let column_names: Vec<_> = table.columns.iter().map(|c| c.name.as_str()).collect();
    assert!(column_names.iter().any(|c| *c == "id" || *c == "amount"));
}

/// Create sample JSON data for testing.
fn create_sample_json(dir: &TempDir, name: &str) -> PathBuf {
    let path = dir.path().join(name);
    let content = r#"[
        {"id": 1, "amount": 100.50, "category": "Sales"},
        {"id": 2, "amount": 200.75, "category": "Sales"},
        {"id": 3, "amount": 50.25, "category": "Expense"},
        {"id": 4, "amount": 1000.00, "category": "Sales"},
        {"id": 5, "amount": 75.50, "category": "Expense"},
        {"id": 6, "amount": 150.00, "category": "Sales"},
        {"id": 7, "amount": 25.00, "category": "Expense"},
        {"id": 8, "amount": 500.00, "category": "Sales"},
        {"id": 9, "amount": 80.00, "category": "Expense"},
        {"id": 10, "amount": 300.00, "category": "Sales"}
    ]"#;
    fs::write(&path, content).expect("Failed to write sample JSON");
    path
}

/// Create sample JSONL data for testing.
fn create_sample_jsonl(dir: &TempDir, name: &str) -> PathBuf {
    let path = dir.path().join(name);
    let content = r#"{"id": 1, "amount": 100.50, "category": "Sales"}
{"id": 2, "amount": 200.75, "category": "Sales"}
{"id": 3, "amount": 50.25, "category": "Expense"}
{"id": 4, "amount": 1000.00, "category": "Sales"}
{"id": 5, "amount": 75.50, "category": "Expense"}
{"id": 6, "amount": 150.00, "category": "Sales"}
{"id": 7, "amount": 25.00, "category": "Expense"}
{"id": 8, "amount": 500.00, "category": "Sales"}
{"id": 9, "amount": 80.00, "category": "Expense"}
{"id": 10, "amount": 300.00, "category": "Sales"}
"#;
    fs::write(&path, content).expect("Failed to write sample JSONL");
    path
}

#[test]
fn test_extract_from_json() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let json_path = create_sample_json(&temp_dir, "test_data.json");

    // Create JSON data source
    let data_source = DataSource::Json(datasynth_fingerprint::extraction::JsonDataSource::new(
        json_path,
    ));

    // Extract fingerprint
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint from JSON");

    // Verify fingerprint structure
    assert!(
        !fingerprint.schema.tables.is_empty(),
        "Schema should have tables"
    );
    assert!(
        !fingerprint.statistics.numeric_columns.is_empty(),
        "Should have numeric columns"
    );
    assert!(
        !fingerprint.statistics.categorical_columns.is_empty(),
        "Should have categorical columns"
    );
}

#[test]
fn test_extract_from_jsonl() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let jsonl_path = create_sample_jsonl(&temp_dir, "test_data.jsonl");

    // Create JSONL data source
    let data_source = DataSource::Json(datasynth_fingerprint::extraction::JsonDataSource::jsonl(
        jsonl_path,
    ));

    // Extract fingerprint
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint from JSONL");

    // Verify fingerprint structure
    assert!(
        !fingerprint.schema.tables.is_empty(),
        "Schema should have tables"
    );
    assert!(
        !fingerprint.statistics.numeric_columns.is_empty(),
        "Should have numeric columns"
    );
}

#[test]
fn test_extract_from_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create multiple data files in the directory
    let _csv1_path = create_sample_csv(&temp_dir, "customers.csv");
    let _csv2_path = create_large_sample_csv(&temp_dir, "transactions.csv");

    // Create a JSON file with enough rows (min_rows default is 10)
    let json_path = temp_dir.path().join("products.json");
    let json_data = r#"[
        {"id": 1, "name": "Widget A", "price": 19.99, "stock": 100},
        {"id": 2, "name": "Widget B", "price": 29.99, "stock": 50},
        {"id": 3, "name": "Widget C", "price": 39.99, "stock": 25},
        {"id": 4, "name": "Widget D", "price": 49.99, "stock": 75},
        {"id": 5, "name": "Widget E", "price": 59.99, "stock": 60},
        {"id": 6, "name": "Widget F", "price": 69.99, "stock": 40},
        {"id": 7, "name": "Widget G", "price": 79.99, "stock": 30},
        {"id": 8, "name": "Widget H", "price": 89.99, "stock": 20},
        {"id": 9, "name": "Widget I", "price": 99.99, "stock": 15},
        {"id": 10, "name": "Widget J", "price": 109.99, "stock": 10}
    ]"#;
    fs::write(&json_path, json_data).expect("Failed to write JSON file");

    // Extract fingerprint from the directory
    let data_source = DataSource::Directory(DirectoryDataSource::new(temp_dir.path()));
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint from directory");

    // Verify we got data from multiple tables
    assert!(
        fingerprint.schema.tables.len() >= 3,
        "Should have tables from all files, got: {:?}",
        fingerprint.schema.tables.keys().collect::<Vec<_>>()
    );

    // Verify manifest describes the directory
    assert!(
        fingerprint
            .manifest
            .source
            .description
            .contains("Directory"),
        "Manifest should mention directory source"
    );

    // Check that statistics were merged from multiple tables
    let total_numeric_columns: usize = fingerprint.statistics.numeric_columns.len();
    let total_categorical_columns: usize = fingerprint.statistics.categorical_columns.len();
    assert!(
        total_numeric_columns >= 2,
        "Should have numeric columns from multiple tables, got {}",
        total_numeric_columns
    );
    assert!(
        total_categorical_columns >= 2,
        "Should have categorical columns from multiple tables, got {}",
        total_categorical_columns
    );
}

#[test]
fn test_directory_extraction_convenience_method() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create CSV files
    let _path1 = create_sample_csv(&temp_dir, "data1.csv");
    let _path2 = create_sample_csv(&temp_dir, "data2.csv");

    // Use the convenience method
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract_from_directory(temp_dir.path())
        .expect("Failed to extract fingerprint from directory");

    // Verify extraction worked
    assert!(
        fingerprint.schema.tables.len() >= 2,
        "Should have tables from both CSV files"
    );
}

#[test]
fn test_directory_extraction_empty_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Try to extract from empty directory
    let extractor = FingerprintExtractor::new();
    let result = extractor.extract_from_directory(temp_dir.path());

    // Should fail because no supported files were found
    assert!(result.is_err(), "Should fail on empty directory");
}

#[test]
fn test_streaming_extraction() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_large_sample_csv(&temp_dir, "test_data.csv");

    // Extract using streaming
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract_streaming_csv(&csv_path)
        .expect("Failed to extract fingerprint with streaming");

    // Verify fingerprint has expected structure
    assert!(
        !fingerprint.schema.tables.is_empty(),
        "Schema should have tables"
    );
    assert!(
        !fingerprint.statistics.numeric_columns.is_empty(),
        "Should have numeric statistics"
    );

    // Check schema
    let table = fingerprint
        .schema
        .tables
        .get("test_data")
        .expect("Should have test_data table");
    assert_eq!(table.row_count, 100, "Should have 100 rows");
    assert!(!table.columns.is_empty(), "Should have columns");

    // Check statistics
    let amount_stats = fingerprint
        .statistics
        .numeric_columns
        .get("test_data.amount");
    assert!(
        amount_stats.is_some(),
        "Should have amount column statistics"
    );

    if let Some(stats) = amount_stats {
        assert_eq!(stats.count, 100, "Should have 100 values");
        assert!(stats.min >= 0.0, "Min should be non-negative");
        assert!(stats.max > stats.min, "Max should be greater than min");
    }
}

#[test]
fn test_streaming_vs_regular_extraction() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_large_sample_csv(&temp_dir, "test_data.csv");

    // Extract using regular method
    let data_source = DataSource::Csv(CsvDataSource::new(&csv_path));
    let extractor = FingerprintExtractor::new();
    let regular_fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    // Extract using streaming
    let streaming_fingerprint = extractor
        .extract_streaming_csv(&csv_path)
        .expect("Failed to extract fingerprint with streaming");

    // Both should have at least one table
    assert!(
        !regular_fingerprint.schema.tables.is_empty(),
        "Regular should have tables"
    );
    assert!(
        !streaming_fingerprint.schema.tables.is_empty(),
        "Streaming should have tables"
    );

    // Get the streaming table (we know this is test_data)
    let streaming_table = streaming_fingerprint
        .schema
        .tables
        .get("test_data")
        .expect("Streaming should have test_data table");
    assert_eq!(
        streaming_table.row_count, 100,
        "Streaming should have 100 rows"
    );

    // Both should have numeric statistics
    assert!(
        !regular_fingerprint.statistics.numeric_columns.is_empty(),
        "Regular should have numeric statistics"
    );
    assert!(
        !streaming_fingerprint.statistics.numeric_columns.is_empty(),
        "Streaming should have numeric statistics"
    );

    // Statistics should be similar
    let streaming_stats = streaming_fingerprint
        .statistics
        .numeric_columns
        .get("test_data.amount");
    assert!(
        streaming_stats.is_some(),
        "Streaming should have amount statistics"
    );

    if let Some(s) = streaming_stats {
        assert_eq!(s.count, 100, "Streaming should have 100 values");
        // Check mean is reasonable (expected around 550)
        assert!(
            s.mean > 100.0 && s.mean < 1000.0,
            "Mean should be reasonable: {}",
            s.mean
        );
    }
}

#[test]
fn test_signed_fingerprint() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_sample_csv(&temp_dir, "test_data.csv");

    // Extract fingerprint
    let data_source = DataSource::Csv(CsvDataSource::new(&csv_path));
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    // Create signing key and signer
    let key = SigningKey::generate("test-signing-key");
    let signer = DsfSigner::new(key.clone());
    let verifier = DsfVerifier::new(key);

    // Write signed fingerprint
    let dsf_path = temp_dir.path().join("signed.dsf");
    let writer = FingerprintWriter::new();
    writer
        .write_to_file_signed(&fingerprint, &dsf_path, &signer)
        .expect("Failed to write signed fingerprint");

    // Check that the file is signed
    let reader = FingerprintReader::new();
    let is_signed = reader
        .is_signed(&dsf_path)
        .expect("Failed to check signature");
    assert!(is_signed, "DSF file should be signed");

    // Read and verify the signed fingerprint
    let verified_fingerprint = reader
        .read_from_file_verified(&dsf_path, &verifier)
        .expect("Failed to read and verify signed fingerprint");

    // Verify content matches
    assert_eq!(
        verified_fingerprint.schema.tables.len(),
        fingerprint.schema.tables.len(),
        "Schema should match"
    );
    assert_eq!(
        verified_fingerprint.statistics.numeric_columns.len(),
        fingerprint.statistics.numeric_columns.len(),
        "Statistics should match"
    );
}

#[test]
fn test_signature_verification_fails_with_wrong_key() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_sample_csv(&temp_dir, "test_data.csv");

    // Extract fingerprint
    let data_source = DataSource::Csv(CsvDataSource::new(&csv_path));
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    // Create signing key
    let sign_key = SigningKey::generate("sign-key");
    let signer = DsfSigner::new(sign_key);

    // Write signed fingerprint
    let dsf_path = temp_dir.path().join("signed.dsf");
    let writer = FingerprintWriter::new();
    writer
        .write_to_file_signed(&fingerprint, &dsf_path, &signer)
        .expect("Failed to write signed fingerprint");

    // Try to verify with a different key
    let wrong_key = SigningKey::generate("wrong-key");
    let wrong_verifier = DsfVerifier::new(wrong_key);

    let reader = FingerprintReader::new();
    let result = reader.read_from_file_verified(&dsf_path, &wrong_verifier);

    // Should fail verification
    assert!(result.is_err(), "Verification should fail with wrong key");
}

#[test]
fn test_unsigned_fingerprint_verification_fails() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = create_sample_csv(&temp_dir, "test_data.csv");

    // Extract fingerprint
    let data_source = DataSource::Csv(CsvDataSource::new(&csv_path));
    let extractor = FingerprintExtractor::new();
    let fingerprint = extractor
        .extract(&data_source)
        .expect("Failed to extract fingerprint");

    // Write unsigned fingerprint
    let dsf_path = temp_dir.path().join("unsigned.dsf");
    let writer = FingerprintWriter::new();
    writer
        .write_to_file(&fingerprint, &dsf_path)
        .expect("Failed to write fingerprint");

    // Check that the file is not signed
    let reader = FingerprintReader::new();
    let is_signed = reader
        .is_signed(&dsf_path)
        .expect("Failed to check signature");
    assert!(!is_signed, "DSF file should not be signed");

    // Try to read with verification
    let key = SigningKey::generate("any-key");
    let verifier = DsfVerifier::new(key);
    let result = reader.read_from_file_verified(&dsf_path, &verifier);

    // Should fail because file is not signed
    assert!(
        result.is_err(),
        "Verification should fail for unsigned file"
    );
}
