//! Permission domain
//!
//! Represents all the shared types for permissions, with helper functions for using them.
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Represents all available permissions in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // Super admin permission (wildcard)
    All,

    // Admin permissions
    AdminRead,
    AdminWrite,
    AdminDelete,
    AdminUsers,
    AdminRoles,
    AdminLogs,
    AdminDatabase,
    AdminHealth,
    AdminMetrics,

    // User permissions
    UserRead,
    UserWrite,
    UserDelete,
    UserProfile,
    UserCreate,

    // System permissions
    SystemHealth,
    SystemMetrics,
    SystemLogs,
    SystemDatabase,

    // Custom permissions (for dynamic roles)
    Custom(String),
}

impl Permission {
    /// Convert permission to string representation
    pub fn to_string(&self) -> String {
        match self {
            Permission::All => "*".to_string(),
            Permission::AdminRead => "admin:read".to_string(),
            Permission::AdminWrite => "admin:write".to_string(),
            Permission::AdminDelete => "admin:delete".to_string(),
            Permission::AdminUsers => "admin:users".to_string(),
            Permission::AdminRoles => "admin:roles".to_string(),
            Permission::AdminLogs => "admin:logs".to_string(),
            Permission::AdminDatabase => "admin:database".to_string(),
            Permission::AdminHealth => "admin:health".to_string(),
            Permission::AdminMetrics => "admin:metrics".to_string(),
            Permission::UserRead => "user:read".to_string(),
            Permission::UserWrite => "user:write".to_string(),
            Permission::UserDelete => "user:delete".to_string(),
            Permission::UserProfile => "user:profile".to_string(),
            Permission::UserCreate => "user:create".to_string(),
            Permission::SystemHealth => "system:health".to_string(),
            Permission::SystemMetrics => "system:metrics".to_string(),
            Permission::SystemLogs => "system:logs".to_string(),
            Permission::SystemDatabase => "system:database".to_string(),
            Permission::Custom(s) => s.clone(),
        }
    }

    /// Create permission from string
    pub fn from_string(s: &str) -> Self {
        match s {
            "*" => Permission::All,
            "admin:read" => Permission::AdminRead,
            "admin:write" => Permission::AdminWrite,
            "admin:delete" => Permission::AdminDelete,
            "admin:users" => Permission::AdminUsers,
            "admin:roles" => Permission::AdminRoles,
            "admin:logs" => Permission::AdminLogs,
            "admin:database" => Permission::AdminDatabase,
            "admin:health" => Permission::AdminHealth,
            "admin:metrics" => Permission::AdminMetrics,
            "user:read" => Permission::UserRead,
            "user:write" => Permission::UserWrite,
            "user:delete" => Permission::UserDelete,
            "user:profile" => Permission::UserProfile,
            "user:create" => Permission::UserCreate,
            "system:health" => Permission::SystemHealth,
            "system:metrics" => Permission::SystemMetrics,
            "system:logs" => Permission::SystemLogs,
            "system:database" => Permission::SystemDatabase,
            _ => Permission::Custom(s.to_string()),
        }
    }

