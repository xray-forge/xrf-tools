//! Walking a resolved config: its sections in declared order and each section's fields in written order.

pub(crate) mod ltx_iterator;
pub(crate) mod properties_into_iter;
pub(crate) mod property_iter;
pub(crate) mod section_into_iter;
pub(crate) mod section_iter;
pub(crate) mod section_iter_mut;

pub use crate::ltx::iter::properties_into_iter::PropertiesIntoIter;
pub use crate::ltx::iter::property_iter::PropertyIter;
pub use crate::ltx::iter::section_into_iter::SectionIntoIter;
pub use crate::ltx::iter::section_iter::SectionIter;
pub use crate::ltx::iter::section_iter_mut::SectionIterMut;
