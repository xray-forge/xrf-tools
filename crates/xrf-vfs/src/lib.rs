#![doc = include_str!("../README.md")]
//!
//! # Module map
//!
//! The crate is grouped by the question each part answers; every module is private and its supported names are
//! re-exported here:
//!
//! - `path` — what an engine identity is, and the only place separators and case are decided.
//! - `asset` — what a resolved asset is, plus the per-kind rules that turn a reference into one.
//! - `fsgame` — the declaration file an installation describes its own layout with.
//! - `source` — the mountable surface, and the two sources the engine itself has.
//! - `mount` — composing sources into a mounted order, and planning one from a path.
//! - `search` — how a VFS snapshot is read: the ordered search, its steps and what its lookups answer.
//! - `vfs` — the VFS snapshot itself (mounts, revisions, what it retains) and the host owning the current one.
//! - `change` — changing what a snapshot holds: a transaction committed to disk, or changes already there applied.
//! - `input` — which inputs a recording search resolved, and the revision each had then.
//! - `cache` — what a snapshot keeps after parsing an asset, the policy deciding what is kept, and the coordination
//!   that keeps two readers from parsing one asset twice.
//! - `trace` — what a snapshot actually read, when a caller asked to be told.
//!
//! The `.db` volume format the archive source reads lives below this crate, in `xrf-archive`.

mod asset;
mod cache;
mod change;
mod fsgame;
mod input;
mod mount;
mod path;
mod search;
mod source;
mod trace;
mod vfs;

pub use asset::{XrayAsset, XrayAssetContainer, XrayAssetRules, XrayAssetType, require_writable_path};
pub use cache::{XrayCachePolicy, XrayCacheStats};
pub use change::{XrayChange, XrayChangeKind, XrayChangeSet, XrayTransaction};
pub use fsgame::{FsgameDeclaration, FsgameFile};
pub use input::{XrayInput, XrayInputAnswer, XrayInputKind, XrayInputRecord, XrayRevision, XraySearchKey};
pub use mount::{
  XrayMount, XrayMountId, XrayMountMode, XrayRoot, XrayRootKind, XrayRootProbe, XrayRoots, XraySkippedMount,
};
pub use path::{XrayCollisionSite, XrayLogicalPath, XrayPathCollision};
pub use search::{
  XrayDirectoryListing, XrayLookupScope, XrayMountedEntry, XrayReadHandle, XrayResolution, XraySearch, XraySearchPlan,
  XraySearchStep, XraySearchedSource, XrayShadowedCopy, XrayShadowingEntry,
};
pub use source::{XrayArchiveSource, XrayAssetSource, XrayDeclaredRoot, XraySourceKind, label_from_path};
pub use trace::{XrayReadTrace, XrayReadTraceEntry, XrayReadTraceHotPath, XrayReadTraceSummary};
pub use vfs::{XrayVfs, XrayVfsHost};