    /// Get permission category
    pub fn category(&self) -> &'static str {
        match self {
            Permission::All => "super",
            Permission::AdminRead
            | Permission::AdminWrite
            | Permission::AdminDelete
            | Permission::AdminUsers
            | Permission::AdminRoles
            | Permission::AdminLogs
            | Permission::AdminDatabase
            | Permission::AdminHealth
            | Permission::AdminMetrics => "admin",
            Permission::UserRead
            | Permission::UserWrite
            | Permission::UserDelete
            | Permission::UserProfile
            | Permission::UserCreate => "user",
            Permission::SystemHealth
            | Permission::SystemMetrics
            | Permission::SystemLogs
            | Permission::SystemDatabase => "system",
            Permission::Custom(_) => "custom",
        }
    }

    /// Get permission description
    pub fn description(&self) -> &'static str {
        match self {
            Permission::All => "Full system access",
            Permission::AdminRead => "Read admin data",
            Permission::AdminWrite => "Write admin data",
            Permission::AdminDelete => "Delete admin data",
            Permission::AdminUsers => "Manage users",
            Permission::AdminRoles => "Manage roles",
            Permission::AdminLogs => "View system logs",
            Permission::AdminDatabase => "Access database",
            Permission::AdminHealth => "View system health",
            Permission::AdminMetrics => "View system metrics",
            Permission::UserRead => "Read user data",
            Permission::UserWrite => "Write user data",
            Permission::UserDelete => "Delete user data",
            Permission::UserProfile => "Manage user profile",
            Permission::UserCreate => "Create users",
            Permission::SystemHealth => "View system health",
            Permission::SystemMetrics => "View system metrics",
            Permission::SystemLogs => "View system logs",
            Permission::SystemDatabase => "Access system database",
            Permission::Custom(_) => "Custom permission",
        }
    }

    /// Check if this permission includes another permission
    pub fn includes(&self, other: &Permission) -> bool {
        match self {
            Permission::All => true,
            _ => self == other,
        }
    }
}

/// Collection of permissions with helper methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    /// Create a new empty permission set
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }

    /// Create from vector of permissions
    pub fn from_vec(permissions: Vec<Permission>) -> Self {
        Self {
            permissions: permissions.into_iter().collect(),
        }
    }

    /// Create from vector of strings
    pub fn from_strings(permissions: Vec<String>) -> Self {
        Self {
            permissions: permissions
                .into_iter()
                .map(|s| Permission::from_string(&s))
                .collect(),
        }
    }

    /// Add a permission
    pub fn add(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Remove a permission
    pub fn remove(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    /// Check if set contains a permission
    pub fn contains(&self, permission: &Permission) -> bool {
        self.permissions.contains(&Permission::All) || self.permissions.contains(permission)
    }

    /// Check if set contains any of the given permissions
    pub fn contains_any(&self, permissions: &[Permission]) -> bool {
        permissions.iter().any(|p| self.contains(p))
    }

    /// Check if set contains all of the given permissions
    pub fn contains_all(&self, permissions: &[Permission]) -> bool {
        permissions.iter().all(|p| self.contains(p))
    }

    /// Get all permissions as vector
    pub fn to_vec(&self) -> Vec<Permission> {
        self.permissions.iter().cloned().collect()
    }

    /// Get all permissions as strings
    pub fn to_strings(&self) -> Vec<String> {
        self.permissions.iter().map(|p| p.to_string()).collect()
    }

    /// Merge with another permission set
    pub fn merge(&mut self, other: &PermissionSet) {
        for permission in &other.permissions {
            self.permissions.insert(permission.clone());
        }
    }
}

impl Default for PermissionSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined permission sets for common roles
pub struct DefaultPermissions;

impl DefaultPermissions {
    /// Super admin permissions (all permissions)
    pub fn super_admin() -> PermissionSet {
        PermissionSet::from_vec(vec![Permission::All])
    }

    /// Admin permissions (most admin functions)
    pub fn admin() -> PermissionSet {
        PermissionSet::from_vec(vec![
            Permission::AdminRead,
            Permission::AdminWrite,
            Permission::AdminUsers,
            Permission::AdminRoles,
            Permission::AdminLogs,
            Permission::AdminDatabase,
            Permission::AdminHealth,
            Permission::AdminMetrics,
            Permission::UserRead,
            Permission::UserWrite,
            Permission::UserDelete,
            Permission::UserCreate,
        ])
    }

    /// User permissions (basic user functions)
    pub fn user() -> PermissionSet {
        PermissionSet::from_vec(vec![Permission::UserProfile, Permission::UserRead])
    }

    /// Read-only admin permissions
    pub fn admin_readonly() -> PermissionSet {
        PermissionSet::from_vec(vec![
            Permission::AdminRead,
            Permission::AdminLogs,
            Permission::AdminHealth,
            Permission::AdminMetrics,
            Permission::UserRead,
        ])
    }
}
