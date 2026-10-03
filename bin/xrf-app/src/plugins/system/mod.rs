//! Desktop integration that belongs to no editor in particular, grouped by the question each surface asks.
//!
//! - [`paths`] - filesystem paths as the application addresses them: what one holds, where it can be shown, and where
//!   tools write when nobody named a destination.
//! - [`diagnostics`] - what this instance is and what it costs: which build, on which machine, using how much of it.

pub mod diagnostics;
pub mod paths;
pub mod plugin;
