//! String table XML: how one file is decoded, read, and edited in place.
//!
//! Reading and writing sit together because they share the encoding rules and the parser. Nothing in
//! here knows about project layouts; it works on one file at a time.

pub(crate) mod decoded_translation;
pub(crate) mod translation_identity;
pub(crate) mod translation_line_limit;
pub(crate) mod translation_xml_reader;
pub(crate) mod translation_xml_writer;
pub(crate) mod xml_layout;

#[cfg(test)]
mod tests;
