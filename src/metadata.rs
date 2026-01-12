//! Plugin metadata structures for introspection.
//!
//! This module provides types for storing and querying plugin metadata
//! at runtime. Enabled by the `introspection` feature.

use bevy::prelude::Plugin;
use std::any::TypeId;

/// Information about a registered type (resource, message, state, etc.)
#[derive(Debug, Clone, Copy)]
pub struct TypeInfo {
    /// Human-readable name of the type
    pub name: &'static str,
    /// Function to get the TypeId (deferred to avoid const evaluation issues)
    pub type_id_fn: fn() -> TypeId,
}

impl TypeInfo {
    /// Create a new TypeInfo for a type
    pub const fn new<T: 'static>(name: &'static str) -> Self {
        Self {
            name,
            type_id_fn: std::any::TypeId::of::<T>,
        }
    }

    /// Get the TypeId of this type
    pub fn type_id(&self) -> TypeId {
        (self.type_id_fn)()
    }
}

impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.type_id() == other.type_id()
    }
}

impl Eq for TypeInfo {}

/// Metadata about systems registered in different schedules
#[derive(Debug, Clone, Default)]
pub struct PluginSystems {
    /// Names of startup systems
    pub startup: &'static [&'static str],
    /// Names of update systems
    pub update: &'static [&'static str],
    /// Names of fixed update systems
    pub fixed_update: &'static [&'static str],
    /// Number of on_enter state systems
    pub on_enter_count: usize,
    /// Number of on_exit state systems
    pub on_exit_count: usize,
}

/// Static metadata about a plugin's registrations.
///
/// This struct contains all the information about what a plugin registers,
/// stored as static data with zero runtime allocation.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin name (usually the struct name)
    pub name: &'static str,
    /// Version string from meta block (if provided)
    pub version: Option<&'static str>,
    /// Description from meta block (if provided)
    pub description: Option<&'static str>,
    /// Resources registered with init_resource
    pub resources: &'static [TypeInfo],
    /// Messages registered with add_message
    pub messages: &'static [TypeInfo],
    /// States registered with init_state
    pub states: &'static [TypeInfo],
    /// Sub-states registered with add_sub_state
    pub sub_states: &'static [TypeInfo],
    /// Types registered for reflection
    pub reflected_types: &'static [TypeInfo],
    /// Sub-plugins added
    pub sub_plugins: &'static [&'static str],
    /// Plugin dependencies
    pub dependencies: &'static [&'static str],
    /// System information
    pub systems: PluginSystems,
}

impl PluginMetadata {
    /// Create an empty metadata instance (for plugins with no registrations)
    pub const fn empty(name: &'static str) -> Self {
        Self {
            name,
            version: None,
            description: None,
            resources: &[],
            messages: &[],
            states: &[],
            sub_states: &[],
            reflected_types: &[],
            sub_plugins: &[],
            dependencies: &[],
            systems: PluginSystems {
                startup: &[],
                update: &[],
                fixed_update: &[],
                on_enter_count: 0,
                on_exit_count: 0,
            },
        }
    }

    /// Check if this plugin registers a specific resource type
    pub fn has_resource<R: 'static>(&self) -> bool {
        let target_id = TypeId::of::<R>();
        self.resources
            .iter()
            .any(|info| info.type_id() == target_id)
    }

    /// Check if this plugin registers a specific message type
    pub fn has_message<M: 'static>(&self) -> bool {
        let target_id = TypeId::of::<M>();
        self.messages.iter().any(|info| info.type_id() == target_id)
    }

    /// Check if this plugin registers a specific state type
    pub fn has_state<S: 'static>(&self) -> bool {
        let target_id = TypeId::of::<S>();
        self.states.iter().any(|info| info.type_id() == target_id)
    }

    /// Get the total number of systems registered by this plugin
    pub fn total_systems(&self) -> usize {
        self.systems.startup.len()
            + self.systems.update.len()
            + self.systems.fixed_update.len()
            + self.systems.on_enter_count
            + self.systems.on_exit_count
    }

    /// Check if this plugin depends on another plugin by name
    pub fn depends_on(&self, plugin_name: &str) -> bool {
        self.dependencies.contains(&plugin_name)
    }
}

/// Trait for plugins that can provide static metadata.
///
/// This trait is automatically implemented by `define_plugin!` when the
/// `introspection` feature is enabled. It provides zero-cost access to
/// plugin metadata.
pub trait PluginInfo: Plugin {
    /// The plugin's name
    const NAME: &'static str;

    /// The plugin's version (from meta block)
    const VERSION: Option<&'static str> = None;

    /// Get the static metadata for this plugin
    fn metadata() -> &'static PluginMetadata;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_info_equality() {
        let info1 = TypeInfo::new::<String>("String");
        let info2 = TypeInfo::new::<String>("String");
        let info3 = TypeInfo::new::<i32>("i32");

        assert_eq!(info1, info2);
        assert_ne!(info1, info3);
    }

    #[test]
    fn test_plugin_metadata_queries() {
        static TEST_RESOURCES: [TypeInfo; 2] = [
            TypeInfo::new::<String>("String"),
            TypeInfo::new::<i32>("i32"),
        ];

        static TEST_DEPS: [&str; 1] = ["OtherPlugin"];

        let metadata = PluginMetadata {
            name: "TestPlugin",
            version: Some("1.0.0"),
            description: Some("A test plugin"),
            resources: &TEST_RESOURCES,
            messages: &[],
            states: &[],
            sub_states: &[],
            reflected_types: &[],
            sub_plugins: &[],
            dependencies: &TEST_DEPS,
            systems: PluginSystems::default(),
        };

        assert!(metadata.has_resource::<String>());
        assert!(metadata.has_resource::<i32>());
        assert!(!metadata.has_resource::<f32>());

        assert!(metadata.depends_on("OtherPlugin"));
        assert!(!metadata.depends_on("NonExistent"));

        assert_eq!(metadata.total_systems(), 0);
    }

    #[test]
    fn test_empty_metadata() {
        let metadata = PluginMetadata::empty("EmptyPlugin");

        assert_eq!(metadata.name, "EmptyPlugin");
        assert!(metadata.version.is_none());
        assert!(metadata.resources.is_empty());
        assert_eq!(metadata.total_systems(), 0);
    }
}
