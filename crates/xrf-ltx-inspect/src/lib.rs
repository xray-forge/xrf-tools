#![doc = include_str!("../README.md")]

pub(crate) mod findings;
pub(crate) mod inventory;
pub(crate) mod ltx_root_reader;
pub(crate) mod resolved;
pub(crate) mod scheme;
pub(crate) mod structure;
pub(crate) mod text;

#[cfg(test)]
mod tests;

pub use crate::findings::LtxAnchoredFinding;
pub use crate::inventory::{LtxInventory, LtxInventoryFile, LtxInventoryReader, LtxInventoryRole};
pub use crate::ltx_root_reader::LtxRootReader;
pub use crate::resolved::{
  LtxResolvedDiagnostic, LtxResolvedField, LtxResolvedFieldOrigin, LtxResolvedIndex, LtxResolvedIndexEntry,
  LtxResolvedSection,
};
pub use crate::scheme::{LtxSchemeFieldDeclaration, LtxSchemeFieldReport, LtxSectionSchemeReport};
pub use crate::structure::{
  LtxFileStructure, LtxStructureEntry, LtxStructureInclude, LtxStructureParent, LtxStructureParseError,
  LtxStructureScheme, LtxStructureSection,
};
pub use crate::text::{LtxFileText, LtxTextReader};
