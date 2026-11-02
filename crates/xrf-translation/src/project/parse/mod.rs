//! Importing raw X-Ray string tables into the JSON sources the project authors.

pub(crate) mod resolved_parse_scope;
pub(crate) mod translation_merge_outcome;
pub(crate) mod translation_merger;
pub(crate) mod translation_parse_census;
pub(crate) mod translation_parse_options;
pub(crate) mod translation_parse_result;
pub(crate) mod translation_parser;

#[cfg(test)]
mod tests;
