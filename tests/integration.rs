//! Integration tests for bevy-plugin-builder
//!
//! These tests verify that the `define_plugin!` macro generates
//! correct Plugin implementations that integrate properly with Bevy.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_plugin_builder::define_plugin;

// Test resources and components
#[derive(Resource, Default)]
struct TestResource {
    value: i32,
}

#[derive(Resource, Default)]
struct AnotherResource;

#[derive(Event)]
struct TestEvent;

#[derive(Event)]
struct AnotherEvent;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum TestState {
    #[default]
    StateA,
    StateB,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct TestComponent {
    value: f32,
}

// Test systems
fn startup_system(mut commands: Commands) {
    commands.spawn(TestComponent { value: 42.0 });
}

fn update_system(mut resource: ResMut<TestResource>) {
    resource.value += 1;
}

fn another_update_system(resource: Res<TestResource>) {
    if resource.value > 0 {
        info!("Resource value: {}", resource.value);
    }
}

fn enter_state_b(mut commands: Commands) {
    commands.spawn(TestComponent { value: 100.0 });
}

fn exit_state_a() {
    info!("Exiting state A");
}

// Test plugin with basic functionality
define_plugin!(BasicTestPlugin {
    resources: [TestResource],
    events: [TestEvent],
    startup: [startup_system],
    update: [update_system]
});

// Test plugin with all features
define_plugin!(FullFeatureTestPlugin {
    resources: [TestResource, AnotherResource],
    events: [TestEvent, AnotherEvent],
    states: [TestState],
    reflect: [TestComponent],

    startup: [startup_system],

    update: [
        update_system,
        another_update_system.run_if(in_state(TestState::StateA))
    ],

    on_enter: {
        TestState::StateB => [enter_state_b]
    },

    on_exit: {
        TestState::StateA => [exit_state_a]
    },

    custom_init: |app: &mut App| {
        app.insert_resource(ClearColor(Color::BLACK));
    },

    custom_finish: |app: &mut App| {
        // Validate resources are properly initialized
        assert!(app.world().contains_resource::<TestResource>());
        assert!(app.world().contains_resource::<AnotherResource>());
    }
});

#[test]
fn test_basic_plugin_compilation() {
    // Test that the basic plugin compiles and can be added to an app
    let mut app = App::new();
    app.add_plugins(BasicTestPlugin);

    // Verify resources were registered
    assert!(app.world().contains_resource::<TestResource>());

    // Verify the resource has the expected default value
    let resource = app.world().resource::<TestResource>();
    assert_eq!(resource.value, 0);
}

#[test]
fn test_full_feature_plugin() {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .add_plugins(FullFeatureTestPlugin);

    // Verify all resources were registered
    assert!(app.world().contains_resource::<TestResource>());
    assert!(app.world().contains_resource::<AnotherResource>());
    assert!(app.world().contains_resource::<ClearColor>());

    // Verify state was initialized
    assert!(app.world().contains_resource::<State<TestState>>());
    let state = app.world().resource::<State<TestState>>();
    assert_eq!(*state.get(), TestState::StateA);

    // Verify reflection was registered
    let registry = app.world().resource::<AppTypeRegistry>();
    let registry = registry.read();
    assert!(registry.contains(std::any::TypeId::of::<TestComponent>()));
}

#[test]
fn test_system_execution() {
    let mut app = App::new();
    app.add_plugins(BasicTestPlugin);

    // Run startup systems
    app.update();

    // Check that startup system ran (spawned entity)
    let entities: Vec<_> = app
        .world_mut()
        .query::<&TestComponent>()
        .iter(app.world())
        .collect();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].value, 42.0);

    // Run update systems
    app.update();

    // Check that update system ran (incremented resource)
    // After startup (value: 0) + one update (value: 1) + another update call above (value: 2)
    let resource = app.world().resource::<TestResource>();
    assert_eq!(resource.value, 2);
}

#[test]
fn test_state_transitions() {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .add_plugins(FullFeatureTestPlugin);

    // Initial state should be StateA
    let state = app.world().resource::<State<TestState>>();
    assert_eq!(*state.get(), TestState::StateA);

    // Transition to StateB
    app.world_mut()
        .resource_mut::<NextState<TestState>>()
        .set(TestState::StateB);
    app.update();

    // Verify transition happened
    let state = app.world().resource::<State<TestState>>();
    assert_eq!(*state.get(), TestState::StateB);

    // Verify on_enter system ran (spawned additional entity)
    let entities: Vec<_> = app
        .world_mut()
        .query::<&TestComponent>()
        .iter(app.world())
        .collect();
    assert_eq!(entities.len(), 2); // startup + on_enter systems

    // Find the entity with value 100.0 (from on_enter system)
    let has_enter_entity = entities.iter().any(|component| component.value == 100.0);
    assert!(has_enter_entity);
}

#[test]
fn test_event_registration() {
    let mut app = App::new();
    app.add_plugins(BasicTestPlugin);

    // Test that we can send events without errors
    let mut test_events = app.world_mut().resource_mut::<Events<TestEvent>>();
    test_events.send(TestEvent);

    // Verify that Events resource exists and we can access it
    assert!(app.world().contains_resource::<Events<TestEvent>>());

    // Run an update to process the events
    app.update();

    // The fact that we reached this point means event registration worked correctly
}

#[test]
fn test_custom_init_and_finish() {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .add_plugins(FullFeatureTestPlugin);

    // Verify custom_init ran (added ClearColor resource)
    assert!(app.world().contains_resource::<ClearColor>());
    let clear_color = app.world().resource::<ClearColor>();
    // Check the color value directly since ClearColor doesn't implement PartialEq
    assert_eq!(clear_color.0, Color::BLACK);

    // custom_finish validation happens during plugin setup
    // If we reach this point, the assertions in custom_finish passed
}

#[test]
fn test_conditional_systems() {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .add_plugins(FullFeatureTestPlugin);

    // Initial state is StateA, so conditional system should run
    app.update();

    // The conditional system (another_update_system) only runs in StateA
    // Since we start in StateA, it should have run and the resource should be incremented
    let resource = app.world().resource::<TestResource>();
    assert_eq!(resource.value, 1);

    // Transition to StateB
    app.world_mut()
        .resource_mut::<NextState<TestState>>()
        .set(TestState::StateB);
    app.update();

    // After transition, the conditional system should not run anymore
    // But the unconditional update_system should still run
    app.update();

    let resource = app.world().resource::<TestResource>();
    // Should be 3: initial update in StateA (1) + state transition update (2) + final update in StateB (3)
    assert_eq!(resource.value, 3);
}
