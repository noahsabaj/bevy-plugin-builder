//! Declarative plugin registration macro implementation
//!
//! This module contains the main `define_plugin!` macro that generates
//! Bevy plugin implementations from declarative syntax.

/// Define a Bevy plugin declaratively, eliminating boilerplate registration code.
///
/// This macro takes a plugin name and a configuration block, then generates
/// a complete `impl Plugin for PluginName` with all the specified registrations.
///
/// ## Supported Configuration Options
///
/// All keywords are aligned with Bevy's API for familiarity.
///
/// ### Metadata
/// - `meta: { version: "1.0.0", description: "..." }` - Plugin metadata
///
/// ### Dependencies
/// - `depends_on: [Plugin1, Plugin2]` - Declare plugin dependencies
///
/// ### Type Registration (Bevy-aligned naming)
/// - `init_resource: [Type1, Type2]` - Initialize resources with `init_resource`
/// - `insert_resource: [Value1, Value2]` - Insert resources with values
/// - `add_message: [Msg1, Msg2]` - Register messages with `add_message`
/// - `add_plugins: [Plugin1, Plugin2]` - Add sub-plugins
/// - `init_state: [State1]` - Initialize states
/// - `add_sub_state: [SubState1]` - Add sub-states
/// - `register_type: [Type1, Type2]` - Register types for reflection
///
/// ### System Scheduling (Bevy-aligned naming)
/// - `add_systems_startup: [system1, system2]` - Add startup systems
/// - `add_systems_update: [system3, system4]` - Add update systems
/// - `add_systems_fixed_update: [system5]` - Add fixed update systems
/// - `add_systems_on_enter: { State::Variant => [system6] }` - State enter systems
/// - `add_systems_on_exit: { State::Variant => [system7] }` - State exit systems
///
/// ### Custom Logic
/// - `custom_build: |app| { ... }` - Custom build logic
/// - `custom_finish: |app| { ... }` - Custom finish logic
///
/// ## Example
///
/// ```rust
/// use bevy_plugin_builder::define_plugin;
/// use bevy::prelude::*;
///
/// #[derive(Resource, Default)]
/// struct GameSettings;
///
/// #[derive(Message)]
/// struct GameStarted;
///
/// fn setup() {}
/// fn game_loop() {}
///
/// define_plugin!(MyGamePlugin {
///     init_resource: [GameSettings],
///     add_message: [GameStarted],
///     add_systems_startup: [setup],
///     add_systems_update: [game_loop]
/// });
/// ```
///
/// ## Example with Dependencies
///
/// ```rust
/// use bevy_plugin_builder::define_plugin;
/// use bevy::prelude::*;
///
/// #[derive(Resource, Default)]
/// struct PhysicsConfig;
/// fn physics_system() {}
///
/// define_plugin!(PhysicsPlugin {
///     init_resource: [PhysicsConfig],
///     add_systems_update: [physics_system]
/// });
///
/// #[derive(Resource, Default)]
/// struct GameConfig;
/// fn game_system() {}
///
/// define_plugin!(GamePlugin {
///     depends_on: [PhysicsPlugin],
///     init_resource: [GameConfig],
///     add_systems_update: [game_system]
/// });
/// ```
#[macro_export]
macro_rules! define_plugin {
    // Main entry point - delegates to internal implementation
    ($plugin_name:ident { $($config:tt)* }) => {
        $crate::define_plugin_impl!($plugin_name { $($config)* });
        // Generate metadata when introspection feature is enabled
        $crate::define_plugin_metadata!($plugin_name { $($config)* });
        // Generate tests when testing feature is enabled
        $crate::define_plugin_tests!($plugin_name { $($config)* });
    };
}

