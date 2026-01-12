//! Core traits for plugin dependency checking and introspection.
//!
//! This module provides the trait hierarchy that enables compile-time
//! and runtime validation of plugin dependencies.

use bevy::prelude::{App, Plugin};

/// Error returned when a required plugin is missing.
#[derive(Debug, Clone)]
pub struct MissingPluginError {
    /// Name of the plugin that requires the dependency
    pub required_by: &'static str,
    /// Name of the missing plugin
    pub missing: &'static str,
}

impl std::fmt::Display for MissingPluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Plugin '{}' requires '{}' to be added first. \
             Add '{}' before '{}' in your app.add_plugins() call.",
            self.required_by, self.missing, self.missing, self.required_by
        )
    }
}

impl std::error::Error for MissingPluginError {}

/// Marker trait for plugins created with `define_plugin!`.
///
/// This trait is automatically implemented by the macro and enables
/// compile-time dependency checking. If you try to depend on a plugin
/// that doesn't implement `PluginMarker`, compilation will fail.
///
/// The associated `Id` type is used for type-level identification.
pub trait PluginMarker: Plugin + 'static {
    /// Type-level identifier for this plugin.
    /// Typically the plugin struct itself.
    type Id;
}

/// Trait for tuples of plugin markers, enabling dependency verification.
///
/// This is implemented for tuples of increasing sizes (up to 12 elements)
/// to allow declaring multiple dependencies.
pub trait PluginSet {
    /// Verify that all plugins in this set are registered in the App.
    ///
    /// Returns `Ok(())` if all plugins are present, or `Err` with the
    /// first missing plugin.
    fn verify_registered(app: &App, required_by: &'static str) -> Result<(), MissingPluginError>;

    /// Get the type names of all plugins in this set for error messages.
    fn type_names() -> Vec<&'static str>;
}

/// Trait declaring plugin dependencies.
///
/// Automatically implemented by `define_plugin!` when `depends_on:` is used.
/// The `Required` associated type is a tuple of plugin marker types.
pub trait PluginDependencies: Plugin {
    /// Tuple of required plugin marker types.
    /// Empty tuple `()` means no dependencies.
    type Required: PluginSet;

    /// Verify all dependencies are satisfied.
    ///
    /// This is called at runtime during `build()` to ensure plugins
    /// were added in the correct order.
    fn verify_dependencies(app: &App) -> Result<(), MissingPluginError>
    where
        Self: Sized,
    {
        Self::Required::verify_registered(app, std::any::type_name::<Self>())
    }
}

// ============================================================================
// PluginSet implementations for tuples
// ============================================================================

/// Implementation for empty tuple - no dependencies
impl PluginSet for () {
    fn verify_registered(_app: &App, _required_by: &'static str) -> Result<(), MissingPluginError> {
        Ok(())
    }

    fn type_names() -> Vec<&'static str> {
        Vec::new()
    }
}

/// Implementation for single plugin dependency
impl<P1> PluginSet for (P1,)
where
    P1: PluginMarker,
{
    fn verify_registered(app: &App, required_by: &'static str) -> Result<(), MissingPluginError> {
        if !app.is_plugin_added::<P1>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P1>(),
            });
        }
        Ok(())
    }

    fn type_names() -> Vec<&'static str> {
        vec![std::any::type_name::<P1>()]
    }
}

/// Implementation for two plugin dependencies
impl<P1, P2> PluginSet for (P1, P2)
where
    P1: PluginMarker,
    P2: PluginMarker,
{
    fn verify_registered(app: &App, required_by: &'static str) -> Result<(), MissingPluginError> {
        if !app.is_plugin_added::<P1>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P1>(),
            });
        }
        if !app.is_plugin_added::<P2>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P2>(),
            });
        }
        Ok(())
    }

    fn type_names() -> Vec<&'static str> {
        vec![std::any::type_name::<P1>(), std::any::type_name::<P2>()]
    }
}

/// Implementation for three plugin dependencies
impl<P1, P2, P3> PluginSet for (P1, P2, P3)
where
    P1: PluginMarker,
    P2: PluginMarker,
    P3: PluginMarker,
{
    fn verify_registered(app: &App, required_by: &'static str) -> Result<(), MissingPluginError> {
        if !app.is_plugin_added::<P1>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P1>(),
            });
        }
        if !app.is_plugin_added::<P2>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P2>(),
            });
        }
        if !app.is_plugin_added::<P3>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P3>(),
            });
        }
        Ok(())
    }

    fn type_names() -> Vec<&'static str> {
        vec![
            std::any::type_name::<P1>(),
            std::any::type_name::<P2>(),
            std::any::type_name::<P3>(),
        ]
    }
}

