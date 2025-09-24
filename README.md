# bevy-plugin-builder

Declarative plugin system for Bevy

 [![Crates.io](https://img.shields.io/crates/v/bevy-plugin-builder.svg)](https://crates.io/crates/bevy-plugin-builder)
 [![Documentation](https://docs.rs/bevy-plugin-builder/badge.svg)](https://docs.rs/bevy-plugin-builder)
 [![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/noahsabaj/bevy-plugin-builder)
 [![Bevy 
 tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/quick-start/plugin-development/)

 ## Before & After

 ### Before (Traditional Bevy Plugin)
 ```rust
 pub struct GamePlugin;

 impl Plugin for GamePlugin {
     fn build(&self, app: &mut App) {
         app.init_resource::<GameSettings>()
            .init_resource::<PlayerStats>()
            .add_event::<PlayerLevelUp>()
            .add_event::<GameOver>()
            .add_systems(Startup, initialize_game)
            .add_systems(Update, (
                handle_input,
                update_player,
                check_collisions
            ).chain().run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::GameOver), cleanup_game);
     }
 }
 ```
 After (bevy-plugin-builder)
 ```rust
 use bevy_plugin_builder::define_plugin;

 define_plugin!(GamePlugin {
     resources: [GameSettings, PlayerStats],
     events: [PlayerLevelUp, GameOver],
     startup: [initialize_game],
     update: [
         (handle_input, update_player, check_collisions)
             .chain().run_if(in_state(GameState::Playing))
     ],
     on_enter: { GameState::GameOver => [cleanup_game] }
 });
 ```

## Installation

 [dependencies]
 bevy-plugin-builder = "0.1"
 bevy = "0.16"