//! Plugin registry for runtime introspection.
//!
//! This module provides the `PluginRegistry` resource that tracks all
//! registered plugins and allows querying their metadata at runtime.
//! Enabled by the `introspection` feature.

use crate::metadata::{PluginInfo, PluginMetadata};
use bevy::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;

/// A registry of all plugins registered with `define_plugin!`.
///
/// This resource is automatically initialized when the first plugin
/// with introspection enabled is added. It allows querying what
/// plugins are loaded and what they registered.
///
/// ## Example
///
/// ```ignore
/// fn debug_plugins(registry: Res<PluginRegistry>) {
///     for metadata in registry.list_all() {
///         println!("Plugin: {} v{:?}", metadata.name, metadata.version);
///         println!("  Resources: {}", metadata.resources.len());
///         println!("  Systems: {}", metadata.total_systems());
///     }
/// }
/// ```
#[derive(Resource, Default)]
pub struct PluginRegistry {
    /// Map from plugin TypeId to its metadata
    plugins: HashMap<TypeId, &'static PluginMetadata>,
    /// Order in which plugins were registered
    load_order: Vec<TypeId>,
}

impl PluginRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a plugin's metadata
    ///
    /// Called automatically by the macro-generated plugin code.
    pub fn register<P: PluginInfo + 'static>(&mut self) {
        let type_id = TypeId::of::<P>();
        if let std::collections::hash_map::Entry::Vacant(e) = self.plugins.entry(type_id) {
            e.insert(P::metadata());
            self.load_order.push(type_id);
        }
    }

    /// Get metadata for a specific plugin type
    ///
    /// Returns `None` if the plugin wasn't registered with introspection.
    pub fn get<P: PluginInfo + 'static>(&self) -> Option<&'static PluginMetadata> {
        self.plugins.get(&TypeId::of::<P>()).copied()
    }

    /// Check if a plugin type is registered
    pub fn is_registered<P: PluginInfo + 'static>(&self) -> bool {
        self.plugins.contains_key(&TypeId::of::<P>())
    }

    /// Get the number of registered plugins
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// List all registered plugins in registration order
    pub fn list_all(&self) -> impl Iterator<Item = &'static PluginMetadata> + '_ {
        self.load_order
            .iter()
            .filter_map(|id| self.plugins.get(id).copied())
    }

    /// Find plugins that registered a specific resource type
    pub fn plugins_with_resource<R: 'static>(&self) -> Vec<&'static str> {
        self.plugins
            .values()
            .filter(|meta| meta.has_resource::<R>())
            .map(|meta| meta.name)
            .collect()
    }

    /// Find plugins that registered a specific message type
    pub fn plugins_with_message<M: 'static>(&self) -> Vec<&'static str> {
        self.plugins
            .values()
            .filter(|meta| meta.has_message::<M>())
            .map(|meta| meta.name)
            .collect()
    }

    /// Find plugins that registered a specific state type
    pub fn plugins_with_state<S: 'static>(&self) -> Vec<&'static str> {
        self.plugins
            .values()
            .filter(|meta| meta.has_state::<S>())
            .map(|meta| meta.name)
            .collect()
    }

    /// Get the total number of resources registered across all plugins
    pub fn total_resources(&self) -> usize {
        self.plugins.values().map(|meta| meta.resources.len()).sum()
    }

    /// Get the total number of systems registered across all plugins
    pub fn total_systems(&self) -> usize {
        self.plugins.values().map(|meta| meta.total_systems()).sum()
    }

    /// Find a plugin by name
    pub fn find_by_name(&self, name: &str) -> Option<&'static PluginMetadata> {
        self.plugins
            .values()
            .find(|meta| meta.name == name)
            .copied()
    }

    /// Get all plugin names
    pub fn plugin_names(&self) -> Vec<&'static str> {
        self.load_order
            .iter()
            .filter_map(|id| self.plugins.get(id))
            .map(|meta| meta.name)
            .collect()
    }
}

impl std::fmt::Debug for PluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginRegistry")
            .field("plugin_count", &self.plugins.len())
            .field("plugins", &self.plugin_names())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{PluginSystems, TypeInfo};

    // Mock plugin for testing
    struct MockPlugin;

    impl Plugin for MockPlugin {
        fn build(&self, _app: &mut App) {}
    }

    static MOCK_RESOURCES: [TypeInfo; 1] = [TypeInfo::new::<String>("String")];

    static MOCK_METADATA: PluginMetadata = PluginMetadata {
        name: "MockPlugin",
        version: Some("1.0.0"),
        description: None,
        resources: &MOCK_RESOURCES,
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
    };

    impl PluginInfo for MockPlugin {
        const NAME: &'static str = "MockPlugin";
        const VERSION: Option<&'static str> = Some("1.0.0");

        fn metadata() -> &'static PluginMetadata {
            &MOCK_METADATA
        }
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = PluginRegistry::new();
        assert!(registry.is_empty());

        registry.register::<MockPlugin>();

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.is_registered::<MockPlugin>());

        let metadata = registry.get::<MockPlugin>().unwrap();
        assert_eq!(metadata.name, "MockPlugin");
        assert_eq!(metadata.version, Some("1.0.0"));
    }

    #[test]
    fn test_registry_list_all() {
        let mut registry = PluginRegistry::new();
        registry.register::<MockPlugin>();

        let plugins: Vec<_> = registry.list_all().collect();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "MockPlugin");
    }

    #[test]
    fn test_registry_plugins_with_resource() {
        let mut registry = PluginRegistry::new();
        registry.register::<MockPlugin>();

        let plugins = registry.plugins_with_resource::<String>();
        assert_eq!(plugins, vec!["MockPlugin"]);

        let empty = registry.plugins_with_resource::<i32>();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_registry_find_by_name() {
        let mut registry = PluginRegistry::new();
        registry.register::<MockPlugin>();

        let found = registry.find_by_name("MockPlugin");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "MockPlugin");

        let not_found = registry.find_by_name("NonExistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_registry_duplicate_registration() {
        let mut registry = PluginRegistry::new();
        registry.register::<MockPlugin>();
        registry.register::<MockPlugin>(); // Duplicate

        // Should still only have one entry
        assert_eq!(registry.len(), 1);
    }
}
