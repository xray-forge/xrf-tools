#![doc = include_str!("../README.md")]

pub(crate) mod app_cache;
pub(crate) mod app_cache_policy;
pub(crate) mod app_cache_store;
pub(crate) mod app_cache_tidy;
#[cfg(test)]
mod tests;

pub use crate::app_cache::AppCache;
pub use crate::app_cache_policy::{AppCachePolicy, DEFAULT_APP_CACHE_POLICY};
pub use crate::app_cache_store::AppCacheStore;
pub use crate::app_cache_tidy::AppCacheTidy;
