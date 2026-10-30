//! The application's caches: what rebuilds from nothing, kept between runs by client, marked used as it is read, and
//! tidied by age and size.

mod app_cache;
mod app_cache_policy;
mod app_cache_store;
mod app_cache_tidy;

#[cfg(test)]
mod tests;

pub use app_cache::AppCache;
pub use app_cache_policy::AppCachePolicy;
pub use app_cache_store::AppCacheStore;
pub use app_cache_tidy::AppCacheTidy;
