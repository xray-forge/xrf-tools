//! Desktop integration that belongs to no editor in particular, grouped by the question each surface asks.
//!
//! - [`paths`] - filesystem paths as the application addresses them: what one holds, where it can be shown, and where
//!   tools write when nobody named a destination.
//! - [`diagnostics`] - what this instance is and what it costs: which build, on which machine, using how much of it.
//! - [`window`] - the window as its page sees it: a close the page asked about first and agreed to, and what it decided.
pub mod diagnostics;
pub mod paths;
pub mod plugin;
pub mod registry;
pub mod window;
