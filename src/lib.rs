//! # bevy-plugin-builder
//!
//! Declarative plugin system for Bevy
//!
//! This crate provides the `define_plugin!` macro that eliminates boilerplate
//! from Bevy plugin registration. Instead of manually implementing `Plugin`,
//! you declare your plugin's requirements and the macro handles all registration.
//!
//! ## Features
//!
//! - Eliminates repetitive `impl Plugin for` blocks
//! - Compile-time validation of all registrations
//! - Supports all Bevy registration patterns
//! - Plugin dependency tracking
//! - Optional introspection (query plugin metadata at runtime)
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! bevy-plugin-builder = "0.3"
//! bevy = "0.18.0-rc.2"
//! ```
//!
//! Then transform your plugins:
//!
//! ```rust
//! use bevy_plugin_builder::define_plugin;
//! use bevy::prelude::*;
//!
//! // Define your game types
//! #[derive(Resource, Default)]
//! struct GameSettings;
//!
//! #[derive(Resource, Default)]
//! struct PlayerStats;
//!
//! #[derive(Message)]
//! struct PlayerDied;
//!
//! #[derive(Message)]
//! struct ScoreChanged;
//!
//! #[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
//! enum GameState {
//!     #[default]
//!     Menu,
//!     Playing,
//! }
//!
//! // Define your systems
//! fn setup_game() {}
//! fn handle_input() {}
//! fn update_physics() {}
//! fn render_game() {}
//!
//! define_plugin!(MyGamePlugin {
//!     init_resource: [GameSettings, PlayerStats],
//!     add_message: [PlayerDied, ScoreChanged],
//!     add_systems_startup: [setup_game],
//!     add_systems_update: [
//!         (handle_input, update_physics, render_game)
//!             .chain()
//!             .run_if(in_state(GameState::Playing))
//!     ]
//! });
//! ```
//!
//! ## Supported Configuration Options
//!
//! All options use Bevy-aligned naming to match Bevy's App builder API.
//!
//! ### Registration Options
//!
//! - **`init_resource: [Type]`** - Initialize resources with `init_resource`
//! - **`insert_resource: [Instance]`** - Insert resource instances directly
//! - **`add_message: [Msg]`** - Register messages with `add_message`
//! - **`add_plugins: [Plugin]`** - Add sub-plugins with `add_plugins`
//! - **`init_state: [State]`** - Initialize states with `init_state`
//! - **`add_sub_state: [SubState]`** - Add sub-states with `add_sub_state`
//! - **`register_type: [Type]`** - Register types for reflection
//!
//! ### System Scheduling Options
//!
//! - **`add_systems_startup: [sys]`** - Add startup systems
//! - **`add_systems_update: [sys]`** - Add update systems
//! - **`add_systems_fixed_update: [sys]`** - Add fixed update systems
//! - **`add_systems_on_enter: { State => [sys] }`** - State enter systems
//! - **`add_systems_on_exit: { State => [sys] }`** - State exit systems
//!
//! ### Custom Logic Options
//!
//! - **`custom_build: |app| { ... }`** - Custom build logic
//! - **`custom_finish: |app| { ... }`** - Custom finish logic
//!
//! ### Plugin Features
//!
//! - **`depends_on: [Plugin1, Plugin2]`** - Declare plugin dependencies (panics if missing)
//! - **`meta: { version: "1.0", description: "..." }`** - Plugin metadata (requires `introspection` feature)
//!
//! ## Advanced Example
//!
//! ```rust
//! use bevy_plugin_builder::define_plugin;
//! use bevy::prelude::*;
//!
//! #[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
//! enum GameState {
//!     #[default]
//!     Menu,
//!     Playing,
//!     Paused,
//! }
//!
//! #[derive(Resource, Default, Reflect)]
//! #[reflect(Resource)]
//! struct GameSettings { volume: f32 }
//!
//! #[derive(Message)]
//! struct GameStarted;
//!
//! define_plugin!(ComplexGamePlugin {
//!     // Type registration
//!     init_resource: [GameSettings],
//!     add_message: [GameStarted],
//!     init_state: [GameState],
//!     register_type: [GameSettings],
//!
//!     // System scheduling
//!     add_systems_startup: [initialize_audio, load_assets],
//!
//!     add_systems_update: [
//!         handle_menu_input.run_if(in_state(GameState::Menu)),
//!         (update_game, process_events).chain().run_if(in_state(GameState::Playing))
//!     ],
//!
//!     add_systems_fixed_update: [
//!         physics_simulation.run_if(in_state(GameState::Playing))
//!     ],
//!
//!     // State transitions
//!     add_systems_on_enter: {
//!         GameState::Playing => [setup_level, spawn_player],
//!         GameState::Paused => [show_pause_menu]
//!     },
//!
//!     add_systems_on_exit: {
//!         GameState::Playing => [cleanup_level]
//!     },
//!
//!     // Custom logic
//!     custom_build: |app: &mut App| {
//!         #[cfg(debug_assertions)]
//!         app.add_plugins(bevy::diagnostic::DiagnosticsPlugin::default());
//!     }
//! });
//!
//! fn initialize_audio() { /* ... */ }
//! fn load_assets() { /* ... */ }
//! fn handle_menu_input() { /* ... */ }
//! fn update_game() { /* ... */ }
//! fn process_events() { /* ... */ }
//! fn physics_simulation() { /* ... */ }
//! fn setup_level() { /* ... */ }
//! fn spawn_player() { /* ... */ }
//! fn show_pause_menu() { /* ... */ }
//! fn cleanup_level() { /* ... */ }
//! ```
//!
//! ## Cargo Features
//!
//! - **`introspection`** - Enables runtime metadata querying via `PluginInfo` trait and `PluginRegistry`
//! - **`testing`** - Enables automatic test generation with `generate_tests:` syntax
//! - **`full`** - Enables all features
//!
//! ### Introspection Example
//!
//! With the `introspection` feature enabled:
//!
//! ```rust,ignore
//! use bevy_plugin_builder::{define_plugin, PluginInfo, PluginRegistry};
//!
//! define_plugin!(MyPlugin {
//!     meta: {
//!         version: "1.0.0",
//!         description: "My plugin"
//!     },
//!     init_resource: [MyResource]
//! });
//!
//! // Query plugin metadata
//! assert_eq!(MyPlugin::NAME, "MyPlugin");
//! assert_eq!(MyPlugin::VERSION, Some("1.0.0"));
//!
//! let metadata = MyPlugin::metadata();
//! assert!(metadata.has_resource::<MyResource>());
//!
//! // Use the registry for multi-plugin queries
//! let mut registry = PluginRegistry::new();
//! registry.register::<MyPlugin>();
//! let plugins = registry.plugins_with_resource::<MyResource>();
//! ```
//!
//! ## Migration Guide
//!
//! Converting existing plugins is straightforward:
//!
//! 1. **Replace `impl Plugin`** with `define_plugin!`
//! 2. **Group registrations by type** (resources, messages, systems)
//! 3. **Use declarative syntax** instead of method chains
//!
//! See the [examples](https://github.com/noahsabaj/bevy-plugin-builder/tree/main/examples) for complete migration examples.
//!

// Private implementation modules
mod macros;
mod traits;

// Introspection modules (feature-gated)
#[cfg(feature = "introspection")]
mod metadata;
#[cfg(feature = "introspection")]
mod registry;

// Re-export commonly used Bevy types for convenience
pub use bevy::prelude::{App, FixedUpdate, OnEnter, OnExit, Plugin, Startup, Update};

// Re-export traits for plugin dependency checking
pub use traits::{MissingPluginError, PluginDependencies, PluginMarker, PluginSet};

// Re-export introspection types (feature-gated)
#[cfg(feature = "introspection")]
pub use metadata::{PluginInfo, PluginMetadata, PluginSystems, TypeInfo};
#[cfg(feature = "introspection")]
pub use registry::PluginRegistry;

// The macro is automatically available via #[macro_export] in macros.rs
