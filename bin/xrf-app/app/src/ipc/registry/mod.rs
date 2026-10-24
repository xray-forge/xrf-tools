//! The command surface every plugin crate declares, as the application composes it.
//!
//! Each crate pairs its wire names with Rust command paths in its own `ipc/domains.rs` and expands that list into
//! its plugin builders, dispatch handlers, Specta collections, routes and job kinds (`xrf_app_core::ipc`).
//! `build.rs` expands every crate's list into the inline plugin and ACL declarations the build script needs;
//! `transport_routes` joins their transport routes.

#[cfg(test)]
mod tests;
mod transport_routes;

pub(crate) use transport_routes::transport_routes;
