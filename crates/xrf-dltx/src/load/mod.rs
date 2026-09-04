//! What a config tree declares, recorded without deciding any of it.
//!
//! The engine fills the same set of tables in one walk and resolves from them afterwards, which is what lets a mod file
//! patch a section declared anywhere. Deciding is [`crate::resolve`]'s half.

pub(crate) mod dltx_item;
pub(crate) mod dltx_load_result;
pub(crate) mod dltx_loader;
pub(crate) mod dltx_logical_path;
