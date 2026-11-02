#![doc = include_str!("../README.md")]

mod asset;
mod change;
mod input;
mod mount;
mod path;
mod search;
mod source;
mod vfs;

pub use asset::{XrayAsset, XrayAssetContainer, XrayAssetRules, XrayAssetType, require_writable_path};
pub use change::{XrayChange, XrayChangeKind, XrayChangeSet, XrayTransaction};
pub use input::{XrayInput, XrayInputAnswer, XrayInputKind, XrayInputRecord, XrayRevision};
pub use mount::{
  FsgameDeclaration, FsgameFile, XrayMount, XrayMountId, XrayMountMode, XrayRoot, XrayRootKind, XrayRootProbe,
  XrayRoots, XraySkippedMount,
};
pub use path::{XrayCollisionSite, XrayLogicalPath, XrayPathCollision};
pub use search::{
  XrayDirectoryListing, XrayLookupScope, XrayMountedEntry, XrayReadHandle, XrayResolution, XraySearch, XraySearchKey,
  XraySearchPlan, XraySearchStep, XraySearchedSource, XrayShadowedCopy, XrayShadowingEntry,
};
pub use source::{
  XrayArchiveSource, XrayAssetSource, XrayDeclaredRoot, XrayFileStamp, XraySourceKind, label_from_path,
};
pub use vfs::{
  XrayCachePolicy, XrayCacheStats, XrayReadTrace, XrayReadTraceEntry, XrayReadTraceHotPath, XrayReadTraceSummary,
  XrayVfs, XrayVfsHost,
};
