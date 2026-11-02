//! One resolved section, and the setter and entry views a caller writes sections through.

pub(crate) mod section;
pub(crate) mod section_entry;
pub(crate) mod section_occupied_entry;
pub(crate) mod section_setter;
pub(crate) mod section_vacant_entry;

pub use crate::ltx::section::section::Section;
pub use crate::ltx::section::section_entry::SectionEntry;
pub use crate::ltx::section::section_occupied_entry::SectionOccupiedEntry;
pub use crate::ltx::section::section_setter::SectionSetter;
pub use crate::ltx::section::section_vacant_entry::SectionVacantEntry;
