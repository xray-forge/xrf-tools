//! String table XML: how one file is decoded, read, and edited in place.
//!
//! Reading and writing sit together because they share the encoding rules and the parser. Nothing in
//! here knows about project layouts; it works on one file at a time.

pub(crate) mod decoded_translation;
pub(crate) mod encoding;
pub(crate) mod layout;
pub(crate) mod read;
pub(crate) mod translation_compiled_xml;
pub(crate) mod translation_entry_compiled;
pub(crate) mod translation_identity;
pub(crate) mod write;

#[cfg(test)]
mod tests;
