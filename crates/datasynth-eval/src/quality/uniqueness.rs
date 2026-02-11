//! Uniqueness and duplicate detection evaluation.
//!
//! Detects exact duplicates, near-duplicates, and validates primary key uniqueness.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Results of uniqueness analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniquenessAnalysis {
    /// Total records analyzed.
    pub total_records: usize,
    /// Exact duplicate count.
    pub exact_duplicates: usize,
    /// Near-duplicate count (similarity > threshold).
    pub near_duplicates: usize,
    /// Duplicate rate (0.0-1.0).
    pub duplicate_rate: f64,
    /// Primary key collisions.
    pub pk_collisions: usize,
    /// Document number collisions.
    pub doc_number_collisions: usize,
    /// Duplicate groups (records that are duplicates of each other).
    pub duplicate_groups: Vec<DuplicateInfo>,
    /// Uniqueness score (1.0 - duplicate_rate).
    pub uniqueness_score: f64,
}

/// Information about a group of duplicates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateInfo {
    /// Type of duplicate.
    pub duplicate_type: DuplicateType,
    /// Number of records in this duplicate group.
    pub count: usize,
    /// Example record identifiers.
    pub example_ids: Vec<String>,
    /// Similarity score for near-duplicates.
    pub similarity: Option<f64>,
}

/// Type of duplicate detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuplicateType {
    /// Exact match on all fields.
    Exact,
    /// High similarity but not exact.
    NearDuplicate,
    /// Same primary key.
    PrimaryKeyCollision,
    /// Same document number.
    DocumentNumberCollision,
}

/// A record for uniqueness checking.
#[derive(Debug, Clone)]
pub struct UniqueRecord {
    /// Primary key value.
    pub primary_key: String,
    /// Document number (if applicable).
    pub document_number: Option<String>,
    /// Hash of record content for exact duplicate detection.
    pub content_hash: u64,
    /// Key fields for near-duplicate detection.
    pub key_fields: Vec<String>,
}

/// Analyzer for uniqueness.
pub struct UniquenessAnalyzer {
    /// Similarity threshold for near-duplicates.
    similarity_threshold: f64,
    /// Maximum duplicates to report in detail.
    max_report_duplicates: usize,
}

impl UniquenessAnalyzer {
    /// Create a new analyzer with specified threshold.
    pub fn new(similarity_threshold: f64) -> Self {
        Self {
            similarity_threshold,
            max_report_duplicates: 100,
        }
    }

    /// Analyze uniqueness of records.
    pub fn analyze(&self, records: &[UniqueRecord]) -> EvalResult<UniquenessAnalysis> {
        let total_records = records.len();
        let mut duplicate_groups = Vec::new();

        // Exact duplicate detection via hash
        let mut hash_counts: HashMap<u64, Vec<usize>> = HashMap::new();
        for (idx, record) in records.iter().enumerate() {
            hash_counts
                .entry(record.content_hash)
                .or_default()
                .push(idx);
        }

        let mut exact_duplicates = 0;
        for indices in hash_counts.values() {
            if indices.len() > 1 {
                exact_duplicates += indices.len() - 1;
                if duplicate_groups.len() < self.max_report_duplicates {
                    duplicate_groups.push(DuplicateInfo {
                        duplicate_type: DuplicateType::Exact,
                        count: indices.len(),
                        example_ids: indices
                            .iter()
                            .take(3)
                            .map(|&i| records[i].primary_key.clone())
                            .collect(),
                        similarity: Some(1.0),
                    });
                }
            }
        }

        // Primary key collision detection
        let mut pk_seen: HashSet<&str> = HashSet::new();
        let mut pk_collisions = 0;
        for record in records {
            if !pk_seen.insert(&record.primary_key) {
                pk_collisions += 1;
            }
        }

        // Document number collision detection
        let mut doc_seen: HashSet<&str> = HashSet::new();
        let mut doc_number_collisions = 0;
        for record in records {
            if let Some(ref doc_num) = record.document_number {
                if !doc_seen.insert(doc_num) {
                    doc_number_collisions += 1;
                }
            }
        }

        // Near-duplicate detection (simplified - check key field similarity)
        let near_duplicates = self.detect_near_duplicates(records, &mut duplicate_groups);

        let duplicate_rate = if total_records > 0 {
            (exact_duplicates + near_duplicates) as f64 / total_records as f64
        } else {
            0.0
        };

        let uniqueness_score = 1.0 - duplicate_rate;

        Ok(UniquenessAnalysis {
            total_records,
            exact_duplicates,
            near_duplicates,
            duplicate_rate,
            pk_collisions,
            doc_number_collisions,
            duplicate_groups,
            uniqueness_score,
        })
    }

