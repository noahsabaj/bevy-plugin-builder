//! Complex Plugin Example
//!
//! This example demonstrates advanced features of `define_plugin!`:
//! - State management with enter/exit systems
//! - Complex system scheduling with conditions
//! - Custom initialization and finish logic
//! - Reflection, sub-states, and all registration types

use bevy::prelude::*;
use bevy_plugin_builder::define_plugin;

// Game states
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(GameState = GameState::Playing)]
enum PlayingSubState {
    #[default]
    Normal,
    BossLevel,
}

// Game resources
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct GameSettings {
    volume: f32,
    difficulty: u8,
    debug_mode: bool,
}

#[derive(Resource, Default)]
struct PlayerStats {
    health: f32,
    level: u32,
    experience: u32,
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct GameTimer {
    seconds: f32,
}

// Game events
#[derive(Event, Reflect)]
struct LevelUp {
    new_level: u32,
}

#[derive(Event)]
struct PlayerDamaged {
    damage: f32,
}

#[derive(Event)]
struct BossDefeated;

// Game components
#[derive(Component, Reflect)]
struct Player {
    speed: f32,
}

#[derive(Component, Reflect)]
struct Boss {
    health: f32,
}

#[derive(Component)]
struct MenuText;

// Systems
fn setup_game_world(mut commands: Commands) {
    info!("Setting up game world...");
    commands.spawn(Camera2d);
}

fn load_game_assets() {
    info!("Loading game assets...");
    // Simulate asset loading
}

fn enter_main_menu(mut commands: Commands) {
    info!("Entering main menu");
    commands.spawn((
        Text::new("Press ENTER to play"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        MenuText,
        Name::new("MenuText"),
    ));
}

fn exit_main_menu(mut commands: Commands, query: Query<Entity, With<MenuText>>) {
    info!("Exiting main menu");
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn enter_playing(mut commands: Commands) {
    info!("Starting game!");
    commands.spawn((
        Player { speed: 100.0 },
        Transform::default(),
        Name::new("Player"),
    ));
}

fn exit_playing(mut commands: Commands, query: Query<Entity, With<Player>>) {
    info!("Stopping game");
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Playing);
    }
}

fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => {}
        }
    }
}

fn update_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &Player)>,
) {
    for (mut transform, player) in &mut query {
        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }

        if direction != Vec3::ZERO {
            transform.translation += direction.normalize() * player.speed * time.delta_secs();
        }
    }
}

fn update_game_timer(time: Res<Time>, mut timer: ResMut<GameTimer>) {
    timer.seconds += time.delta_secs();
}

fn check_level_up(mut stats: ResMut<PlayerStats>, mut level_up_events: EventWriter<LevelUp>) {
    let required_xp = (stats.level + 1) * 100;
    if stats.experience >= required_xp {
        stats.level += 1;
        stats.experience = 0;
        level_up_events.write(LevelUp {
            new_level: stats.level,
        });
        info!("Level up! New level: {}", stats.level);
    }
}

fn handle_damage(
    mut damage_events: EventReader<PlayerDamaged>,
    mut stats: ResMut<PlayerStats>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in damage_events.read() {
        stats.health -= event.damage;
        info!(
            "Player took {} damage! Health: {}",
            event.damage, stats.health
        );

        if stats.health <= 0.0 {
            next_state.set(GameState::GameOver);
        }
    }
}

fn physics_simulation(mut query: Query<&mut Transform, With<Player>>) {
    // Fixed update physics
    for mut transform in &mut query {
        // Apply gravity, collision detection, etc.
        transform.translation.y -= 9.8 * 0.016; // Simplified gravity
    }
}

fn spawn_boss(mut commands: Commands, mut next_sub_state: ResMut<NextState<PlayingSubState>>) {
    commands.spawn((
        Boss { health: 100.0 },
        Transform::from_xyz(200.0, 0.0, 0.0),
        Name::new("Boss"),
    ));
    next_sub_state.set(PlayingSubState::BossLevel);
    info!("Boss spawned!");
}

fn cleanup_boss(mut commands: Commands, query: Query<Entity, With<Boss>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    info!("Boss defeated!");
}

// All advanced features in one declarative block
define_plugin!(ComplexGamePlugin {
    // Type registration
    resources: [GameSettings, PlayerStats, GameTimer],
    events: [LevelUp, PlayerDamaged, BossDefeated],
    states: [GameState],
    sub_states: [PlayingSubState],
    reflect: [GameSettings, Player, Boss, LevelUp],

    // System scheduling
    startup: [setup_game_world, load_game_assets],

    update: [
        // Menu systems
        handle_menu_input.run_if(in_state(GameState::MainMenu)),

        // Game systems with complex conditions
        handle_pause_input.run_if(not(in_state(GameState::MainMenu))),
        (update_player, update_game_timer, check_level_up)
            .chain()
            .run_if(in_state(GameState::Playing)),

        // Damage handling
        handle_damage.run_if(any_with_component::<Player>)
    ],

    fixed_update: [
        physics_simulation.run_if(in_state(GameState::Playing))
    ],

    // State transitions
    on_enter: {
        GameState::MainMenu => [enter_main_menu],
        GameState::Playing => [enter_playing],
        PlayingSubState::BossLevel => [spawn_boss]
    },

    on_exit: {
        GameState::MainMenu => [exit_main_menu],
        GameState::Playing => [exit_playing],
        PlayingSubState::BossLevel => [cleanup_boss]
    },

    // Custom logic with proper type annotations
    custom_init: |app: &mut App| {
        // Conditional plugin registration
        #[cfg(debug_assertions)]
        {
            info!("Debug mode enabled - adding diagnostics");
            app.add_plugins(bevy::diagnostic::DiagnosticsPlugin);
        }

        // Custom resource initialization
        app.insert_resource(ClearColor(Color::BLACK));

        // Register additional reflection types
        app.register_type::<GameTimer>();
    },

    custom_finish: |app: &mut App| {
        info!("ComplexGamePlugin initialization complete!");

        // Validate plugin setup
        if !app.world().contains_resource::<GameSettings>() {
            panic!("GameSettings resource not properly initialized!");
        }
    }
});

// Traditional implementation would be 80+ lines of complex setup!
// Declarative approach: 25 lines

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ComplexGamePlugin)
        .run();
}
