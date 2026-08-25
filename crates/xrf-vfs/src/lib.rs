#![doc = include_str!("../README.md")]
//!
//! # Module map
//!
//! The crate is grouped by the question each part answers:
//!
//! - [`path`] — what an engine identity is, and the only place separators and case are decided.
//! - [`asset`] — what a resolved asset is, plus the per-kind rules that turn a reference into one.
//! - [`cache`] — what a mounted world keeps after parsing an asset, and the policy deciding what is kept.
//! - [`source`] — the mountable surface, and the two sources the engine itself has.
//! - [`mount`] — composing sources into a searchable order, and planning one from a path.
//! - [`vfs`] — resolving and reading through that order, one scope at a time or as an ordered probe.
//! - [`fsgame`] — the declaration file an installation describes its own layout with.
//!
//! The `.db` volume format the archive source reads lives below this crate, in `xrf-archive`.

pub mod asset;
pub mod cache;
pub mod fsgame;
pub mod mount;
pub mod path;
pub mod source;
pub mod vfs;

pub use asset::{XrayAsset, XrayAssetContainer, XrayAssetRules, XrayAssetType, require_writable_path};
pub use cache::{XrayAssetCache, XrayCachePolicy, XrayCacheStats};
pub use fsgame::{FsgameDeclaration, FsgameFile};
pub use mount::{
  XrayMount, XrayMountId, XrayMountMode, XrayMountPlan, XrayPlannedMount, XrayProbePlan, XrayRoot, XrayRoots,
  XraySkippedMount,
};
pub use path::{XrayLogicalPath, XrayPathCollision};
pub use source::{XrayArchiveSource, XrayAssetSource, XraySourceKind};
pub use vfs::{
  XrayDirectoryListing, XrayLookupScope, XrayProbe, XrayProbeStep, XrayResolution, XrayScopedVfs, XrayVfs,
};
