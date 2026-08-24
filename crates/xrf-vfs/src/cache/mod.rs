//! What a mounted world keeps after parsing an asset, and the policy deciding what is worth keeping.

#[cfg(test)]
mod tests;
mod xray_asset_cache;
mod xray_cache_policy;
mod xray_cache_stats;

pub use xray_asset_cache::XrayAssetCache;
pub use xray_cache_policy::XrayCachePolicy;
pub use xray_cache_stats::XrayCacheStats;
