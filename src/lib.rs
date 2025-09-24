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
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! bevy-plugin-builder = "0.1"
//! bevy = "0.16"
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
//! #[derive(Event)]
//! struct PlayerDied;
//!
//! #[derive(Event)]
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
//!     resources: [GameSettings, PlayerStats],
//!     events: [PlayerDied, ScoreChanged],
//!     startup: [setup_game],
//!     update: [
//!         (handle_input, update_physics, render_game)
//!             .chain()
//!             .run_if(in_state(GameState::Playing))
//!     ]
//! });
//! ```
//!
//! ## Supported Configuration Options
//!
//! - **`resources: [Type1, Type2]`** - Initialize resources with `init_resource`
//! - **`events: [Event1, Event2]`** - Register events with `add_event`
//! - **`plugins: [Plugin1, Plugin2]`** - Add sub-plugins with `add_plugins`
//! - **`states: [State1]`** - Initialize states with `init_state`
//! - **`sub_states: [SubState1]`** - Add sub-states with `add_sub_state`
//! - **`reflect: [Type1, Type2]`** - Register types for reflection
//! - **`startup: [system1, system2]`** - Add startup systems
//! - **`update: [system3, system4]`** - Add update systems (supports conditions/ordering)
//! - **`fixed_update: [system5]`** - Add fixed update systems
//! - **`on_enter: { State::Variant => [system6] }`** - Add state enter systems
//! - **`on_exit: { State::Variant => [system7] }`** - Add state exit systems
//! - **`custom_init: |app| { ... }`** - Custom initialization logic
//! - **`custom_finish: |app| { ... }`** - Custom finish logic
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
//! #[derive(Event)]
//! struct GameStarted;
//!
//! define_plugin!(ComplexGamePlugin {
//!     // Type registration
//!     resources: [GameSettings],
//!     events: [GameStarted],
//!     states: [GameState],
//!     reflect: [GameSettings],
//!
//!     // System scheduling
//!     startup: [initialize_audio, load_assets],
//!
//!     update: [
//!         handle_menu_input.run_if(in_state(GameState::Menu)),
//!         (update_game, process_events).chain().run_if(in_state(GameState::Playing))
//!     ],
//!
//!     fixed_update: [
//!         physics_simulation.run_if(in_state(GameState::Playing))
//!     ],
//!
//!     // State transitions
//!     on_enter: {
//!         GameState::Playing => [setup_level, spawn_player],
//!         GameState::Paused => [show_pause_menu]
//!     },
//!
//!     on_exit: {
//!         GameState::Playing => [cleanup_level]
//!     },
//!
//!     // Custom logic
//!     custom_init: |app: &mut App| {
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
//! ## Migration Guide
//!
//! Converting existing plugins is straightforward:
//!
//! 1. **Replace `impl Plugin`** with `define_plugin!`
//! 2. **Group registrations by type** (resources, events, systems)
//! 3. **Use declarative syntax** instead of method chains
//!
//! See the [examples](https://github.com/noahsabaj/bevy-plugin-builder/tree/main/examples) for complete migration examples.
//!

// Private implementation modules
mod macros;

// Re-export commonly used Bevy types for convenience
pub use bevy::prelude::{App, FixedUpdate, OnEnter, OnExit, Plugin, Startup, Update};

// The macro is automatically available via #[macro_export] in macros.rs
