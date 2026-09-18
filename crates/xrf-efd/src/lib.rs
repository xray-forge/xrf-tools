//! `.efd` evaluation function data, the pattern tables the game evaluates.

pub(crate) mod efd_file;
pub(crate) mod efd_pattern;
pub(crate) mod efd_variable;

#[cfg(test)]
mod tests;

pub use crate::efd_file::*;
pub use crate::efd_pattern::*;
pub use crate::efd_variable::*;
