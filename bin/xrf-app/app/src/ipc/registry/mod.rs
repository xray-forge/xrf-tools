//! The command surface every plugin crate declares, as the application composes it.
//!
//! Each crate pairs its wire names with Rust command paths in its own `registry/domains.rs` and expands that list into
//! its dispatch handlers, Specta collections and routes (`xrf_app_core::registry`). `build.rs` expands every crate's list
//! into the inline plugin and ACL declarations the build script needs; this module joins their transport routes.

use tauri::AppHandle;
use xrf_app_core::transport::TransportRoutes;

#[cfg(test)]
mod tests;

/// Every route each plugin crate's domains serve, as the transport serves them.
pub(crate) fn transport_routes() -> TransportRoutes<AppHandle> {
  TransportRoutes::new(
    [
      xrf_app_core::registry::list_routes(),
      xrf_app_render::registry::list_routes(),
      xrf_app_archives::registry::list_routes(),
      xrf_app_configs::registry::list_routes(),
      xrf_app_dialogs::registry::list_routes(),
      xrf_app_exports::registry::list_routes(),
      xrf_app_gamedata::registry::list_routes(),
      xrf_app_levels::registry::list_routes(),
      xrf_app_spawn::registry::list_routes(),
      xrf_app_sprite_equipment::registry::list_routes(),
      xrf_app_system::registry::list_routes(),
      xrf_app_textures::registry::list_routes(),
      xrf_app_translations::registry::list_routes(),
      xrf_app_visuals::registry::list_routes(),
    ]
    .into_iter()
    .flatten()
    .collect(),
  )
}
