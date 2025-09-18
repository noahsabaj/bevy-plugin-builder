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
/// - `resources: [Type1, Type2]` - Initialize resources with `init_resource`
/// - `events: [Event1, Event2]` - Register events with `add_event`
/// - `plugins: [Plugin1, Plugin2]` - Add sub-plugins with `add_plugins`
/// - `states: [State1]` - Initialize states with `init_state`
/// - `sub_states: [SubState1]` - Add sub-states with `add_sub_state`
/// - `reflect: [Type1, Type2]` - Register types for reflection
/// - `startup: [system1, system2]` - Add startup systems
/// - `update: [system3, system4]` - Add update systems (supports conditions/ordering)
/// - `fixed_update: [system5]` - Add fixed update systems
/// - `on_enter: { State::Variant => [system6] }` - Add state enter systems
/// - `on_exit: { State::Variant => [system7] }` - Add state exit systems
/// - `custom_init: |app| { ... }` - Custom initialization logic
/// - `custom_finish: |app| { ... }` - Custom finish logic
///
/// ## Example
///
/// ```rust
/// use bevy_plugin_builder::define_plugin;
/// use bevy::prelude::*;
///
/// #[derive(Resource, Default)]
/// struct MyResource;
///
/// #[derive(Event)]
/// struct MyEvent;
///
/// fn setup_system() {}
/// fn update_system() {}
///
/// define_plugin!(MyPlugin {
///     resources: [MyResource],
///     events: [MyEvent],
///     startup: [setup_system],
///     update: [update_system]
/// });
/// ```
///
/// This generates:
///
/// ```rust
/// # use bevy::prelude::*;
/// # #[derive(Resource, Default)]
/// # struct MyResource;
/// # #[derive(Event)]
/// # struct MyEvent;
/// # fn setup_system() {}
/// # fn update_system() {}
/// pub struct MyPlugin;
///
/// impl Plugin for MyPlugin {
///     fn build(&self, app: &mut App) {
///         app.init_resource::<MyResource>()
///            .add_event::<MyEvent>()
///            .add_systems(Startup, setup_system)
///            .add_systems(Update, update_system);
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_plugin {
    // Main entry point - plugin with configuration block
    ($plugin_name:ident { $($config:tt)* }) => {
        pub struct $plugin_name;

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
macro_rules! define_plugin_internal {
    // Empty configuration (base case)
    ($app:ident,) => {};

    // Resources registration
    ($app:ident, resources: [$($resource:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $(
            $app.init_resource::<$resource>();
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // Events registration
    ($app:ident, events: [$($event:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $(
            $app.add_event::<$event>();
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // Sub-plugins registration
    ($app:ident, plugins: [$($plugin:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $(
            $app.add_plugins($plugin);
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // States registration
    ($app:ident, states: [$($state:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $(
            $app.init_state::<$state>();
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // Sub-states registration
    ($app:ident, sub_states: [$($state:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $(
            $app.add_sub_state::<$state>();
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // Reflection registration
    ($app:ident, reflect: [$($reflect_type:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $(
            $app.register_type::<$reflect_type>();
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // Startup systems
    ($app:ident, startup: [$($system:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $app.add_systems(
            ::bevy::prelude::Startup,
            ($($system,)*)
        );
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // Update systems (supports complex system expressions)
    ($app:ident, update: [$($system:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $app.add_systems(
            ::bevy::prelude::Update,
            ($($system,)*)
        );
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // FixedUpdate systems (supports complex system expressions)
    ($app:ident, fixed_update: [$($system:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $app.add_systems(
            ::bevy::prelude::FixedUpdate,
            ($($system,)*)
        );
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // OnEnter systems with state mapping
    ($app:ident, on_enter: { $($state:expr => [$($system:expr),* $(,)?]),* $(,)? } $(, $($rest:tt)*)?) => {
        $(
            $app.add_systems(
                ::bevy::prelude::OnEnter($state),
                ($($system,)*)
            );
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // OnExit systems with state mapping
    ($app:ident, on_exit: { $($state:expr => [$($system:expr),* $(,)?]),* $(,)? } $(, $($rest:tt)*)?) => {
        $(
            $app.add_systems(
                ::bevy::prelude::OnExit($state),
                ($($system,)*)
            );
        )*
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // Custom initialization (for complex setup)
    ($app:ident, custom_init: $init_fn:expr $(, $($rest:tt)*)?) => {
        $init_fn($app);
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // Custom finish handler (skip in build, handled in finish)
    ($app:ident, custom_finish: $finish_fn:expr $(, $($rest:tt)*)?) => {
        $crate::define_plugin_internal!($app, $($($rest)*)?);
    };

    // Error case - unrecognized configuration
    ($app:ident, $unknown:tt $($rest:tt)*) => {
        compile_error!(concat!(
            "Unknown plugin configuration option: ",
            stringify!($unknown),
            "\nSupported options: resources, events, plugins, states, sub_states, reflect, startup, update, fixed_update, on_enter, on_exit, custom_init, custom_finish"
        ));
    };
}

/// Macro for handling Plugin finish() method configuration
#[macro_export]
macro_rules! define_plugin_finish {
    // Empty configuration (base case) - default finish does nothing
    ($app:ident,) => {};

    // Skip all standard configurations (only process custom_finish)
    ($app:ident, resources: [$($resource:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, events: [$($event:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, plugins: [$($plugin:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, startup: [$($system:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, update: [$($system:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, fixed_update: [$($system:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, on_enter: { $($state:expr => [$($system:expr),* $(,)?]),* $(,)? } $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, on_exit: { $($state:expr => [$($system:expr),* $(,)?]),* $(,)? } $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, custom_init: $init_fn:expr $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, states: [$($state:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, sub_states: [$($state:ty),* $(,)?] $(, $($rest:tt)*)?) => {
        $crate::define_plugin_finish!($app, $($($rest)*)?);
    };
    ($app:ident, reflect: [$($reflect_type:ty),* $(,)?] $(, $($rest:tt)*)?) => {
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

// The macro is exported at crate root via #[macro_export]
