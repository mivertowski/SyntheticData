//! Bidirectional ID mapping between string external IDs and numeric u64 IDs.
//!
//! The [`IdMap`] is the single source of truth for ID assignment in the pipeline.
//! Every node gets a unique u64 ID, and the map supports both forward (string → u64)
//! and reverse (u64 → string) lookups. Aliases allow multiple external IDs to map
//! to the same numeric ID (e.g., "VENDOR-001" and "V001" for the same node).

use std::collections::HashMap;

/// Bidirectional map: external string ID ↔ numeric u64 ID.
///
/// IDs are auto-assigned starting from 1 (0 is reserved).
#[derive(Debug, Clone)]
pub struct IdMap {
    /// Forward map: external_id → numeric_id.
    forward: HashMap<String, u64>,
    /// Reverse map: numeric_id → primary external_id.
    reverse: HashMap<u64, String>,
    /// Next ID to assign.
    next_id: u64,
}

impl IdMap {
    /// Create a new empty IdMap. IDs start from 1.
    pub fn new() -> Self {
        Self {
            forward: HashMap::new(),
            reverse: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new IdMap with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            forward: HashMap::with_capacity(capacity),
            reverse: HashMap::with_capacity(capacity),
            next_id: 1,
        }
    }

    /// Insert or get the numeric ID for the given external ID.
    ///
    /// If the external ID already exists, returns the existing numeric ID.
    /// Otherwise, assigns a new numeric ID and returns it.
    pub fn get_or_insert(&mut self, external_id: &str) -> u64 {
        if let Some(&id) = self.forward.get(external_id) {
            return id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.forward.insert(external_id.to_owned(), id);
        self.reverse.insert(id, external_id.to_owned());
        id
    }

    /// Register an alias that maps to the same numeric ID as the primary external ID.
    ///
    /// Returns `Some(numeric_id)` if the primary exists, `None` otherwise.
    pub fn alias(&mut self, alias: &str, primary_external_id: &str) -> Option<u64> {
        let id = *self.forward.get(primary_external_id)?;
        self.forward.insert(alias.to_owned(), id);
        // Reverse map still points to the primary external ID.
        Some(id)
    }

    /// Look up the numeric ID for an external ID.
    pub fn get(&self, external_id: &str) -> Option<u64> {
        self.forward.get(external_id).copied()
    }

    /// Look up the primary external ID for a numeric ID.
    pub fn reverse_get(&self, numeric_id: u64) -> Option<&str> {
        self.reverse.get(&numeric_id).map(|s| s.as_str())
    }

    /// Returns true if the external ID is already mapped.
    pub fn contains(&self, external_id: &str) -> bool {
        self.forward.contains_key(external_id)
    }

    /// Number of unique numeric IDs (primary entries, not counting aliases).
    pub fn len(&self) -> usize {
        self.reverse.len()
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.reverse.is_empty()
    }

    /// The next numeric ID that will be assigned.
    pub fn next_id(&self) -> u64 {
        self.next_id
    }

    /// Retain only the nodes whose numeric IDs are in the given set.
    ///
    /// This is used after budget trimming to remove stale entries.
    pub fn retain_nodes(&mut self, keep_ids: &std::collections::HashSet<u64>) {
        self.forward.retain(|_, id| keep_ids.contains(id));
        self.reverse.retain(|id, _| keep_ids.contains(id));
    }

    /// Iterate over all (external_id, numeric_id) pairs (including aliases).
    pub fn iter_forward(&self) -> impl Iterator<Item = (&str, u64)> {
        self.forward.iter().map(|(k, &v)| (k.as_str(), v))
    }

    /// Iterate over all (numeric_id, primary_external_id) pairs.
    pub fn iter_reverse(&self) -> impl Iterator<Item = (u64, &str)> {
        self.reverse.iter().map(|(&k, v)| (k, v.as_str()))
    }
}

impl Default for IdMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_lookup() {
        let mut map = IdMap::new();
        let id1 = map.get_or_insert("VENDOR-001");
        let id2 = map.get_or_insert("VENDOR-002");
        let id1_again = map.get_or_insert("VENDOR-001");

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id1_again, 1); // idempotent
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn reverse_lookup() {
        let mut map = IdMap::new();
        map.get_or_insert("ACCOUNT-100");
        map.get_or_insert("ACCOUNT-200");

        assert_eq!(map.reverse_get(1), Some("ACCOUNT-100"));
        assert_eq!(map.reverse_get(2), Some("ACCOUNT-200"));
        assert_eq!(map.reverse_get(99), None);
    }

    #[test]
    fn alias_support() {
        let mut map = IdMap::new();
        map.get_or_insert("VENDOR-001");
        let alias_id = map.alias("V001", "VENDOR-001");

        assert_eq!(alias_id, Some(1));
        assert_eq!(map.get("V001"), Some(1));
        assert_eq!(map.get("VENDOR-001"), Some(1));
        // Reverse still points to primary
        assert_eq!(map.reverse_get(1), Some("VENDOR-001"));
        // Alias for non-existent primary returns None
        assert_eq!(map.alias("X", "DOES-NOT-EXIST"), None);
    }

    #[test]
    fn retain_nodes() {
        let mut map = IdMap::new();
        map.get_or_insert("A");
        map.get_or_insert("B");
        map.get_or_insert("C");
        map.alias("A2", "A");

        let keep: std::collections::HashSet<u64> = [1, 3].into_iter().collect();
        map.retain_nodes(&keep);

        assert_eq!(map.len(), 2); // A and C remain
        assert!(map.contains("A"));
        assert!(map.contains("A2")); // alias of A
        assert!(!map.contains("B"));
        assert!(map.contains("C"));
        assert_eq!(map.reverse_get(1), Some("A"));
        assert_eq!(map.reverse_get(2), None);
        assert_eq!(map.reverse_get(3), Some("C"));
    }
}
