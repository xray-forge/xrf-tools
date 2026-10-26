//! One config as written, minus what a syntax highlighter can already see.

pub(crate) mod ltx_declared_parents;
pub(crate) mod ltx_document_scan;
pub(crate) mod ltx_structure_reader;
pub(crate) mod record;

pub(crate) use crate::structure::ltx_declared_parents::LtxDeclaredParents;
pub(crate) use crate::structure::ltx_document_scan::LtxDocumentScan;
pub(crate) use crate::structure::ltx_structure_reader::LtxStructureReader;
pub use crate::structure::record::{
  LtxFileStructure, LtxStructureEntry, LtxStructureInclude, LtxStructureParent, LtxStructureParseError,
  LtxStructureScheme, LtxStructureSection,
};