/// Internal implementation macro that handles the actual code generation.
/// This separates the entry point from the implementation details.
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_impl {
    // Case 1: Plugin WITH dependencies (depends_on must be first if present)
    ($plugin_name:ident {
        depends_on: [$($dep:ty),* $(,)?]
        $(, $($rest:tt)*)?
    }) => {
        pub struct $plugin_name;

        // PluginMarker trait - enables compile-time dependency checking
        impl $crate::PluginMarker for $plugin_name {
            type Id = $plugin_name;
        }

        // PluginDependencies trait - declares what this plugin requires
        impl $crate::PluginDependencies for $plugin_name {
            type Required = ($($dep,)*);
        }

        impl ::bevy::prelude::Plugin for $plugin_name {
            fn build(&self, app: &mut ::bevy::prelude::App) {
                // Compile-time check: verify dependency types implement PluginMarker
                $(
                    let _: <$dep as $crate::PluginMarker>::Id;
                )*

                // Runtime check: verify dependencies were added in correct order
                if let Err(e) = <Self as $crate::PluginDependencies>::verify_dependencies(app) {
                    panic!("{}", e);
                }

                // Process remaining configuration
                $crate::define_plugin_internal!(app, $($($rest)*)?);
            }

            fn finish(&self, app: &mut ::bevy::prelude::App) {
                $crate::define_plugin_finish!(app, $($($rest)*)?);
            }
        }
    };

    // Case 2: Plugin WITHOUT dependencies (backward compatible)
    ($plugin_name:ident { $($config:tt)* }) => {
        pub struct $plugin_name;

        // PluginMarker trait - all plugins get this for dependency checking
        impl $crate::PluginMarker for $plugin_name {
            type Id = $plugin_name;
        }

        // PluginDependencies with empty tuple - no dependencies
        impl $crate::PluginDependencies for $plugin_name {
            type Required = ();
        }

        impl ::bevy::prelude::Plugin for $plugin_name {
            fn build(&self, app: &mut ::bevy::prelude::App) {
                $crate::define_plugin_internal!(app, $($config)*);
            }

            fn finish(&self, app: &mut ::bevy::prelude::App) {
                $crate::define_plugin_finish!(app, $($config)*);
            }
        }
    };
}

