//! Standard LTX: includes merged in read order, inheritance resolved eagerly, each document lowered as it is read.

pub(crate) mod ltx_open_section;
pub(crate) mod ltx_standard_dialect;
pub(crate) mod ltx_standard_lower;

pub(crate) use crate::dialect::standard::ltx_open_section::LtxOpenSection;
pub use crate::dialect::standard::ltx_standard_dialect::LtxStandardDialect;
