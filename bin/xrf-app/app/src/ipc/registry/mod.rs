//! The command surface every plugin crate declares, as the application composes it.
//!
//! Each crate pairs its wire names with Rust command paths in its own `ipc/domains.rs` and expands that list into
//! its plugin builders, dispatch handlers, Specta collections, routes and job kinds (`xrf_app_core::ipc`).
//! `build.rs` expands every crate's list into the inline plugin and ACL declarations the build script needs; this module
//! joins their transport routes.

use tauri::AppHandle;
use xrf_app_core::transport::TransportRoutes;

#[cfg(test)]
mod tests;

/// Every route each plugin crate's domains serve, as the transport serves them.
pub(crate) fn transport_routes() -> TransportRoutes<AppHandle> {
  macro_rules! collect_routes {
    ($($krate:ident),*) => {
      TransportRoutes::new([$($krate::ipc::list_routes(),)*].into_iter().flatten().collect())
    };
  }

  for_each_plugin_crate!(collect_routes)
}
