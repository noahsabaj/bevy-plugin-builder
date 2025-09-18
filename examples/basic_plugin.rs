//! Basic Plugin Example
//!
//! This example demonstrates the simplest usage of `define_plugin!`
//! for registering resources, events, and systems.

use bevy::prelude::*;
use bevy_plugin_builder::define_plugin;

// Define game resources
#[derive(Resource, Default)]
struct GameSettings {
    volume: f32,
    difficulty: u8,
}

#[derive(Resource, Default)]
struct ScoreResource {
    current: u32,
    high: u32,
}

// Define game events
#[derive(Event)]
struct PlayerScored {
    points: u32,
}

#[derive(Event)]
struct GameOver {
    final_score: u32,
}

// Define systems
fn setup_game(mut commands: Commands) {
    info!("🎮 Game setup complete!");
    commands.spawn((Name::new("Player"), Transform::default()));
}

fn update_score(
    mut score: ResMut<ScoreResource>,
    mut score_events: EventReader<PlayerScored>,
    mut game_over_events: EventWriter<GameOver>,
) {
    for event in score_events.read() {
        score.current += event.points;
        info!("Score: {}", score.current);

        if score.current >= 1000 {
            game_over_events.write(GameOver {
                final_score: score.current,
            });
        }
    }
}

fn handle_game_over(mut game_over_events: EventReader<GameOver>) {
    for event in game_over_events.read() {
        info!("🏆 Game Over! Final Score: {}", event.final_score);
    }
}

fn check_input(keyboard: Res<ButtonInput<KeyCode>>, mut score_events: EventWriter<PlayerScored>) {
    if keyboard.just_pressed(KeyCode::Space) {
        score_events.write(PlayerScored { points: 10 });
    }
}

// 🚀 THE MAGIC: Replace 20+ lines of boilerplate with 8 lines of intent!
define_plugin!(BasicGamePlugin {
    resources: [GameSettings, ScoreResource],
    events: [PlayerScored, GameOver],
    startup: [setup_game],
    update: [update_score, handle_game_over, check_input]
});

// Compare with traditional Bevy plugin:
/*
pub struct BasicGamePlugin;

impl Plugin for BasicGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSettings>()
           .init_resource::<ScoreResource>()
           .add_event::<PlayerScored>()
           .add_event::<GameOver>()
           .add_systems(Startup, setup_game)
           .add_systems(Update, (
               update_score,
               handle_game_over,
               check_input,
           ));
    }
}
*/

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BasicGamePlugin)
        .run();
}
