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

#[derive(Message)]
struct TestEvent;

#[derive(Message)]
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

#[derive(Resource, Default)]
struct NestedPluginResource;

struct ManualNestedPlugin;

impl Plugin for ManualNestedPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NestedPluginResource>();
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum RootFlowState {
    #[default]
    Setup,
    Active,
}

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(RootFlowState = RootFlowState::Active)]
#[allow(dead_code)]
enum ActiveSubState {
    #[default]
    PhaseOne,
    PhaseTwo,
}

#[derive(Resource, Default)]
struct SubStateTransitionCounter(u32);

fn enter_phase_one(mut counter: ResMut<SubStateTransitionCounter>) {
    counter.0 += 1;
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
    init_resource: [TestResource],
    add_message: [TestEvent],
    add_systems_startup: [startup_system],
    add_systems_update: [update_system]
});

// Test plugin with all features
define_plugin!(FullFeatureTestPlugin {
    init_resource: [TestResource, AnotherResource],
    add_message: [TestEvent, AnotherEvent],
    init_state: [TestState],
    register_type: [TestComponent],

    add_systems_startup: [startup_system],

    add_systems_update: [
        update_system,
        another_update_system.run_if(in_state(TestState::StateA))
    ],

    add_systems_on_enter: {
        TestState::StateB => [enter_state_b]
    },

    add_systems_on_exit: {
        TestState::StateA => [exit_state_a]
    },

    custom_build: |app: &mut App| {
        app.insert_resource(ClearColor(Color::BLACK));
    },

    custom_finish: |app: &mut App| {
        // Validate resources are properly initialized
        assert!(app.world().contains_resource::<TestResource>());
        assert!(app.world().contains_resource::<AnotherResource>());
    }
});

define_plugin!(ParentWithNestedPlugin {
    add_plugins: [ManualNestedPlugin]
});

