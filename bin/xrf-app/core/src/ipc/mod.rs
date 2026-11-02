//! Command registration, derived from token lists each plugin crate keeps.
//!
//! A plugin crate declares its command domains once, in its `ipc/domains.rs`: a macro pairing every wire name with
//! its Rust command path, for typed commands, raw commands answering bytes over IPC, and bulk routes answering bytes over
//! the loopback transport, and naming the kinds of job the domain runs. The crate expands it through
//! [`define_runtime_domains!`](crate::define_runtime_domains) into a module per domain, carrying its plugin builder, dispatch handler, plugin name,
//! Specta collection, bindings surface, routes and job kinds; the application's build script expands every crate's list
//! into the inline plugin and ACL declarations, so a domain's wire names and permissions come from one place.

#[macro_use]
mod domains;
#[macro_use]
mod runtime;
mod unhandled_command;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub use unhandled_command::warn_on_unhandled_command;
#[cfg(feature = "typescript-bindings")]
pub use xrf_ipc_typescript::IpcCommandSurface;

for_each_core_domain!(define_runtime_domains);
