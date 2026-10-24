//! Command registration, derived from token lists each plugin crate keeps.
//!
//! A plugin crate declares its command domains once, in its `registry/domains.rs`: a macro pairing every wire name with
//! its Rust command path, for typed commands, raw commands answering bytes over IPC, and bulk routes answering bytes over
//! the loopback transport. The crate expands it through [`define_runtime_domains`] into a module per domain, carrying its
//! dispatch handler, plugin name, Specta collection and routes; the application's build script expands every crate's
//! list into the inline plugin and ACL declarations, so a domain's wire names and permissions come from one place.

#[macro_use]
mod domains;
#[macro_use]
mod runtime;

#[cfg(test)]
mod tests;

for_each_core_domain!(define_runtime_domains);
