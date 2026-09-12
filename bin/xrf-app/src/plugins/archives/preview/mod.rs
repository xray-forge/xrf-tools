//! Decoding one asset for display.
//!
//! Addressed by `XrayRoots` rather than by a session, so these serve either browsing subject without belonging to
//! one: the roots travel with the request and the mounts are the application's shared ones.

pub mod commands;
