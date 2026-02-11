//! Role-Based Access Control (RBAC) for the REST API.
//!
//! Defines roles, permissions, and authorization logic. Roles are hierarchical:
//! - **Admin**: full access to all operations
//! - **Operator**: can generate data and manage jobs, but cannot manage API keys
//! - **Viewer**: read-only access to jobs, config, and metrics

use serde::{Deserialize, Serialize};

// ===========================================================================
// Roles
// ===========================================================================

/// User roles for access control.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Full access to all operations including API key management.
    Admin,
    /// Can generate data, manage and view jobs, and view config/metrics.
    #[default]
    Operator,
    /// Read-only access: view jobs, config, and metrics.
    Viewer,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Operator => write!(f, "operator"),
            Role::Viewer => write!(f, "viewer"),
        }
    }
}

// ===========================================================================
// Permissions
// ===========================================================================

/// Fine-grained permissions that can be checked against a role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Start a data generation job.
    GenerateData,
    /// Create, cancel, or modify generation jobs.
    ManageJobs,
    /// View job status and history.
    ViewJobs,
    /// Create or update server/generation configuration.
    ManageConfig,
    /// View current configuration.
    ViewConfig,
    /// View server metrics and health data.
    ViewMetrics,
    /// Create, revoke, or rotate API keys.
    ManageApiKeys,
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::GenerateData => write!(f, "generate_data"),
            Permission::ManageJobs => write!(f, "manage_jobs"),
            Permission::ViewJobs => write!(f, "view_jobs"),
            Permission::ManageConfig => write!(f, "manage_config"),
            Permission::ViewConfig => write!(f, "view_config"),
            Permission::ViewMetrics => write!(f, "view_metrics"),
            Permission::ManageApiKeys => write!(f, "manage_api_keys"),
        }
    }
}

// ===========================================================================
// Role → Permission mapping
// ===========================================================================

/// Resolves whether a given role has a specific permission.
pub struct RolePermissions;

impl RolePermissions {
    /// Check if `role` is granted `permission`.
    ///
    /// Permission matrix:
    ///
    /// | Permission      | Admin | Operator | Viewer |
    /// |-----------------|-------|----------|--------|
    /// | GenerateData    |   Y   |    Y     |   N    |
    /// | ManageJobs      |   Y   |    Y     |   N    |
    /// | ViewJobs        |   Y   |    Y     |   Y    |
    /// | ManageConfig    |   Y   |    N     |   N    |
    /// | ViewConfig      |   Y   |    Y     |   Y    |
    /// | ViewMetrics     |   Y   |    Y     |   Y    |
    /// | ManageApiKeys   |   Y   |    N     |   N    |
    pub fn has_permission(role: &Role, permission: &Permission) -> bool {
        match role {
            Role::Admin => true,
            Role::Operator => matches!(
                permission,
                Permission::GenerateData
                    | Permission::ManageJobs
                    | Permission::ViewJobs
                    | Permission::ViewConfig
                    | Permission::ViewMetrics
            ),
            Role::Viewer => matches!(
                permission,
                Permission::ViewJobs | Permission::ViewConfig | Permission::ViewMetrics
            ),
        }
    }

    /// Return all permissions granted to a role.
    pub fn permissions_for(role: &Role) -> Vec<Permission> {
        let all = [
            Permission::GenerateData,
            Permission::ManageJobs,
            Permission::ViewJobs,
            Permission::ManageConfig,
            Permission::ViewConfig,
            Permission::ViewMetrics,
            Permission::ManageApiKeys,
        ];
        all.into_iter()
            .filter(|p| Self::has_permission(role, p))
            .collect()
    }
}

// ===========================================================================
// Configuration
// ===========================================================================

/// Configuration for the RBAC subsystem.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RbacConfig {
    /// Whether RBAC enforcement is enabled.
    /// When `false`, all authenticated requests are treated as Admin.
    #[serde(default)]
    pub enabled: bool,
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_has_all_permissions() {
        let all_permissions = [
            Permission::GenerateData,
            Permission::ManageJobs,
            Permission::ViewJobs,
            Permission::ManageConfig,
            Permission::ViewConfig,
            Permission::ViewMetrics,
            Permission::ManageApiKeys,
        ];
        for perm in &all_permissions {
            assert!(
                RolePermissions::has_permission(&Role::Admin, perm),
                "Admin should have permission: {}",
                perm
            );
        }
    }

    #[test]
    fn test_viewer_denied_generate() {
        assert!(!RolePermissions::has_permission(
            &Role::Viewer,
            &Permission::GenerateData
        ));
        assert!(!RolePermissions::has_permission(
            &Role::Viewer,
            &Permission::ManageJobs
        ));
        assert!(!RolePermissions::has_permission(
            &Role::Viewer,
            &Permission::ManageConfig
        ));
        assert!(!RolePermissions::has_permission(
            &Role::Viewer,
            &Permission::ManageApiKeys
        ));
    }

    #[test]
    fn test_viewer_allowed_read_only() {
        assert!(RolePermissions::has_permission(
            &Role::Viewer,
            &Permission::ViewJobs
        ));
        assert!(RolePermissions::has_permission(
            &Role::Viewer,
            &Permission::ViewConfig
        ));
        assert!(RolePermissions::has_permission(
            &Role::Viewer,
            &Permission::ViewMetrics
        ));
    }

    #[test]
    fn test_operator_permissions() {
        // Allowed
        assert!(RolePermissions::has_permission(
            &Role::Operator,
            &Permission::GenerateData
        ));
        assert!(RolePermissions::has_permission(
            &Role::Operator,
            &Permission::ManageJobs
        ));
        assert!(RolePermissions::has_permission(
            &Role::Operator,
            &Permission::ViewJobs
        ));
        assert!(RolePermissions::has_permission(
            &Role::Operator,
            &Permission::ViewConfig
        ));
        assert!(RolePermissions::has_permission(
            &Role::Operator,
            &Permission::ViewMetrics
        ));

        // Denied
        assert!(!RolePermissions::has_permission(
            &Role::Operator,
            &Permission::ManageConfig
        ));
        assert!(!RolePermissions::has_permission(
            &Role::Operator,
            &Permission::ManageApiKeys
        ));
    }

    #[test]
    fn test_default_role_is_operator() {
        let role = Role::default();
        assert_eq!(role, Role::Operator);
    }

    #[test]
    fn test_rbac_config_default_disabled() {
        let config = RbacConfig::default();
        assert!(!config.enabled);
    }

    #[test]
    fn test_role_serialization_roundtrip() {
        let role = Role::Admin;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"admin\"");
        let deserialized: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Role::Admin);
    }

    #[test]
    fn test_permissions_for_role() {
        let admin_perms = RolePermissions::permissions_for(&Role::Admin);
        assert_eq!(admin_perms.len(), 7);

        let operator_perms = RolePermissions::permissions_for(&Role::Operator);
        assert_eq!(operator_perms.len(), 5);

        let viewer_perms = RolePermissions::permissions_for(&Role::Viewer);
        assert_eq!(viewer_perms.len(), 3);
    }
}
