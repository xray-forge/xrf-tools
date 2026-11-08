//! What the application keeps in its own data directory, by kind, and clearing what may be rebuilt.

mod app_storage_kind;
mod app_storage_usage;

pub(crate) use app_storage_kind::AppStorageKind;
pub(crate) use app_storage_usage::AppStorageUsage;