    /// Detect near-duplicates based on key field similarity.
    fn detect_near_duplicates(
        &self,
        records: &[UniqueRecord],
        duplicate_groups: &mut Vec<DuplicateInfo>,
    ) -> usize {
        let mut near_duplicates = 0;

        // For efficiency, only check a sample if dataset is large
        let sample_size = records.len().min(1000);
        let step = if records.len() > sample_size {
            records.len() / sample_size
        } else {
            1
        };

        let sampled: Vec<_> = records.iter().step_by(step).take(sample_size).collect();

        for i in 0..sampled.len() {
            for j in (i + 1)..sampled.len() {
                let sim = self.calculate_similarity(&sampled[i].key_fields, &sampled[j].key_fields);
                if sim >= self.similarity_threshold && sim < 1.0 {
                    near_duplicates += 1;
                    if duplicate_groups.len() < self.max_report_duplicates {
                        duplicate_groups.push(DuplicateInfo {
                            duplicate_type: DuplicateType::NearDuplicate,
                            count: 2,
                            example_ids: vec![
                                sampled[i].primary_key.clone(),
                                sampled[j].primary_key.clone(),
                            ],
                            similarity: Some(sim),
                        });
                    }
                }
            }
        }

        // Extrapolate if we sampled
        if step > 1 {
            near_duplicates = near_duplicates * step * step;
        }

        near_duplicates
    }

    /// Calculate Jaccard similarity between two sets of key fields.
    fn calculate_similarity(&self, fields1: &[String], fields2: &[String]) -> f64 {
        if fields1.is_empty() && fields2.is_empty() {
            return 1.0;
        }

        let set1: HashSet<_> = fields1.iter().collect();
        let set2: HashSet<_> = fields2.iter().collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            1.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

impl Default for UniquenessAnalyzer {
    fn default() -> Self {
        Self::new(0.9) // 90% similarity threshold
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_content(s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn test_no_duplicates() {
        let records = vec![
            UniqueRecord {
                primary_key: "1".to_string(),
                document_number: Some("DOC001".to_string()),
                content_hash: hash_content("record1"),
                key_fields: vec!["a".to_string(), "b".to_string()],
            },
            UniqueRecord {
                primary_key: "2".to_string(),
                document_number: Some("DOC002".to_string()),
                content_hash: hash_content("record2"),
                key_fields: vec!["c".to_string(), "d".to_string()],
            },
        ];

        let analyzer = UniquenessAnalyzer::default();
        let result = analyzer.analyze(&records).unwrap();

        assert_eq!(result.exact_duplicates, 0);
        assert_eq!(result.pk_collisions, 0);
        assert_eq!(result.doc_number_collisions, 0);
    }

    #[test]
    fn test_exact_duplicates() {
        let hash = hash_content("same_content");
        let records = vec![
            UniqueRecord {
                primary_key: "1".to_string(),
                document_number: Some("DOC001".to_string()),
                content_hash: hash,
                key_fields: vec!["a".to_string()],
            },
            UniqueRecord {
                primary_key: "2".to_string(),
                document_number: Some("DOC002".to_string()),
                content_hash: hash, // Same hash = duplicate
                key_fields: vec!["a".to_string()],
            },
        ];

        let analyzer = UniquenessAnalyzer::default();
        let result = analyzer.analyze(&records).unwrap();

        assert_eq!(result.exact_duplicates, 1);
    }

    #[test]
    fn test_pk_collision() {
        let records = vec![
            UniqueRecord {
                primary_key: "SAME_PK".to_string(),
                document_number: None,
                content_hash: hash_content("record1"),
                key_fields: vec![],
            },
            UniqueRecord {
                primary_key: "SAME_PK".to_string(),
                document_number: None,
                content_hash: hash_content("record2"),
                key_fields: vec![],
            },
        ];

        let analyzer = UniquenessAnalyzer::default();
        let result = analyzer.analyze(&records).unwrap();

        assert_eq!(result.pk_collisions, 1);
    }
}
