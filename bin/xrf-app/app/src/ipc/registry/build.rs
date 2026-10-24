// The build script's side of the registry: every plugin crate's command domains, included by `build.rs` as macros,
// declared as inline plugins with their permissions.

use std::path::PathBuf;
use std::{env, fs};

/// A crate's command domains as `(plugin name, inline plugin)`, its commands permitted by default.
macro_rules! inline_plugins {
  (
    $(
      $domain:ident => $plugin_name:literal {
        $($command_name:ident => $command_head:ident $(:: $command_tail:ident)*,)*
      }
      $(@raw {
        $($raw_name:ident ( $($raw_arg:ident : $raw_arg_type:literal),* $(,)? )
          => $raw_head:ident $(:: $raw_tail:ident)*,)*
      })?
      $(@bulk {
        $($route_name:ident ( $($route_arg:ident : $route_arg_type:literal),* $(,)? )
          => $route_head:ident $(:: $route_tail:ident)*,)*
      })?
    )*
  ) => {
    vec![
      $((
        $plugin_name,
        tauri_build::InlinedPlugin::new()
          // Raw commands are permitted like any other: the ACL governs dispatch, and only the Specta collection
          // excludes them. Bulk routes are not commands: the transport serves them, behind its own token, so the
          // ACL never sees them.
          .commands(&[
            $(stringify!($command_name),)*
            $($(stringify!($raw_name),)*)?
          ])
          .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands),
      ),)*
    ]
  };
}

/// Declares every plugin crate's domains as inline plugins of the application, and lists their names in
/// `OUT_DIR/inline_plugins.rs` (`INLINE_PLUGINS`), against which the app's tests check every list it keeps by hand.
pub fn configure(attributes: tauri_build::Attributes) -> tauri_build::Attributes {
  let plugins: Vec<Vec<(&str, tauri_build::InlinedPlugin)>> = vec![
    for_each_core_domain!(inline_plugins),
    for_each_render_domain!(inline_plugins),
    for_each_archives_domain!(inline_plugins),
    for_each_configs_domain!(inline_plugins),
    for_each_dialogs_domain!(inline_plugins),
    for_each_exports_domain!(inline_plugins),
    for_each_gamedata_domain!(inline_plugins),
    for_each_levels_domain!(inline_plugins),
    for_each_spawn_domain!(inline_plugins),
    for_each_sprite_equipment_domain!(inline_plugins),
    for_each_system_domain!(inline_plugins),
    for_each_textures_domain!(inline_plugins),
    for_each_translations_domain!(inline_plugins),
    for_each_visuals_domain!(inline_plugins),
  ];

  let names: Vec<&str> = plugins.iter().flatten().map(|(name, _)| *name).collect();
  let listed: PathBuf =
    PathBuf::from(env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR")).join("inline_plugins.rs");

  fs::write(
    &listed,
    format!(
      "/// Every inline plugin the build script declared, by name.\nconst INLINE_PLUGINS: &[&str] = &{names:?};\n"
    ),
  )
  .unwrap_or_else(|error| panic!("Failed to write {}: {error}", listed.display()));

  plugins
    .into_iter()
    .flatten()
    .fold(attributes, |attributes, (name, plugin)| attributes.plugin(name, plugin))
}
