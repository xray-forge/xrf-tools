//! Evaluation functions, the pattern tables the AI scores its choices with.
//!
//! A `.efd` is flat rather than chunked, and its trailing parameter block is sized by a number the file never
//! stores: the sum over the patterns of the product of their inputs ranges.

pub(crate) mod efd_file;
pub(crate) mod efd_pattern;
pub(crate) mod efd_variable;

#[cfg(test)]
mod tests;
