//! One resolved section, and the setter and entry views a caller writes sections through.

pub(crate) mod section;
pub(crate) mod section_entry;
pub(crate) mod section_occupied_entry;
pub(crate) mod section_setter;
pub(crate) mod section_vacant_entry;
pub(crate) mod section_value_text;

pub use crate::ltx::section::section::Section;
pub(crate) use crate::ltx::section::section_entry::SectionEntry;
pub(crate) use crate::ltx::section::section_occupied_entry::SectionOccupiedEntry;
pub(crate) use crate::ltx::section::section_setter::SectionSetter;
pub(crate) use crate::ltx::section::section_vacant_entry::SectionVacantEntry;
