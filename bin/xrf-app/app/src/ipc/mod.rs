//! The command surface the webview calls into, and everything derived from it.
//!
//! `registry` composes what every plugin crate declares: the inline plugins and their permissions at build time, and the
//! transport routes and job kinds at run time. `bindings` drives the same surface through `xrf-ipc-typescript` to write
//! the frontend's mirrors, and exists only in a test build because that is the harness it is run from.

/// Every plugin crate whose registry the application composes, written once for everything collected from them at run
/// time. The build script declares the same crates from their `domains.rs` files, since it compiles none of them; the
/// registry's tests hold both, and the plugins set up, to the inline plugins it declared.
macro_rules! for_each_plugin_crate {
  ($consumer:ident) => {
    $consumer!(
      xrf_app_core,
      xrf_app_render,
      xrf_app_archives,
      xrf_app_configs,
      xrf_app_dialogs,
      xrf_app_exports,
      xrf_app_gamedata,
      xrf_app_levels,
      xrf_app_spawn,
      xrf_app_sprite_equipment,
      xrf_app_system,
      xrf_app_textures,
      xrf_app_translations,
      xrf_app_visuals
    )
  };
}

#[cfg(all(test, feature = "typescript-bindings"))]
pub(crate) mod bindings;
pub(crate) mod registry;
