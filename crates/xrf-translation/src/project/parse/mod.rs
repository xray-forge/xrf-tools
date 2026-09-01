//! Importing raw X-Ray string tables into the JSON sources the project authors.

pub(crate) mod merge;
pub(crate) mod scope;
pub(crate) mod translation_parse_options;
pub(crate) mod translation_parse_result;
pub(crate) mod translation_parser;

#[cfg(test)]
mod tests;