/// Implementation for four plugin dependencies
impl<P1, P2, P3, P4> PluginSet for (P1, P2, P3, P4)
where
    P1: PluginMarker,
    P2: PluginMarker,
    P3: PluginMarker,
    P4: PluginMarker,
{
    fn verify_registered(app: &App, required_by: &'static str) -> Result<(), MissingPluginError> {
        if !app.is_plugin_added::<P1>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P1>(),
            });
        }
        if !app.is_plugin_added::<P2>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P2>(),
            });
        }
        if !app.is_plugin_added::<P3>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P3>(),
            });
        }
        if !app.is_plugin_added::<P4>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P4>(),
            });
        }
        Ok(())
    }

    fn type_names() -> Vec<&'static str> {
        vec![
            std::any::type_name::<P1>(),
            std::any::type_name::<P2>(),
            std::any::type_name::<P3>(),
            std::any::type_name::<P4>(),
        ]
    }
}

/// Implementation for five plugin dependencies
impl<P1, P2, P3, P4, P5> PluginSet for (P1, P2, P3, P4, P5)
where
    P1: PluginMarker,
    P2: PluginMarker,
    P3: PluginMarker,
    P4: PluginMarker,
    P5: PluginMarker,
{
    fn verify_registered(app: &App, required_by: &'static str) -> Result<(), MissingPluginError> {
        if !app.is_plugin_added::<P1>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P1>(),
            });
        }
        if !app.is_plugin_added::<P2>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P2>(),
            });
        }
        if !app.is_plugin_added::<P3>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P3>(),
            });
        }
        if !app.is_plugin_added::<P4>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P4>(),
            });
        }
        if !app.is_plugin_added::<P5>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P5>(),
            });
        }
        Ok(())
    }

    fn type_names() -> Vec<&'static str> {
        vec![
            std::any::type_name::<P1>(),
            std::any::type_name::<P2>(),
            std::any::type_name::<P3>(),
            std::any::type_name::<P4>(),
            std::any::type_name::<P5>(),
        ]
    }
}

/// Implementation for six plugin dependencies
impl<P1, P2, P3, P4, P5, P6> PluginSet for (P1, P2, P3, P4, P5, P6)
where
    P1: PluginMarker,
    P2: PluginMarker,
    P3: PluginMarker,
    P4: PluginMarker,
    P5: PluginMarker,
    P6: PluginMarker,
{
    fn verify_registered(app: &App, required_by: &'static str) -> Result<(), MissingPluginError> {
        if !app.is_plugin_added::<P1>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P1>(),
            });
        }
        if !app.is_plugin_added::<P2>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P2>(),
            });
        }
        if !app.is_plugin_added::<P3>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P3>(),
            });
        }
        if !app.is_plugin_added::<P4>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P4>(),
            });
        }
        if !app.is_plugin_added::<P5>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P5>(),
            });
        }
        if !app.is_plugin_added::<P6>() {
            return Err(MissingPluginError {
                required_by,
                missing: std::any::type_name::<P6>(),
            });
        }
        Ok(())
    }

    fn type_names() -> Vec<&'static str> {
        vec![
            std::any::type_name::<P1>(),
            std::any::type_name::<P2>(),
            std::any::type_name::<P3>(),
            std::any::type_name::<P4>(),
            std::any::type_name::<P5>(),
            std::any::type_name::<P6>(),
        ]
    }
}

// Additional implementations can be added for tuples up to 12 elements
// following the same pattern. For most use cases, 6 dependencies is sufficient.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_plugin_error_display() {
        let err = MissingPluginError {
            required_by: "GamePlugin",
            missing: "PhysicsPlugin",
        };
        let msg = format!("{}", err);
        assert!(msg.contains("GamePlugin"));
        assert!(msg.contains("PhysicsPlugin"));
        assert!(msg.contains("add_plugins()"));
    }

    #[test]
    fn test_empty_plugin_set() {
        // Empty tuple should always succeed
        let app = App::new();
        assert!(<()>::verify_registered(&app, "TestPlugin").is_ok());
        assert!(<()>::type_names().is_empty());
    }
}
