//! Compiling translation sources into the per-language string tables the game loads.

pub(crate) mod compile;
pub(crate) mod targets;
pub(crate) mod translation_build_options;
pub(crate) mod translation_build_result;
pub(crate) mod translation_builder;

#[cfg(test)]
mod tests;
