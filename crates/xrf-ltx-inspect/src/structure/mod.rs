//! One config as written, minus what a syntax highlighter can already see.

pub(crate) mod ltx_declared_parents;
pub(crate) mod ltx_document_scan;
pub(crate) mod ltx_file_structure;
pub(crate) mod ltx_structure_entry;
pub(crate) mod ltx_structure_include;
pub(crate) mod ltx_structure_parent;
pub(crate) mod ltx_structure_parse_error;
pub(crate) mod ltx_structure_reader;
pub(crate) mod ltx_structure_scheme;
pub(crate) mod ltx_structure_section;

pub(crate) use crate::structure::ltx_declared_parents::LtxDeclaredParents;
pub(crate) use crate::structure::ltx_document_scan::LtxDocumentScan;
pub use crate::structure::ltx_file_structure::LtxFileStructure;
pub use crate::structure::ltx_structure_entry::LtxStructureEntry;
pub use crate::structure::ltx_structure_include::LtxStructureInclude;
pub use crate::structure::ltx_structure_parent::LtxStructureParent;
pub use crate::structure::ltx_structure_parse_error::LtxStructureParseError;
pub(crate) use crate::structure::ltx_structure_reader::LtxStructureReader;
pub use crate::structure::ltx_structure_scheme::LtxStructureScheme;
pub use crate::structure::ltx_structure_section::LtxStructureSection;