define_plugin!(SubStateTestPlugin {
    init_resource: [SubStateTransitionCounter],
    init_state: [RootFlowState],
    add_sub_state: [ActiveSubState],

    add_systems_on_enter: {
        ActiveSubState::PhaseOne => [enter_phase_one]
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

    // Test that we can send messages without errors
    let mut test_events = app.world_mut().resource_mut::<Messages<TestEvent>>();
    test_events.write(TestEvent);

    // Verify that Messages resource exists and we can access it
    assert!(app.world().contains_resource::<Messages<TestEvent>>());

    // Run an update to process the messages
    app.update();

    // The fact that we reached this point means message registration worked correctly
}

#[test]
fn test_nested_plugin_registration() {
    let mut app = App::new();
    app.add_plugins(ParentWithNestedPlugin);

    assert!(app.world().contains_resource::<NestedPluginResource>());
}

#[test]
fn test_sub_state_registration_and_execution() {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .add_plugins(SubStateTestPlugin);

    assert!(app.world().contains_resource::<State<RootFlowState>>());
    assert!(!app.world().contains_resource::<State<ActiveSubState>>());

    let initial_counter = app.world().resource::<SubStateTransitionCounter>().0;
    assert_eq!(initial_counter, 0);

    app.world_mut()
        .resource_mut::<NextState<RootFlowState>>()
        .set(RootFlowState::Active);
    app.update();

    assert!(app.world().contains_resource::<State<ActiveSubState>>());

    let counter = app.world().resource::<SubStateTransitionCounter>();
    assert_eq!(counter.0, 1);
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

// Test resource for fixed update
#[derive(Resource, Default)]
struct FixedUpdateCounter(u32);

fn fixed_update_system(mut counter: ResMut<FixedUpdateCounter>) {
    counter.0 += 1;
}

define_plugin!(FixedUpdateTestPlugin {
    init_resource: [FixedUpdateCounter],
    add_systems_fixed_update: [fixed_update_system]
});

#[test]
fn test_fixed_update_systems() {
    let mut app = App::new();
    app.add_plugins(FixedUpdateTestPlugin);

    // Fixed update systems should run on fixed timestep
    app.update();

    let counter = app.world().resource::<FixedUpdateCounter>();
    // Counter should exist (u32 default is 0)
    assert_eq!(counter.0, 0);
}

// Test empty plugin
define_plugin!(EmptyPlugin {});

#[test]
fn test_empty_plugin() {
    let mut app = App::new();
    app.add_plugins(EmptyPlugin);

    // Empty plugin should compile and work without errors
    app.update();
}

// ============================================================================
// Dependency checking tests
// ============================================================================

use bevy_plugin_builder::{PluginDependencies, PluginMarker};

// Base plugin that others can depend on
#[derive(Resource, Default)]
struct PhysicsConfig;

fn physics_system() {}

define_plugin!(PhysicsPlugin {
    init_resource: [PhysicsConfig],
    add_systems_update: [physics_system]
});

// Plugin that depends on PhysicsPlugin
#[derive(Resource, Default)]
struct GameConfig;

fn game_system() {}

define_plugin!(GamePlugin {
    depends_on: [PhysicsPlugin],
    init_resource: [GameConfig],
    add_systems_update: [game_system]
});

// Plugin with multiple dependencies
#[derive(Resource, Default)]
struct AudioConfig;

fn audio_system() {}

define_plugin!(AudioPlugin {
    init_resource: [AudioConfig],
    add_systems_update: [audio_system]
});

#[derive(Resource, Default)]
struct UIConfig;

fn ui_system() {}

define_plugin!(UIPlugin {
    depends_on: [PhysicsPlugin, AudioPlugin],
    init_resource: [UIConfig],
    add_systems_update: [ui_system]
});

#[test]
fn test_plugin_marker_trait() {
    // All plugins should implement PluginMarker
    fn assert_plugin_marker<T: PluginMarker>() {}

    assert_plugin_marker::<PhysicsPlugin>();
    assert_plugin_marker::<GamePlugin>();
    assert_plugin_marker::<AudioPlugin>();
    assert_plugin_marker::<UIPlugin>();
    assert_plugin_marker::<BasicTestPlugin>();
    assert_plugin_marker::<EmptyPlugin>();
}

#[test]
fn test_plugin_dependencies_trait() {
    // All plugins should implement PluginDependencies
    fn assert_plugin_dependencies<T: PluginDependencies>() {}

    assert_plugin_dependencies::<PhysicsPlugin>();
    assert_plugin_dependencies::<GamePlugin>();
    assert_plugin_dependencies::<AudioPlugin>();
    assert_plugin_dependencies::<UIPlugin>();
}

#[test]
fn test_dependency_satisfied() {
    // When dependencies are added first, the plugin should work
    let mut app = App::new();
    app.add_plugins(PhysicsPlugin);
    app.add_plugins(GamePlugin);

    // Both resources should be registered
    assert!(app.world().contains_resource::<PhysicsConfig>());
    assert!(app.world().contains_resource::<GameConfig>());
}

#[test]
fn test_multiple_dependencies_satisfied() {
    // When all dependencies are added first, the plugin should work
    let mut app = App::new();
    app.add_plugins(PhysicsPlugin);
    app.add_plugins(AudioPlugin);
    app.add_plugins(UIPlugin);

    // All resources should be registered
    assert!(app.world().contains_resource::<PhysicsConfig>());
    assert!(app.world().contains_resource::<AudioConfig>());
    assert!(app.world().contains_resource::<UIConfig>());
}

#[test]
#[should_panic(expected = "requires")]
fn test_dependency_missing_panics() {
    // When a dependency is missing, the plugin should panic
    let mut app = App::new();
    // Deliberately NOT adding PhysicsPlugin first
    app.add_plugins(GamePlugin); // This should panic
}

#[test]
#[should_panic(expected = "requires")]
fn test_multiple_dependency_first_missing_panics() {
    // When the first of multiple dependencies is missing, it should panic
    let mut app = App::new();
    // Only add one of the two required dependencies
    app.add_plugins(AudioPlugin);
    app.add_plugins(UIPlugin); // This should panic because PhysicsPlugin is missing
}

#[test]
fn test_dependency_type_checking() {
    // Verify that PluginDependencies::Required has the correct type
    type GameDeps = <GamePlugin as PluginDependencies>::Required;
    type UIDeps = <UIPlugin as PluginDependencies>::Required;
    type EmptyDeps = <EmptyPlugin as PluginDependencies>::Required;

    // GamePlugin depends on one plugin
    #[allow(dead_code)]
    fn assert_single_dep<T: PluginMarker>(_: (T,)) {}
    let _: GameDeps = (PhysicsPlugin,);

    // UIPlugin depends on two plugins
    #[allow(dead_code)]
    fn assert_double_dep<T1: PluginMarker, T2: PluginMarker>(_: (T1, T2)) {}
    let _: UIDeps = (PhysicsPlugin, AudioPlugin);

    // EmptyPlugin has no dependencies
    #[allow(dead_code)]
    fn assert_no_deps(_: ()) {}
    let _: EmptyDeps = ();
}

// ============================================================================
// New Bevy-aligned syntax tests
// ============================================================================

#[derive(Resource, Default)]
struct NewSyntaxResource;

#[derive(Resource)]
struct InsertedResource {
    value: i32,
}

#[derive(Message)]
struct NewSyntaxMessage;

fn new_syntax_startup() {}
fn new_syntax_update() {}

// Test plugin using all new Bevy-aligned syntax
define_plugin!(NewSyntaxPlugin {
    init_resource: [NewSyntaxResource],
    insert_resource: [InsertedResource { value: 42 }],
    add_message: [NewSyntaxMessage],
    add_systems_startup: [new_syntax_startup],
    add_systems_update: [new_syntax_update]
});

#[test]
fn test_new_bevy_aligned_syntax() {
    let mut app = App::new();
    app.add_plugins(NewSyntaxPlugin);

    // Verify init_resource worked
    assert!(app.world().contains_resource::<NewSyntaxResource>());

    // Verify insert_resource worked with the correct value
    assert!(app.world().contains_resource::<InsertedResource>());
    let inserted = app.world().resource::<InsertedResource>();
    assert_eq!(inserted.value, 42);

    // Verify add_message worked
    assert!(app
        .world()
        .contains_resource::<Messages<NewSyntaxMessage>>());
}

// Test plugin with meta block (currently just skipped, for future introspection)
define_plugin!(MetaPlugin {
    meta: {
        version: "1.0.0",
        description: "A test plugin with metadata"
    },
    init_resource: [NewSyntaxResource]
});

#[test]
fn test_meta_block_compiles() {
    let mut app = App::new();
    app.add_plugins(MetaPlugin);

    // Meta block should be ignored for now but not cause errors
    assert!(app.world().contains_resource::<NewSyntaxResource>());
}

// Test plugin using new system scheduling syntax with states
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum NewSyntaxState {
    #[default]
    Idle,
    Active,
}

#[derive(Resource, Default)]
struct StateTransitionMarker(bool);

fn mark_entered(mut marker: ResMut<StateTransitionMarker>) {
    marker.0 = true;
}

define_plugin!(NewSyntaxStatePlugin {
    init_resource: [StateTransitionMarker],
    init_state: [NewSyntaxState],
    add_systems_on_enter: {
        NewSyntaxState::Active => [mark_entered]
    }
});

#[test]
fn test_new_syntax_state_systems() {
    let mut app = App::new();
    app.add_plugins(StatesPlugin);
    app.add_plugins(NewSyntaxStatePlugin);

    // Initial state
    let marker = app.world().resource::<StateTransitionMarker>();
    assert!(!marker.0);

    // Transition to Active
    app.world_mut()
        .resource_mut::<NextState<NewSyntaxState>>()
        .set(NewSyntaxState::Active);
    app.update();

    // Verify on_enter system ran
    let marker = app.world().resource::<StateTransitionMarker>();
    assert!(marker.0);
}

// Test custom_build (new name for custom_init)
#[derive(Resource)]
struct CustomBuildMarker;

define_plugin!(CustomBuildPlugin {
    custom_build: |app: &mut App| {
        app.insert_resource(CustomBuildMarker);
    }
});

#[test]
fn test_custom_build_syntax() {
    let mut app = App::new();
    app.add_plugins(CustomBuildPlugin);

    assert!(app.world().contains_resource::<CustomBuildMarker>());
}

// ============================================================================
// Introspection tests (feature-gated)
// ============================================================================

#[cfg(feature = "introspection")]
mod introspection_tests {
    use super::*;
    use bevy_plugin_builder::{PluginInfo, PluginRegistry};

    // Test plugin with full metadata
    #[derive(Resource, Default)]
    struct IntrospectionResource;

    #[derive(Message)]
    struct IntrospectionMessage;

    fn introspection_startup() {}
    fn introspection_update() {}

    define_plugin!(IntrospectionTestPlugin {
        meta: {
            version: "1.2.3",
            description: "A test plugin for introspection"
        },
        init_resource: [IntrospectionResource],
        add_message: [IntrospectionMessage],
        add_systems_startup: [introspection_startup],
        add_systems_update: [introspection_update]
    });

    #[test]
    fn test_plugin_info_trait() {
        // PluginInfo trait should be implemented
        assert_eq!(IntrospectionTestPlugin::NAME, "IntrospectionTestPlugin");
        assert_eq!(IntrospectionTestPlugin::VERSION, Some("1.2.3"));

        let metadata = IntrospectionTestPlugin::metadata();
        assert_eq!(metadata.name, "IntrospectionTestPlugin");
        assert_eq!(metadata.version, Some("1.2.3"));
        assert_eq!(
            metadata.description,
            Some("A test plugin for introspection")
        );
    }

    #[test]
    fn test_plugin_metadata_resources() {
        let metadata = IntrospectionTestPlugin::metadata();
        assert_eq!(metadata.resources.len(), 1);
        assert_eq!(metadata.resources[0].name, "IntrospectionResource");
        assert!(metadata.has_resource::<IntrospectionResource>());
        assert!(!metadata.has_resource::<String>()); // Non-existent resource
    }

    #[test]
    fn test_plugin_metadata_messages() {
        let metadata = IntrospectionTestPlugin::metadata();
        assert_eq!(metadata.messages.len(), 1);
        assert_eq!(metadata.messages[0].name, "IntrospectionMessage");
        assert!(metadata.has_message::<IntrospectionMessage>());
    }

    #[test]
    fn test_plugin_metadata_systems() {
        let metadata = IntrospectionTestPlugin::metadata();
        assert_eq!(metadata.systems.startup.len(), 1);
        assert_eq!(metadata.systems.startup[0], "introspection_startup");
        assert_eq!(metadata.systems.update.len(), 1);
        assert_eq!(metadata.systems.update[0], "introspection_update");
        assert_eq!(metadata.total_systems(), 2);
    }

    // Test plugin without metadata block
    define_plugin!(NoMetaPlugin {
        init_resource: [IntrospectionResource]
    });

    #[test]
    fn test_plugin_info_without_meta() {
        assert_eq!(NoMetaPlugin::NAME, "NoMetaPlugin");
        assert_eq!(NoMetaPlugin::VERSION, None);

        let metadata = NoMetaPlugin::metadata();
        assert_eq!(metadata.name, "NoMetaPlugin");
        assert!(metadata.version.is_none());
        assert!(metadata.description.is_none());
    }

    // Test plugin with dependencies recorded in metadata
    define_plugin!(DependentIntrospectionPlugin {
        depends_on: [PhysicsPlugin],
        init_resource: [IntrospectionResource]
    });

    #[test]
    fn test_plugin_metadata_dependencies() {
        let metadata = DependentIntrospectionPlugin::metadata();
        assert_eq!(metadata.dependencies.len(), 1);
        assert_eq!(metadata.dependencies[0], "PhysicsPlugin");
        assert!(metadata.depends_on("PhysicsPlugin"));
        assert!(!metadata.depends_on("NonExistent"));
    }

    // Test PluginRegistry integration
    #[test]
    fn test_plugin_registry_manual() {
        let mut registry = PluginRegistry::new();

        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register::<IntrospectionTestPlugin>();

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.is_registered::<IntrospectionTestPlugin>());
        assert!(!registry.is_registered::<NoMetaPlugin>());

        let metadata = registry.get::<IntrospectionTestPlugin>().unwrap();
        assert_eq!(metadata.name, "IntrospectionTestPlugin");
        assert_eq!(metadata.version, Some("1.2.3"));
    }

    #[test]
    fn test_plugin_registry_queries() {
        let mut registry = PluginRegistry::new();
        registry.register::<IntrospectionTestPlugin>();
        registry.register::<NoMetaPlugin>();

        // Query by resource type
        let plugins = registry.plugins_with_resource::<IntrospectionResource>();
        assert_eq!(plugins.len(), 2);
        assert!(plugins.contains(&"IntrospectionTestPlugin"));
        assert!(plugins.contains(&"NoMetaPlugin"));

        // Query by message type
        let plugins = registry.plugins_with_message::<IntrospectionMessage>();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0], "IntrospectionTestPlugin");

        // Find by name
        let found = registry.find_by_name("IntrospectionTestPlugin");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "IntrospectionTestPlugin");

        // Total counts
        assert_eq!(registry.total_resources(), 2);
        assert_eq!(registry.total_systems(), 2); // Only IntrospectionTestPlugin has systems
    }

    #[test]
    fn test_plugin_registry_list_order() {
        let mut registry = PluginRegistry::new();
        registry.register::<IntrospectionTestPlugin>();
        registry.register::<NoMetaPlugin>();
        registry.register::<DependentIntrospectionPlugin>();

        let names: Vec<_> = registry.plugin_names();
        assert_eq!(names.len(), 3);
        // Should be in registration order
        assert_eq!(names[0], "IntrospectionTestPlugin");
        assert_eq!(names[1], "NoMetaPlugin");
        assert_eq!(names[2], "DependentIntrospectionPlugin");
    }
}

