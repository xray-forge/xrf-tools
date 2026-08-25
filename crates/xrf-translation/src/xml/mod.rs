//! String table XML: how one file is decoded, read, and edited in place.
//!
//! Reading and writing sit together because they share the encoding rules and the parser. Nothing in
//! here knows about project layouts; it works on one file at a time.

pub(crate) mod compiled;
pub(crate) mod constants;
pub(crate) mod encoding;
pub(crate) mod layout;
pub(crate) mod read;
pub(crate) mod write;

pub(crate) use constants::{FILE_EXTENSION, FILE_EXTENSION_DOT};

#[cfg(test)]
mod tests;
