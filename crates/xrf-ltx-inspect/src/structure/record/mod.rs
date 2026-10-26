//! The records a structure read answers with: the file, and each header, key, include and binding it writes.

pub(crate) mod ltx_file_structure;
pub(crate) mod ltx_structure_entry;
pub(crate) mod ltx_structure_include;
pub(crate) mod ltx_structure_parent;
pub(crate) mod ltx_structure_parse_error;
pub(crate) mod ltx_structure_scheme;
pub(crate) mod ltx_structure_section;

pub use crate::structure::record::ltx_file_structure::LtxFileStructure;
pub use crate::structure::record::ltx_structure_entry::LtxStructureEntry;
pub use crate::structure::record::ltx_structure_include::LtxStructureInclude;
pub use crate::structure::record::ltx_structure_parent::LtxStructureParent;
pub use crate::structure::record::ltx_structure_parse_error::LtxStructureParseError;
pub use crate::structure::record::ltx_structure_scheme::LtxStructureScheme;
pub use crate::structure::record::ltx_structure_section::LtxStructureSection;
