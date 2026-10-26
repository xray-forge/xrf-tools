//! A resolved config: sections and their fields once a dialect has had its say, and what a caller can do with one.

pub(crate) mod iter;
pub(crate) mod ltx;
pub(crate) mod ltx_collections;
pub(crate) mod ltx_inherit;
pub(crate) mod ltx_read;
pub(crate) mod ltx_write;
pub(crate) mod section;

#[cfg(test)]
mod tests;

pub(crate) use crate::ltx::iter::PropertyIter;
pub use crate::ltx::ltx::Ltx;
pub(crate) use crate::ltx::ltx_collections::{LtxSections, SectionData};
pub(crate) use crate::ltx::ltx_inherit::LtxInheritConvertor;
pub use crate::ltx::section::{Section, SectionEntry, SectionSetter};
