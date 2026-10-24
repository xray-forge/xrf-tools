//! Browsing a root set: every texture it holds, listed by reference or by path, and what each descriptor makes of its
//! texture for the badges a listing carries.

pub(crate) mod commands;
mod summary;
mod texture_browse;
mod texture_browses;
mod texture_catalog;
mod texture_catalog_mode;
mod texture_entry;
mod texture_role;

pub use summary::{TextureBadges, TextureBumpPair, TextureMaterialSummary};
pub use texture_browse::TextureBrowse;
pub use texture_browses::TextureBrowses;
pub use texture_catalog::TextureCatalog;
pub use texture_catalog_mode::TextureCatalogMode;
pub use texture_entry::TextureEntry;
pub use texture_role::TextureRole;
