//! Browsing a root set: every texture it holds, listed by reference or by path, and what each descriptor makes of its
//! texture for the badges a listing carries.

mod catalog;
pub(crate) mod commands;
mod summary;

pub use catalog::{TextureBrowses, TextureCatalog, TextureCatalogMode, TextureEntry, TextureRole};
pub use summary::{TextureBadges, TextureBumpPair, TextureMaterialSummary};
