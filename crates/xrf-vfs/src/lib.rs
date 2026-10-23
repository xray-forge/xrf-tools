#![doc = include_str!("../README.md")]

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
pub use source::{
  XrayArchiveSource, XrayAssetSource, XrayDeclaredRoot, XrayFileStamp, XraySourceKind, label_from_path,
};
pub use trace::{XrayReadTrace, XrayReadTraceEntry, XrayReadTraceHotPath, XrayReadTraceSummary};
pub use vfs::{XrayVfs, XrayVfsHost};
