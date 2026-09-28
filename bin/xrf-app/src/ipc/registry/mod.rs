//! Command registration, derived from one token list.
//!
//! `domains.rs` pairs every wire name with its Rust command path: typed commands, raw commands answering bytes over
//! IPC, and bulk routes answering bytes over the loopback transport. `runtime.rs` expands that list into a module per
//! domain, re-exported here so a domain reads as `crate::ipc::registry::<domain>`, and into the transport's route
//! table; `build.rs` expands the same list into the inline plugin and ACL declarations the build script needs.

#[macro_use]
mod domains;
// Visible to `tests`, which expands a fixture domain through the same adapter.
#[cfg_attr(test, macro_use)]
mod runtime;

pub(crate) use runtime::*;

#[cfg(test)]
mod tests;
