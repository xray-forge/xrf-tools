//! Reading an LTX project as records something outside Rust can render.
//!
//! Inspection sits on top of `xrf-ltx` rather than inside it: the core is the shared reader every app and command
//! resolves configs through, and nothing there should grow a field because a viewer wants one. Every record here is
//! serialized, line-anchored, and carries the entry point it was computed against, so a consumer can key a cache by it
//! and mark a gutter without ever computing a line itself.
//!
//! Dialect-neutral by construction. The queries read an [`xrf_ltx::LtxResolution`], which both the standard dialect and
//! the Monolith DLTX one answer with, so this crate never depends on `xrf-dltx` and never asks which produced its input.
//!
//! # Module map
//!
//! The crate is grouped by the question each part answers:
//!
//! - [`inventory`] — what the project contains, and which role each config plays in it.
//! - [`structure`] — one config as written, minus what a syntax highlighter can already see.
//! - [`resolved`] — one resolution, indexed and sliceable a page of sections at a time.
//! - [`findings`] — what is wrong, anchored to the line a person has to open.
//! - [`text`] — one config's lines, as the authored view renders them.

pub(crate) mod findings;
pub(crate) mod inventory;
pub(crate) mod resolved;
pub(crate) mod structure;
pub(crate) mod text;

#[cfg(test)]
mod tests;

pub use crate::findings::{LtxAnchoredFinding, LtxFindingAnchor, LtxFindingKind};
pub use crate::inventory::{LtxInventory, LtxInventoryFile, LtxInventoryReader, LtxInventoryRole};
pub use crate::resolved::{
  LtxResolvedDiagnostic, LtxResolvedField, LtxResolvedFieldOrigin, LtxResolvedIndex, LtxResolvedIndexEntry,
  LtxResolvedReader, LtxResolvedSection,
};
pub use crate::structure::{
  LtxFileStructure, LtxStructureInclude, LtxStructureParent, LtxStructureParseError, LtxStructureReader,
  LtxStructureScheme, LtxStructureSection,
};
pub use crate::text::{LtxFileText, read_text};