// =============================================================================
// Testing Feature Tests (generate_tests: syntax)
// =============================================================================
// Note: Tests for the generate_tests: feature work differently. The macro
// generates #[test] functions that are picked up by the test harness directly.
// Here we test that the syntax compiles correctly.

#[cfg(feature = "testing")]
mod testing_feature_tests {
    use super::*;

    // Define a simple resource for testing module scope
    #[derive(Resource, Default)]
    struct TestingModuleResource;

    #[derive(Resource, Default)]
    struct AnotherTestingResource;

    #[derive(Message)]
    struct TestingModuleEvent;

    #[derive(Message)]
    struct AnotherTestingEvent;

    fn testable_startup() {}
    fn testable_update() {}

    // Test that generate_tests: syntax compiles with various options
    define_plugin!(TestableResourcePlugin {
        init_resource: [TestingModuleResource],
        generate_tests: {
            test_resources: true
        }
    });

    #[test]
    fn test_testable_resource_plugin_compiles() {
        // This test verifies the plugin with generate_tests compiles
        let mut app = App::new();
        app.add_plugins(TestableResourcePlugin);
        assert!(app.world().contains_resource::<TestingModuleResource>());
    }

    // Test generate_tests with multiple options
    define_plugin!(TestableMultiOptionPlugin {
        init_resource: [TestingModuleResource, AnotherTestingResource],
        add_message: [TestingModuleEvent, AnotherTestingEvent],
        generate_tests: {
            test_resources: true,
            test_messages: true
        }
    });

