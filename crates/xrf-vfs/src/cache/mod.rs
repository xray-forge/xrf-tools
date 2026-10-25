//! What a VFS snapshot keeps after parsing an asset, the policy deciding what is worth keeping, and the coordination
//! that keeps two readers from parsing one asset twice.

#[cfg(test)]
mod tests;
mod xray_asset_cache;
mod xray_cache_entries;
mod xray_cache_entry;
mod xray_cache_flight;
mod xray_cache_flight_claim;
mod xray_cache_flight_guard;
mod xray_cache_flights;
mod xray_cache_key;
mod xray_cache_policy;
mod xray_cache_stats;
mod xray_stamped_copy;

pub(crate) use xray_asset_cache::XrayAssetCache;
pub(crate) use xray_cache_key::XrayCacheKey;
pub use xray_cache_policy::XrayCachePolicy;
pub use xray_cache_stats::XrayCacheStats;
pub(crate) use xray_stamped_copy::XrayStampedCopy;
