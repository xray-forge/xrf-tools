//! Reading one texture: how it is named, the two files it is, everything the panels say about it, and the names the
//! SDK gives the numbers its descriptor stores.

pub(crate) mod commands;
mod description;
pub(crate) mod routes;
mod vocabulary;

pub use description::{TextureDescription, TextureFiles, TextureSource};
pub use vocabulary::TextureVocabulary;
