//! Resource pools for OCEL 2.0 event generation.
//!
//! A resource pool groups related resources (e.g. "AP Clerks") and provides
//! assignment strategies for selecting which resource performs an activity.

use serde::{Deserialize, Serialize};

/// A pool of resources with an assignment strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    /// Unique pool identifier.
    pub pool_id: String,
    /// Human-readable pool name.
    pub pool_name: String,
    /// Resources in this pool.
    pub resources: Vec<PoolResource>,
    /// Strategy used to pick the next resource.
    pub assignment_strategy: AssignmentStrategy,
    /// Internal round-robin counter (not serialized for cleanliness).
    #[serde(skip)]
    next_index: usize,
}

/// A single resource within a pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolResource {
    /// Unique resource identifier.
    pub resource_id: String,
    /// Human-readable name.
    pub name: String,
    /// Maximum number of concurrent tasks.
    pub max_concurrent: usize,
    /// Current workload as a fraction of capacity (0.0 .. 1.0+).
    pub current_workload: f64,
    /// Total number of tasks ever assigned.
    pub total_assigned: u64,
    /// Skills or capabilities this resource possesses.
    pub skills: Vec<String>,
}

/// Strategy for assigning work to pool resources.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum AssignmentStrategy {
    /// Cycle through resources in order.
    #[default]
    RoundRobin,
    /// Pick the resource with the lowest current workload.
    LeastBusy,
    /// Match required skills to resource skills.
    SkillBased,
}

impl ResourcePool {
    /// Create a new pool with `count` resources using the given naming prefix.
    ///
    /// Resources are named `"{prefix}-001"`, `"{prefix}-002"`, etc. and their
    /// human-readable names follow the same pattern as `"{pool_name} {i}"`.
    pub fn new(pool_id: &str, pool_name: &str, count: usize, prefix: &str) -> Self {
        let resources = (1..=count)
            .map(|i| PoolResource {
                resource_id: format!("{prefix}-{i:03}"),
                name: format!("{pool_name} {i}"),
                max_concurrent: 10,
                current_workload: 0.0,
                total_assigned: 0,
                skills: Vec::new(),
            })
            .collect();
        Self {
            pool_id: pool_id.into(),
            pool_name: pool_name.into(),
            resources,
            assignment_strategy: AssignmentStrategy::RoundRobin,
            next_index: 0,
        }
    }

    /// Assign the next available resource and return its ID.
    ///
    /// For `RoundRobin` (the default), this cycles through resources in order.
    /// Returns `None` if the pool is empty.
    pub fn assign(&mut self) -> Option<&str> {
        if self.resources.is_empty() {
            return None;
        }
        let idx = self.next_index % self.resources.len();
        self.next_index = idx + 1;
        let resource = &mut self.resources[idx];
        resource.total_assigned += 1;
        resource.current_workload += 1.0 / resource.max_concurrent as f64;
        Some(&self.resources[idx].resource_id)
    }

    /// Release a unit of workload from the named resource.
    pub fn release(&mut self, resource_id: &str) {
        if let Some(r) = self
            .resources
            .iter_mut()
            .find(|r| r.resource_id == resource_id)
        {
            r.current_workload = (r.current_workload - 1.0 / r.max_concurrent as f64).max(0.0);
        }
    }
}

/// Return the five default resource pools used by the OCPM generator.
pub fn default_resource_pools() -> Vec<ResourcePool> {
    vec![
        ResourcePool::new("pool-ap", "AP Clerk", 5, "ap"),
        ResourcePool::new("pool-ar", "AR Clerk", 3, "ar"),
        ResourcePool::new("pool-gl", "GL Accountant", 2, "gl"),
        ResourcePool::new("pool-approver", "Approver", 4, "approver"),
        ResourcePool::new("pool-warehouse", "Warehouse Worker", 6, "warehouse"),
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let pool = ResourcePool::new("pool-test", "Tester", 3, "tst");
        assert_eq!(pool.resources.len(), 3);
        assert_eq!(pool.resources[0].resource_id, "tst-001");
        assert_eq!(pool.resources[1].resource_id, "tst-002");
        assert_eq!(pool.resources[2].resource_id, "tst-003");
        assert_eq!(pool.assignment_strategy, AssignmentStrategy::RoundRobin);
    }

    #[test]
    fn test_round_robin_assignment() {
        let mut pool = ResourcePool::new("pool-rr", "Worker", 3, "w");

        let first = pool.assign().unwrap().to_string();
        let second = pool.assign().unwrap().to_string();
        let third = pool.assign().unwrap().to_string();
        let fourth = pool.assign().unwrap().to_string();

        assert_eq!(first, "w-001");
        assert_eq!(second, "w-002");
        assert_eq!(third, "w-003");
        // Wraps around
        assert_eq!(fourth, "w-001");

        // Check total_assigned
        assert_eq!(pool.resources[0].total_assigned, 2);
        assert_eq!(pool.resources[1].total_assigned, 1);
        assert_eq!(pool.resources[2].total_assigned, 1);
    }

    #[test]
    fn test_release_reduces_workload() {
        let mut pool = ResourcePool::new("pool-rel", "Worker", 2, "rel");

        pool.assign(); // rel-001
        let workload_after_assign = pool.resources[0].current_workload;
        assert!(workload_after_assign > 0.0);

        pool.release("rel-001");
        assert!(pool.resources[0].current_workload < workload_after_assign);
        assert!(pool.resources[0].current_workload >= 0.0);
    }

    #[test]
    fn test_default_pools_count() {
        let pools = default_resource_pools();
        assert_eq!(pools.len(), 5);

        let ids: Vec<&str> = pools.iter().map(|p| p.pool_id.as_str()).collect();
        assert!(ids.contains(&"pool-ap"));
        assert!(ids.contains(&"pool-ar"));
        assert!(ids.contains(&"pool-gl"));
        assert!(ids.contains(&"pool-approver"));
        assert!(ids.contains(&"pool-warehouse"));

        // Check specific counts
        let ap = pools.iter().find(|p| p.pool_id == "pool-ap").unwrap();
        assert_eq!(ap.resources.len(), 5);
        let warehouse = pools
            .iter()
            .find(|p| p.pool_id == "pool-warehouse")
            .unwrap();
        assert_eq!(warehouse.resources.len(), 6);
    }

    #[test]
    fn test_serde_roundtrip() {
        let pool = ResourcePool::new("pool-serde", "Clerk", 2, "clk");
        let json = serde_json::to_string(&pool).unwrap();
        let deserialized: ResourcePool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pool_id, "pool-serde");
        assert_eq!(deserialized.resources.len(), 2);
        assert_eq!(
            deserialized.assignment_strategy,
            AssignmentStrategy::RoundRobin
        );
    }
}
