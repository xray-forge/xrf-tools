//! Reading an LTX project as records something outside Rust can render.
//!
//! Inspection sits on top of `xrf-ltx` rather than inside it: the core is the shared reader every app and command
//! resolves configs through, and nothing there should grow a field because a viewer wants one. Every record here is
//! serialized, line-anchored, and carries the entry point it was computed against, so a consumer can key a cache by it
//! and mark a gutter without ever computing a line itself.
//!
//! Dialect-neutral by construction. The readers take an [`xrf_ltx::LtxResolution`], which both the standard dialect and
//! the Monolith DLTX one answer with, so this crate never depends on `xrf-dltx` and never asks which produced its
//! input.
//!
//! # Two scopes
//!
//! - [`LtxInventoryReader`] answers about a whole **project**: every config it holds, and what each one is to it.
//! - [`LtxRootReader`] answers about one resolved **root**: a config's structure, the sections the root resolves to,
//!   and everything wrong with it. One reader rather than one per question, because all of them need the same entry
//!   point, resolution and source to answer, and a surface showing a file beside its findings is one screen.
//!
//! [`LtxTextReader`] belongs to neither: a config's lines need no project and no resolution.
//!
//! # Module map
//!
//! The record groups are named for the question each answers:
//!
//! - `inventory` — what the project contains, and which role each config plays in it.
//! - `structure` — one config as written, minus what a syntax highlighter can already see.
//! - `resolved` — one resolution, indexed and sliceable a page of sections at a time.
//! - `scheme` — what judges one section, and how the section measures against it.
//! - `findings` — what is wrong, anchored to the line a person has to open.
//! - `text` — one config's lines, as the authored view renders them.

pub(crate) mod findings;
pub(crate) mod inventory;
pub(crate) mod ltx_root_reader;
pub(crate) mod resolved;
pub(crate) mod scheme;
pub(crate) mod structure;
pub(crate) mod text;

#[cfg(test)]
mod tests;

pub use crate::findings::{LtxAnchoredFinding, LtxFindingKind};
pub use crate::inventory::{LtxInventory, LtxInventoryFile, LtxInventoryReader, LtxInventoryRole};
pub use crate::ltx_root_reader::LtxRootReader;
pub use crate::resolved::{
  LtxResolvedDiagnostic, LtxResolvedField, LtxResolvedFieldOrigin, LtxResolvedIndex, LtxResolvedIndexEntry,
  LtxResolvedSection,
};
pub use crate::scheme::{LtxSchemeFieldDeclaration, LtxSchemeFieldReport, LtxSectionSchemeReport};
pub use crate::structure::{
  LtxFileStructure, LtxStructureInclude, LtxStructureParent, LtxStructureParseError, LtxStructureScheme,
  LtxStructureSection,
};
pub use crate::text::{LtxFileText, LtxTextReader};
