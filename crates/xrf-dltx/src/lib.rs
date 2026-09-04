//! Monolith-compatible DLTX evaluation over parsed LTX documents.

pub(crate) mod dltx_attachment;
pub(crate) mod dltx_diagnostic;
pub(crate) mod dltx_dialect;
pub(crate) mod dltx_discovery;
pub(crate) mod dltx_item;
pub(crate) mod dltx_map_source;
pub(crate) mod dltx_provenance;
pub(crate) mod dltx_resolver;
pub(crate) mod dltx_severity;
pub(crate) mod dltx_stores;

pub use crate::dltx_attachment::*;
pub use crate::dltx_diagnostic::*;
pub use crate::dltx_dialect::*;
pub use crate::dltx_discovery::*;
pub use crate::dltx_item::*;
pub use crate::dltx_map_source::*;
pub use crate::dltx_provenance::*;
pub use crate::dltx_resolver::*;
pub use crate::dltx_severity::*;
pub use crate::dltx_stores::*;

#[cfg(test)]
mod tests;
