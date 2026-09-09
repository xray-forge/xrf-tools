//! One config as written, minus what a syntax highlighter can already see.

pub(crate) mod ltx_declared_parents;
pub(crate) mod ltx_document_scan;
pub(crate) mod ltx_file_structure;
pub(crate) mod ltx_structure_reader;

pub(crate) use crate::structure::ltx_declared_parents::LtxDeclaredParents;
pub(crate) use crate::structure::ltx_document_scan::LtxDocumentScan;
pub use crate::structure::ltx_file_structure::{
  LtxFileStructure, LtxStructureInclude, LtxStructureParent, LtxStructureParseError, LtxStructureScheme,
  LtxStructureSection,
};
pub(crate) use crate::structure::ltx_structure_reader::LtxStructureReader;
