//! Every archive surface of the application, grouped by the question each answers.
//!
//! One plugin rather than five, because they share a wire namespace and a publication lease; five directories rather
//! than one flat `commands/`, because reading thirty commands in registry order says nothing about which of them
//! belong together.
//!
//! - [`browse`] — what the explorer has open, and reading or extracting out of it.
//! - [`describe`] — saying in words what an entry the viewer cannot draw contains.
//! - [`preview`] — decoding one asset for display, addressed by roots rather than by a session.
//! - [`pack`] — writing a tree into volumes.
//! - [`patch`] — comparing two trees and publishing the difference.
//! - [`unpack`] — writing a volume set back out whole.

pub mod browse;
pub mod describe;
pub mod lease;
pub mod pack;
pub mod patch;
pub mod plugin;
pub mod preview;
pub mod unpack;