/// Internal macro for parsing and applying plugin configuration.
/// This is separate from the main macro to allow for recursive parsing.
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_internal {
    // Empty configuration (base case)
    ($app:ident,) => {};

    // ========================================================================
    // Skip meta and depends_on (handled elsewhere or for introspection)
    // ========================================================================

    ($app:ident, meta: { $($meta:tt)* } $(, $($rest:tt)*)?) => {
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    ($app:ident, depends_on: [$($dep:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // ========================================================================
    // NEW Bevy-aligned syntax
    // ========================================================================

    // init_resource: (new name for resources:)
    ($app:ident, init_resource: [$($resource:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $(
            $app.init_resource::<$resource>();
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // insert_resource: (new - insert resources with values)
    ($app:ident, insert_resource: [$($resource:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $(
            $app.insert_resource($resource);
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // add_message: (Bevy 0.17+ uses Messages instead of Events)
    ($app:ident, add_message: [$($message:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $(
            $app.add_message::<$message>();
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // add_plugins: (new name for plugins:)
    ($app:ident, add_plugins: [$($plugin:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $(
            $app.add_plugins($plugin);
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // init_state: (new name for states:)
    ($app:ident, init_state: [$($state:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $(
            $app.init_state::<$state>();
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // add_sub_state: (new name for sub_states:)
    ($app:ident, add_sub_state: [$($state:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $(
            $app.add_sub_state::<$state>();
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // register_type: (new name for reflect:)
    ($app:ident, register_type: [$($reflect_type:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $(
            $app.register_type::<$reflect_type>();
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // add_systems_startup: (new name for startup:)
    ($app:ident, add_systems_startup: [$($system:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $app.add_systems(
            ::bevy::prelude::Startup,
            ($($system,)*)
        );
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // add_systems_update: (new name for update:)
    ($app:ident, add_systems_update: [$($system:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $app.add_systems(
            ::bevy::prelude::Update,
            ($($system,)*)
        );
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // add_systems_fixed_update: (new name for fixed_update:)
    ($app:ident, add_systems_fixed_update: [$($system:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $app.add_systems(
            ::bevy::prelude::FixedUpdate,
            ($($system,)*)
        );
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // add_systems_on_enter: (new name for on_enter:)
    ($app:ident, add_systems_on_enter: { $($state:expr => [$($system:expr),* $(,)?]),* $(,)? } $(, $($rest:tt)*)?) => {
        $(
            $app.add_systems(
                ::bevy::prelude::OnEnter($state),
                ($($system,)*)
            );
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // add_systems_on_exit: (new name for on_exit:)
    ($app:ident, add_systems_on_exit: { $($state:expr => [$($system:expr),* $(,)?]),* $(,)? } $(, $($rest:tt)*)?) => {
        $(
            $app.add_systems(
                ::bevy::prelude::OnExit($state),
                ($($system,)*)
            );
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // custom_build: (new name for custom_init:)
    ($app:ident, custom_build: $build_fn:expr $(, $($rest:tt)*)?) => {
        $build_fn($app);
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // custom_finish: (skip in build, handled in finish)
    ($app:ident, custom_finish: $finish_fn:expr $(, $($rest:tt)*)?) => {
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // generate_tests: (skip in build, handled by separate macro)
    ($app:ident, generate_tests: { $($test_config:tt)* } $(, $($rest:tt)*)?) => {
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // ========================================================================
    // Error case - unrecognized configuration
    // ========================================================================

    ($app:ident, $unknown:tt $($rest:tt)*) => {
        compile_error!(concat!(
            "Unknown plugin configuration option: ",
            stringify!($unknown),
            "\nSupported options: depends_on, meta, init_resource, insert_resource, add_message, add_plugins, init_state, add_sub_state, register_type, add_systems_startup, add_systems_update, add_systems_fixed_update, add_systems_on_enter, add_systems_on_exit, custom_build, custom_finish, generate_tests"
        ));
    };
}

/// Macro for handling Plugin finish() method configuration
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_finish {
    // Empty configuration (base case) - default finish does nothing
    ($app:ident,) => {};

    // Skip all standard configurations (only process custom_finish)
    ($app:ident, meta: { $($meta:tt)* } $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, depends_on: [$($dep:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, init_resource: [$($resource:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, insert_resource: [$($resource:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, add_message: [$($message:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, add_plugins: [$($plugin:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, init_state: [$($state:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, add_sub_state: [$($state:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, register_type: [$($reflect_type:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, add_systems_startup: [$($system:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, add_systems_update: [$($system:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, add_systems_fixed_update: [$($system:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, add_systems_on_enter: { $($state:expr => [$($system:expr),* $(,)?]),* $(,)? } $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, add_systems_on_exit: { $($state:expr => [$($system:expr),* $(,)?]),* $(,)? } $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, custom_build: $build_fn:expr $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, generate_tests: { $($test_config:tt)* } $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };

    // Custom finish - this is what we're looking for!
    ($app:ident, custom_finish: $finish_fn:expr $(, $($rest:tt)*)?) => {
        $finish_fn($app);
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };

    // Handle all other configurations (catch-all for unknown tokens)
    ($app:ident, $unknown:tt $($rest:tt)*) => {
        $crate::define_plugin_finish!($app, $($rest)*);
    };
}

// ============================================================================
// Introspection support (feature-gated)
// ============================================================================

/// Helper macro to count items in a list (used for static array sizing)
#[macro_export]
#[doc(hidden)]
macro_rules! count_items {
    () => { 0usize };
    ($first:ty $(, $rest:ty)*) => {
        1usize + $crate::count_items!($($rest),*)
    };
    ($first:expr $(, $rest:expr)*) => {
        1usize + $crate::count_items!($($rest),*)
    };
}

/// Helper macro to generate TypeInfo array for a list of types
#[macro_export]
#[doc(hidden)]
macro_rules! type_info_array {
    ($name:ident: [$($ty:ty),* $(,)?]) => {
        static $name: &'static [$crate::TypeInfo] = &[
            $($crate::TypeInfo::new::<$ty>(stringify!($ty)),)*
        ];
    };
    // Empty case
    ($name:ident: []) => {
        static $name: &'static [$crate::TypeInfo] = &[];
    };
}

/// Internal macro to extract metadata from plugin configuration.
/// This generates static metadata when the introspection feature is enabled.
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_metadata {
    // Entry point - initialize accumulators and start processing
    ($plugin_name:ident { $($config:tt)* }) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            // Accumulators: [resources] [messages] [states] [sub_states] [reflected] [plugins] [deps]
            //               [startup_systems] [update_systems] [fixed_systems] [on_enter_count] [on_exit_count]
            //               [version] [description]
            resources: [],
            messages: [],
            states: [],
            sub_states: [],
            reflected: [],
            plugins: [],
            deps: [],
            startup: [],
            update: [],
            fixed: [],
            on_enter: 0,
            on_exit: 0,
            version: None,
            description: None,
            config: { $($config)* }
        );
    };
}

/// Internal recursive macro for accumulating metadata from configuration
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_metadata_internal {
    // ========================================================================
    // Terminal case - generate the metadata structures
    // ========================================================================
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $version:expr,
        description: $description:expr,
        config: {}
    ) => {
        // Static arrays for type information
        #[cfg(feature = "introspection")]
        const _: () = {
            use $crate::{TypeInfo, PluginMetadata, PluginSystems, PluginInfo};

            static RESOURCES: &[TypeInfo] = &[
                $(TypeInfo::new::<$res>(stringify!($res)),)*
            ];

            static MESSAGES: &[TypeInfo] = &[
                $(TypeInfo::new::<$msg>(stringify!($msg)),)*
            ];

            static STATES: &[TypeInfo] = &[
                $(TypeInfo::new::<$state>(stringify!($state)),)*
            ];

            static SUB_STATES: &[TypeInfo] = &[
                $(TypeInfo::new::<$sub>(stringify!($sub)),)*
            ];

            static REFLECTED: &[TypeInfo] = &[
                $(TypeInfo::new::<$refl>(stringify!($refl)),)*
            ];

            static SUB_PLUGINS: &[&str] = &[
                $(stringify!($plug),)*
            ];

            static DEPENDENCIES: &[&str] = &[
                $(stringify!($dep),)*
            ];

            static STARTUP_SYSTEMS: &[&str] = &[
                $(stringify!($startup_sys),)*
            ];

            static UPDATE_SYSTEMS: &[&str] = &[
                $(stringify!($update_sys),)*
            ];

            static FIXED_SYSTEMS: &[&str] = &[
                $(stringify!($fixed_sys),)*
            ];

            static METADATA: PluginMetadata = PluginMetadata {
                name: stringify!($plugin_name),
                version: $version,
                description: $description,
                resources: RESOURCES,
                messages: MESSAGES,
                states: STATES,
                sub_states: SUB_STATES,
                reflected_types: REFLECTED,
                sub_plugins: SUB_PLUGINS,
                dependencies: DEPENDENCIES,
                systems: PluginSystems {
                    startup: STARTUP_SYSTEMS,
                    update: UPDATE_SYSTEMS,
                    fixed_update: FIXED_SYSTEMS,
                    on_enter_count: $on_enter_count,
                    on_exit_count: $on_exit_count,
                },
            };

            impl PluginInfo for $plugin_name {
                const NAME: &'static str = stringify!($plugin_name);
                const VERSION: Option<&'static str> = $version;

                fn metadata() -> &'static PluginMetadata {
                    &METADATA
                }
            }
        };
    };

    // ========================================================================
    // Parsing cases - extract metadata from each configuration option
    // ========================================================================

    // meta: block with version and/or description
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $_old_ver:expr,
        description: $_old_desc:expr,
        config: { meta: { version: $ver:literal, description: $desc:literal } $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: Some($ver),
            description: Some($desc),
            config: { $($($rest)*)? }
        );
    };

    // meta: block with version only
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $_old_ver:expr,
        description: $desc:expr,
        config: { meta: { version: $ver:literal } $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: Some($ver),
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // meta: block with description only
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $_old_desc:expr,
        config: { meta: { description: $desc:literal } $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: Some($desc),
            config: { $($($rest)*)? }
        );
    };

    // Skip unknown meta formats
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { meta: { $($meta_contents:tt)* } $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // depends_on:
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($old_dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { depends_on: [$($dep:ty),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($old_dep,)* $($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // init_resource: / resources:
    ($plugin_name:ident,
        resources: [$($old_res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { init_resource: [$($res:ty),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($old_res,)* $($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // insert_resource: (skip - we can't easily get type from expr)
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { insert_resource: [$($resource:expr),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // add_message: / messages:
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($old_msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { add_message: [$($msg:ty),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($old_msg,)* $($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // add_plugins: / plugins:
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($old_plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { add_plugins: [$($plug:expr),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($old_plug,)* $($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // init_state: / states:
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($old_state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { init_state: [$($state:ty),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($old_state,)* $($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // add_sub_state: / sub_states:
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($old_sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { add_sub_state: [$($sub:ty),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($old_sub,)* $($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // register_type: / reflect:
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($old_refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { register_type: [$($refl:ty),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($old_refl,)* $($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // add_systems_startup: / startup:
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($old_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { add_systems_startup: [$($sys:expr),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($old_sys,)* $($sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // add_systems_update: / update:
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($old_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { add_systems_update: [$($sys:expr),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($old_sys,)* $($sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // add_systems_fixed_update: / fixed_update:
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($old_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { add_systems_fixed_update: [$($sys:expr),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($old_sys,)* $($sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // add_systems_on_enter: / on_enter: (count entries for metadata)
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { add_systems_on_enter: { $($state_val:expr => [$($sys:expr),* $(,)?]),* $(,)? } $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count + $crate::count_items!($($($sys),*),*),
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // add_systems_on_exit: / on_exit:
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { add_systems_on_exit: { $($state_val:expr => [$($sys:expr),* $(,)?]),* $(,)? } $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count + $crate::count_items!($($($sys),*),*),
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // custom_build: / custom_init: (skip for metadata)
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { custom_build: $build_fn:expr $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // custom_finish: (skip for metadata)
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { custom_finish: $finish_fn:expr $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // generate_tests: (skip for metadata, handled by separate macro)
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { generate_tests: { $($test_config:tt)* } $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };

    // Catch-all for unknown options - skip them silently for metadata
    // (the main macro will report errors for truly unknown options)
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        sub_states: [$($sub:ty),*],
        reflected: [$($refl:ty),*],
        plugins: [$($plug:expr),*],
        deps: [$($dep:ty),*],
        startup: [$($startup_sys:expr),*],
        update: [$($update_sys:expr),*],
        fixed: [$($fixed_sys:expr),*],
        on_enter: $on_enter_count:expr,
        on_exit: $on_exit_count:expr,
        version: $ver:expr,
        description: $desc:expr,
        config: { $unknown:ident : $value:tt $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_metadata_internal!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            sub_states: [$($sub),*],
            reflected: [$($refl),*],
            plugins: [$($plug),*],
            deps: [$($dep),*],
            startup: [$($startup_sys),*],
            update: [$($update_sys),*],
            fixed: [$($fixed_sys),*],
            on_enter: $on_enter_count,
            on_exit: $on_exit_count,
            version: $ver,
            description: $desc,
            config: { $($($rest)*)? }
        );
    };
}

// ============================================================================
// Test Generation (feature-gated)
// ============================================================================

/// Internal macro to generate tests for a plugin.
/// This is a no-op unless the plugin has a generate_tests: block.
/// The testing feature must be enabled for tests to be generated.
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_tests {
    // Entry point - scan for generate_tests: block
    ($plugin_name:ident { $($config:tt)* }) => {
        $crate::define_plugin_tests_scan!($plugin_name, config: { $($config)* });
    };
}

/// Scanner macro that looks for generate_tests: block
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_tests_scan {
    // Found generate_tests: block - pass to generator
    ($plugin_name:ident, config: { generate_tests: { $($test_opts:tt)* } $(, $($rest:tt)*)? }) => {
        $crate::define_plugin_tests_generate!($plugin_name, test_opts: { $($test_opts)* }, config: { $($($rest)*)? });
    };

    // Skip other configs and keep looking
    ($plugin_name:ident, config: { $key:ident : [$($value:tt)*] $(, $($rest:tt)*)? }) => {
        $crate::define_plugin_tests_scan!($plugin_name, config: { $($($rest)*)? });
    };
    ($plugin_name:ident, config: { $key:ident : { $($value:tt)* } $(, $($rest:tt)*)? }) => {
        $crate::define_plugin_tests_scan!($plugin_name, config: { $($($rest)*)? });
    };
    // Handle closures like custom_init: |app| { ... } followed by more config
    ($plugin_name:ident, config: { $key:ident : | $param:ident $(: $param_ty:ty)? | { $($body:tt)* } $(, $($rest:tt)*)? }) => {
        $crate::define_plugin_tests_scan!($plugin_name, config: { $($($rest)*)? });
    };
    // Handle closures as trailing item (no comma after)
    ($plugin_name:ident, config: { $key:ident : | $param:ident $(: $param_ty:ty)? | { $($body:tt)* } }) => {
        // No more config - no generate_tests found
    };
    ($plugin_name:ident, config: { $key:ident : $value:expr $(, $($rest:tt)*)? }) => {
        $crate::define_plugin_tests_scan!($plugin_name, config: { $($($rest)*)? });
    };

    // End of config - no generate_tests: found, do nothing
    ($plugin_name:ident, config: {}) => {};
}

/// Generator macro that creates test based on test_opts
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_tests_generate {
    // Entry - start accumulating types
    ($plugin_name:ident, test_opts: { $($test_opts:tt)* }, config: { $($config:tt)* }) => {
        $crate::define_plugin_tests_accumulate!(
            $plugin_name,
            test_opts: { $($test_opts)* },
            resources: [],
            messages: [],
            states: [],
            deps: [],
            config: { $($config)* }
        );
    };
}

/// Accumulator that collects types from config for test generation
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_tests_accumulate {
    // Terminal - generate tests
    ($plugin_name:ident,
        test_opts: { $($test_opts:tt)* },
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($dep:ty),*],
        config: {}
    ) => {
        $crate::define_plugin_tests_emit!(
            $plugin_name,
            test_opts: { $($test_opts)* },
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            deps: [$($dep),*]
        );
    };

    // init_resource:
    ($plugin_name:ident,
        test_opts: { $($test_opts:tt)* },
        resources: [$($old_res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($dep:ty),*],
        config: { init_resource: [$($res:ty),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_tests_accumulate!(
            $plugin_name,
            test_opts: { $($test_opts)* },
            resources: [$($old_res,)* $($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            deps: [$($dep),*],
            config: { $($($rest)*)? }
        );
    };

    // add_message:
    ($plugin_name:ident,
        test_opts: { $($test_opts:tt)* },
        resources: [$($res:ty),*],
        messages: [$($old_msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($dep:ty),*],
        config: { add_message: [$($msg:ty),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_tests_accumulate!(
            $plugin_name,
            test_opts: { $($test_opts)* },
            resources: [$($res),*],
            messages: [$($old_msg,)* $($msg),*],
            states: [$($state),*],
            deps: [$($dep),*],
            config: { $($($rest)*)? }
        );
    };

    // init_state:
    ($plugin_name:ident,
        test_opts: { $($test_opts:tt)* },
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($old_state:ty),*],
        deps: [$($dep:ty),*],
        config: { init_state: [$($state:ty),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_tests_accumulate!(
            $plugin_name,
            test_opts: { $($test_opts)* },
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($old_state,)* $($state),*],
            deps: [$($dep),*],
            config: { $($($rest)*)? }
        );
    };

    // depends_on:
    ($plugin_name:ident,
        test_opts: { $($test_opts:tt)* },
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($old_dep:ty),*],
        config: { depends_on: [$($dep:ty),* $(,)?] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_tests_accumulate!(
            $plugin_name,
            test_opts: { $($test_opts)* },
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            deps: [$($old_dep,)* $($dep),*],
            config: { $($($rest)*)? }
        );
    };

    // Skip other options
    ($plugin_name:ident,
        test_opts: { $($test_opts:tt)* },
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($dep:ty),*],
        config: { $key:ident : [$($value:tt)*] $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_tests_accumulate!(
            $plugin_name,
            test_opts: { $($test_opts)* },
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            deps: [$($dep),*],
            config: { $($($rest)*)? }
        );
    };
    ($plugin_name:ident,
        test_opts: { $($test_opts:tt)* },
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($dep:ty),*],
        config: { $key:ident : { $($value:tt)* } $(, $($rest:tt)*)? }
    ) => {
        $crate::define_plugin_tests_accumulate!(
            $plugin_name,
            test_opts: { $($test_opts)* },
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            deps: [$($dep),*],
            config: { $($($rest)*)? }
        );
    };
    ($plugin_name:ident,
        test_opts: { $($test_opts:tt)* },
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($dep:ty),*],
        config: { $key:ident : | $($value:tt)* }
    ) => {
        // Handle trailing closure - no more config after this
        $crate::define_plugin_tests_accumulate!(
            $plugin_name,
            test_opts: { $($test_opts)* },
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            deps: [$($dep),*],
            config: {}
        );
    };
}

/// Emit the actual test code based on test_opts
/// Tests are generated in a module named after the plugin to ensure unique test names
/// and proper test discovery by the test harness.
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_tests_emit {
    // Collect all test flags and emit a single module with all tests
    ($plugin_name:ident,
        test_opts: { $($opt_key:ident : $opt_val:tt),* $(,)? },
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($dep:ty),*]
    ) => {
        $crate::define_plugin_tests_emit_module!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            deps: [$($dep),*],
            test_resources: false,
            test_messages: false,
            test_states: false,
            test_dependencies: false
            $(, $opt_key : $opt_val)*
        );
    };

    // Terminal - empty test_opts
    ($plugin_name:ident,
        test_opts: {},
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($dep:ty),*]
    ) => {};
}

/// Helper macro to emit the test module with accumulated flags
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_tests_emit_module {
    // Base case - emit the module
    // We wrap in const _: () to avoid name collision between the module and struct
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($dep:ty),*],
        test_resources: $test_res:tt,
        test_messages: $test_msg:tt,
        test_states: $test_states:tt,
        test_dependencies: $test_deps:tt
    ) => {
        // Generate test module wrapped in const to avoid name collision with struct
        #[cfg(all(test, feature = "testing"))]
        const _: () = {
            #[allow(non_snake_case)]
            mod tests {
                // Import from two levels up (through const, then through parent module)
                #[allow(unused_imports)]
                use super::super::*;

                $crate::define_plugin_test_resource!($plugin_name, $test_res, [$($res),*]);
                $crate::define_plugin_test_messages!($plugin_name, $test_msg, [$($msg),*]);
                $crate::define_plugin_test_states!($plugin_name, $test_states, [$($state),*]);
                $crate::define_plugin_test_dependencies!($plugin_name, $test_deps, [$($dep),*]);
            }
        };
    };

    // Override test_resources
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($dep:ty),*],
        test_resources: $_old:tt,
        test_messages: $test_msg:tt,
        test_states: $test_states:tt,
        test_dependencies: $test_deps:tt,
        test_resources: $new_val:tt
        $(, $rest_key:ident : $rest_val:tt)*
    ) => {
        $crate::define_plugin_tests_emit_module!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            deps: [$($dep),*],
            test_resources: $new_val,
            test_messages: $test_msg,
            test_states: $test_states,
            test_dependencies: $test_deps
            $(, $rest_key : $rest_val)*
        );
    };

    // Override test_messages
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($dep:ty),*],
        test_resources: $test_res:tt,
        test_messages: $_old:tt,
        test_states: $test_states:tt,
        test_dependencies: $test_deps:tt,
        test_messages: $new_val:tt
        $(, $rest_key:ident : $rest_val:tt)*
    ) => {
        $crate::define_plugin_tests_emit_module!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            deps: [$($dep),*],
            test_resources: $test_res,
            test_messages: $new_val,
            test_states: $test_states,
            test_dependencies: $test_deps
            $(, $rest_key : $rest_val)*
        );
    };

    // Override test_states
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($dep:ty),*],
        test_resources: $test_res:tt,
        test_messages: $test_msg:tt,
        test_states: $_old:tt,
        test_dependencies: $test_deps:tt,
        test_states: $new_val:tt
        $(, $rest_key:ident : $rest_val:tt)*
    ) => {
        $crate::define_plugin_tests_emit_module!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            deps: [$($dep),*],
            test_resources: $test_res,
            test_messages: $test_msg,
            test_states: $new_val,
            test_dependencies: $test_deps
            $(, $rest_key : $rest_val)*
        );
    };

    // Override test_dependencies
    ($plugin_name:ident,
        resources: [$($res:ty),*],
        messages: [$($msg:ty),*],
        states: [$($state:ty),*],
        deps: [$($dep:ty),*],
        test_resources: $test_res:tt,
        test_messages: $test_msg:tt,
        test_states: $test_states:tt,
        test_dependencies: $_old:tt,
        test_dependencies: $new_val:tt
        $(, $rest_key:ident : $rest_val:tt)*
    ) => {
        $crate::define_plugin_tests_emit_module!(
            $plugin_name,
            resources: [$($res),*],
            messages: [$($msg),*],
            states: [$($state),*],
            deps: [$($dep),*],
            test_resources: $test_res,
            test_messages: $test_msg,
            test_states: $test_states,
            test_dependencies: $new_val
            $(, $rest_key : $rest_val)*
        );
    };
}

/// Generate resource tests if enabled
/// Note: Tests are generated inside const _: () = { mod tests { ... } }
/// so we need super::super to reach the plugin type
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_test_resource {
    ($plugin_name:ident, true, [$($res:ty),+]) => {
        #[test]
        fn test_resources() {
            let mut app = ::bevy::prelude::App::new();
            app.add_plugins(super::super::$plugin_name);
            $(
                assert!(
                    app.world().contains_resource::<$res>(),
                    concat!(stringify!($plugin_name), " should initialize resource: ", stringify!($res))
                );
            )+
        }
    };
    ($plugin_name:ident, true, []) => {};  // No resources to test
    ($plugin_name:ident, false, [$($res:ty),*]) => {};  // Testing disabled
}

/// Generate message tests if enabled
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_test_messages {
    ($plugin_name:ident, true, [$($msg:ty),+]) => {
        #[test]
        fn test_messages() {
            let mut app = ::bevy::prelude::App::new();
            app.add_plugins(super::super::$plugin_name);
            $(
                assert!(
                    app.world().contains_resource::<::bevy::prelude::Messages<$msg>>(),
                    concat!(stringify!($plugin_name), " should register message: ", stringify!($msg))
                );
            )+
        }
    };
    ($plugin_name:ident, true, []) => {};  // No messages to test
    ($plugin_name:ident, false, [$($msg:ty),*]) => {};  // Testing disabled
}

/// Generate state tests if enabled
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_test_states {
    ($plugin_name:ident, true, [$($state:ty),+]) => {
        #[test]
        fn test_states() {
            let mut app = ::bevy::prelude::App::new();
            app.add_plugins(::bevy::state::app::StatesPlugin);
            app.add_plugins(super::super::$plugin_name);
            $(
                assert!(
                    app.world().contains_resource::<::bevy::prelude::State<$state>>(),
                    concat!(stringify!($plugin_name), " should initialize state: ", stringify!($state))
                );
            )+
        }
    };
    ($plugin_name:ident, true, []) => {};  // No states to test
    ($plugin_name:ident, false, [$($state:ty),*]) => {};  // Testing disabled
}

/// Generate dependency tests if enabled
#[macro_export]
#[doc(hidden)]
macro_rules! define_plugin_test_dependencies {
    ($plugin_name:ident, true, [$($dep:ty),+]) => {
        #[test]
        #[should_panic(expected = "requires")]
        fn test_dependencies_panic_when_missing() {
            let mut app = ::bevy::prelude::App::new();
            // Intentionally not adding dependencies - should panic
            app.add_plugins(super::super::$plugin_name);
        }
    };
    ($plugin_name:ident, true, []) => {}; // No dependencies to test
    ($plugin_name:ident, false, [$($dep:ty),*]) => {}; // Testing disabled
}

// The macro is exported at crate root via #[macro_export]
