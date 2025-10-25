//! Migration Guide Example
//!
//! This example shows how to convert traditional Bevy plugins
//! to use `define_plugin!` declarative syntax.
//!
//! Run with: cargo run --example migration_guide

use bevy::prelude::*;
use bevy_plugin_builder::define_plugin;

// Sample game components and resources
#[derive(Resource, Default)]
struct AudioSettings;

#[derive(Resource, Default)]
struct InputSettings;

#[derive(Message)]
struct VolumeChanged {
    new_volume: f32,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum AudioState {
    #[default]
    Enabled,
    Disabled,
}

// Sample systems
fn setup_audio_system() {
    info!("Audio system initialized");
}

fn update_volume(mut volume_events: MessageReader<VolumeChanged>) {
    for event in volume_events.read() {
        info!("Volume changed to: {}", event.new_volume);
    }
}

fn process_audio() {
    // Audio processing logic
}

fn handle_mute_toggle(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        info!("Mute toggled");
    }
}

fn enter_audio_enabled() {
    info!("Audio enabled");
}

fn enter_audio_disabled() {
    info!("Audio disabled");
}

// =============================================================================
// BEFORE: Traditional Bevy Plugin
// =============================================================================
/*
pub struct AudioPluginOld;

impl Plugin for AudioPluginOld {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<AudioSettings>()
            .init_resource::<InputSettings>()

            // Messages
            .add_message::<VolumeChanged>()

            // States
            .init_state::<AudioState>()

            // Startup systems
            .add_systems(Startup, setup_audio_system)

            // Update systems with complex conditions
            .add_systems(
                Update,
                (
                    update_volume,
                    process_audio.run_if(in_state(AudioState::Enabled)),
                    handle_mute_toggle,
                )
            )

            // State transition systems
            .add_systems(
                OnEnter(AudioState::Enabled),
                enter_audio_enabled
            )
            .add_systems(
                OnEnter(AudioState::Disabled),
                enter_audio_disabled
            );
    }

    fn finish(&self, app: &mut App) {
        info!("AudioPlugin setup complete!");

        // Validation logic
        if !app.world().contains_resource::<AudioSettings>() {
            panic!("AudioSettings not initialized properly!");
        }
    }
}
*/

// =============================================================================
// AFTER: Declarative Plugin
// =============================================================================

define_plugin!(AudioPlugin {
    // All registration in organized sections
    resources: [AudioSettings, InputSettings],
    messages: [VolumeChanged],
    states: [AudioState],

    // System scheduling
    startup: [setup_audio_system],

    update: [
        update_volume,
        process_audio.run_if(in_state(AudioState::Enabled)),
        handle_mute_toggle
    ],

    // State transitions
    on_enter: {
        AudioState::Enabled => [enter_audio_enabled],
        AudioState::Disabled => [enter_audio_disabled]
    },

    // Custom logic
    custom_finish: |app: &mut App| {
        info!("AudioPlugin setup complete!");
        if !app.world().contains_resource::<AudioSettings>() {
            panic!("AudioSettings not initialized properly!");
        }
    }
});

// =============================================================================
// MIGRATION STEPS
// =============================================================================

// Step-by-step migration process:
//
// 1. **Replace structure**:
//    `impl Plugin for MyPlugin` → `define_plugin!(MyPlugin { ... })`
//
// 2. **Group by type**:
//    - All `init_resource` calls → `resources: [Type1, Type2]`
//    - All `add_message` calls → `messages: [Message1, Message2]`
//    - All `init_state` calls → `states: [State1]`
//
// 3. **Organize systems**:
//    - `add_systems(Startup, ...)` → `startup: [...]`
//    - `add_systems(Update, ...)` → `update: [...]`
//    - `add_systems(OnEnter(...), ...)` → `on_enter: { State => [...] }`
//
// 4. **Handle custom logic**:
//    - Complex `build()` logic → `custom_init: |app| { ... }`
//    - Custom `finish()` logic → `custom_finish: |app| { ... }`
//
// 5. **Preserve conditions**:
//    - System conditions and ordering work exactly the same
//    - `.run_if(...)`, `.chain()`, etc. work identically

// =============================================================================
// REAL-WORLD MIGRATION EXAMPLE
// =============================================================================

// Before: Typical camera plugin
/*
pub struct CameraPluginOld;

impl Plugin for CameraPluginOld {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CameraBounds>()
            .init_resource::<WindowFocusState>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                (
                    handle_keyboard_movement,
                    handle_mouse_wheel_zoom,
                    handle_mouse_drag,
                    handle_edge_panning,
                    handle_camera_reset,
                    window::handle_window_focus,
                    apply_smooth_movement,
                    apply_camera_bounds,
                )
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                OnEnter(GameState::InGame),
                (
                    calculate_camera_bounds,
                    setup_cursor_confinement,
                ),
            )
            .add_systems(
                OnExit(GameState::InGame),
                release_cursor_confinement,
            );
    }
}
*/

// After: Same functionality with declarative syntax
/*
define_plugin!(CameraPlugin {
    resources: [CameraBounds, WindowFocusState],

    startup: [setup_camera],

    update: [
        (
            handle_keyboard_movement,
            handle_mouse_wheel_zoom,
            handle_mouse_drag,
            handle_edge_panning,
            handle_camera_reset,
            window::handle_window_focus,
            apply_smooth_movement,
            apply_camera_bounds,
        ).chain().run_if(in_state(GameState::InGame))
    ],

    on_enter: {
        GameState::InGame => [calculate_camera_bounds, setup_cursor_confinement]
    },

    on_exit: {
        GameState::InGame => [release_cursor_confinement]
    }
});
*/
// =============================================================================
// BENEFITS DEMONSTRATED
// =============================================================================

// Migration Benefits:
//
// - Less code with declarative syntax
// - No repetitive method chains
// - Improved readability
// - Compile-time validation
// - Plugin capabilities visible at a glance
// - Easier maintenance with logical sections

fn main() {
    println!("Migration Guide Example");
    println!("==========================");
    println!();
    println!("Traditional Plugin vs Declarative Plugin");
    println!("Declarative syntax reduces boilerplate");
    println!();
    println!("Run `cargo run --example basic_plugin` to see it in action!");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioPlugin)
        .run();
}
