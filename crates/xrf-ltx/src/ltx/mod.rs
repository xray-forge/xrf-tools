//! A resolved config: sections and their fields once a dialect has had its say, and what a caller can do with one.

pub(crate) mod ltx;
pub(crate) mod ltx_collections;
pub(crate) mod ltx_include;
pub(crate) mod ltx_inherit;
pub(crate) mod ltx_iterator;
pub(crate) mod ltx_read;
pub(crate) mod ltx_write;
pub(crate) mod properties_into_iter;
pub(crate) mod property_iter;
pub(crate) mod section;
pub(crate) mod section_entry;
pub(crate) mod section_into_iter;
pub(crate) mod section_iter;
pub(crate) mod section_iter_mut;
pub(crate) mod section_occupied_entry;
pub(crate) mod section_setter;
pub(crate) mod section_vacant_entry;

#[cfg(test)]
mod tests;

pub use crate::ltx::ltx::Ltx;
pub use crate::ltx::ltx_collections::{
  LtxIncluded, LtxSectionFieldSchemes, LtxSectionSchemes, LtxSections, SectionData,
};
pub(crate) use crate::ltx::ltx_include::LtxIncludeConvertor;
pub(crate) use crate::ltx::ltx_inherit::LtxInheritConvertor;
pub(crate) use crate::ltx::property_iter::PropertyIter;
pub use crate::ltx::section::Section;
pub use crate::ltx::section_entry::SectionEntry;
pub use crate::ltx::section_setter::SectionSetter;