    #[test]
    fn test_multi_option_plugin_compiles() {
        let mut app = App::new();
        app.add_plugins(TestableMultiOptionPlugin);
        assert!(app.world().contains_resource::<TestingModuleResource>());
        assert!(app.world().contains_resource::<AnotherTestingResource>());
    }

    // Test generate_tests: with false values (should skip those tests)
    define_plugin!(TestableSelectivePlugin {
        init_resource: [TestingModuleResource],
        add_message: [TestingModuleEvent],
        generate_tests: {
            test_resources: true,
            test_messages: false  // Should not generate message tests
        }
    });

    #[test]
    fn test_selective_plugin_compiles() {
        let mut app = App::new();
        app.add_plugins(TestableSelectivePlugin);
        assert!(app.world().contains_resource::<TestingModuleResource>());
    }

    // Test generate_tests with state testing
    #[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
    #[allow(dead_code)]
    enum TestingModuleState {
        #[default]
        Idle,
        Active,
    }

    define_plugin!(TestableStatePlugin {
        init_state: [TestingModuleState],
        generate_tests: {
            test_states: true
        }
    });

    #[test]
    fn test_state_plugin_compiles() {
        let mut app = App::new();
        app.add_plugins(StatesPlugin);
        app.add_plugins(TestableStatePlugin);
        assert!(app.world().contains_resource::<State<TestingModuleState>>());
    }

    // Test generate_tests alongside other complex options
    define_plugin!(TestableComplexPlugin {
        meta: {
            name: "TestableComplex",
            version: "1.0.0"
        },
        init_resource: [TestingModuleResource],
        add_message: [TestingModuleEvent],
        add_systems_startup: [testable_startup],
        add_systems_update: [testable_update],
        generate_tests: {
            test_resources: true,
            test_messages: true
        }
    });

    #[test]
    fn test_complex_plugin_with_generate_tests_compiles() {
        let mut app = App::new();
        app.add_plugins(TestableComplexPlugin);
        assert!(app.world().contains_resource::<TestingModuleResource>());
    }

    // Test generate_tests with custom_build closure after it
    define_plugin!(TestableWithCustomBuild {
        init_resource: [TestingModuleResource],
        generate_tests: {
            test_resources: true
        },
        custom_build: |app: &mut App| {
            // Custom logic here
            let _ = app;
        }
    });

    #[test]
    fn test_plugin_with_custom_build_after_generate_tests() {
        let mut app = App::new();
        app.add_plugins(TestableWithCustomBuild);
        assert!(app.world().contains_resource::<TestingModuleResource>());
    }
}
