//! Resource model for OCPM.
//!
//! Resources represent users or systems that perform activities in the process.

use serde::{Deserialize, Serialize};

/// Resource (user or system) that performs activities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Unique resource identifier
    pub resource_id: String,
    /// Human-readable name
    pub name: String,
    /// Type of resource
    pub resource_type: ResourceType,
    /// Department (for users)
    pub department: Option<String>,
    /// Role (for users)
    pub role: Option<String>,
    /// Is the resource currently active
    pub is_active: bool,
    /// Activity types this resource can perform
    pub capabilities: Vec<String>,
    /// Cost per hour (for workload analysis)
    pub cost_per_hour: Option<f64>,
}

impl Resource {
    /// Create a new user resource.
    pub fn user(id: &str, name: &str) -> Self {
        Self {
            resource_id: id.into(),
            name: name.into(),
            resource_type: ResourceType::User,
            department: None,
            role: None,
            is_active: true,
            capabilities: Vec::new(),
            cost_per_hour: None,
        }
    }

    /// Create a new system resource.
    pub fn system(id: &str, name: &str) -> Self {
        Self {
            resource_id: id.into(),
            name: name.into(),
            resource_type: ResourceType::System,
            department: None,
            role: None,
            is_active: true,
            capabilities: Vec::new(),
            cost_per_hour: None,
        }
    }

    /// Set the department.
    pub fn with_department(mut self, department: &str) -> Self {
        self.department = Some(department.into());
        self
    }

    /// Set the role.
    pub fn with_role(mut self, role: &str) -> Self {
        self.role = Some(role.into());
        self
    }

    /// Add capabilities.
    pub fn with_capabilities(mut self, capabilities: Vec<&str>) -> Self {
        self.capabilities = capabilities.into_iter().map(String::from).collect();
        self
    }

    /// Set cost per hour.
    pub fn with_cost(mut self, cost_per_hour: f64) -> Self {
        self.cost_per_hour = Some(cost_per_hour);
        self
    }

    /// Check if this resource can perform an activity.
    pub fn can_perform(&self, activity_id: &str) -> bool {
        self.capabilities.is_empty() || self.capabilities.iter().any(|c| c == activity_id)
    }

    /// Create a standard ERP system resource.
    pub fn erp_system() -> Self {
        Self::system("SYS_ERP", "ERP System").with_capabilities(vec![
            "post_gr",
            "post_invoice",
            "execute_payment",
            "check_credit",
            "post_customer_invoice",
            "receive_payment",
        ])
    }

    /// Create a standard workflow system resource.
    pub fn workflow_system() -> Self {
        Self::system("SYS_WF", "Workflow System")
            .with_capabilities(vec!["release_po", "release_so"])
    }
}

/// Type of resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    /// Human user
    #[default]
    User,
    /// Automated system
    System,
    /// External service/API
    ExternalService,
    /// Bot/RPA
    Bot,
}

impl ResourceType {
    /// Check if this is a human resource.
    pub fn is_human(&self) -> bool {
        matches!(self, Self::User)
    }

    /// Check if this is an automated resource.
    pub fn is_automated(&self) -> bool {
        !self.is_human()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_user_resource() {
        let user = Resource::user("USR001", "John Doe")
            .with_department("Finance")
            .with_role("AP Clerk");

        assert_eq!(user.resource_type, ResourceType::User);
        assert_eq!(user.department, Some("Finance".into()));
    }

    #[test]
    fn test_system_resource() {
        let system = Resource::erp_system();

        assert_eq!(system.resource_type, ResourceType::System);
        assert!(system.can_perform("post_invoice"));
        assert!(!system.can_perform("create_po"));
    }
}
