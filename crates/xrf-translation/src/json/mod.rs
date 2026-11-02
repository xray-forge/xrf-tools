//! Multi-language JSON sources: the form the project authors translations in.

pub(crate) mod translation_json_canon;
pub(crate) mod translation_json_reader;
pub(crate) mod translation_json_render;
pub(crate) mod translation_json_writer;
pub(crate) mod translation_source_name;

#[cfg(test)]
mod tests;
