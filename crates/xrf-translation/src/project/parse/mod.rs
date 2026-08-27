//! Importing raw X-Ray XML string tables into multi-language JSON sources.
//!
//! One language per run, declared rather than discovered: a tree of raw XML carries its language in a
//! directory at best and nowhere at all at worst, and guessing it would stamp thousands of strings
//! with the wrong key. Running once per language merges them into one JSON per table, which is the
//! form everything else in this crate reads.

pub(crate) mod merge;
pub(crate) mod options;
pub(crate) mod result;
pub(crate) mod run;
pub(crate) mod scope;

#[cfg(test)]
mod tests;
