//! Reading one texture: how it is named, the two files it is, everything the panels say about it, and the names the
//! SDK gives the numbers its descriptor stores.

pub(crate) mod commands;
pub(crate) mod routes;
mod texture_description;
mod texture_files;
mod texture_source;
mod vocabulary;

pub use texture_description::TextureDescription;
pub use texture_files::TextureFiles;
pub use texture_source::TextureSource;
pub use vocabulary::{TextureFlagEntry, TextureVocabulary, TextureVocabularyEntry};
