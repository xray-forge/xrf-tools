//! Monolith-compatible DLTX evaluation, as one [`xrf_ltx::LtxDialect`].

pub(crate) mod discovery;
pub(crate) mod dltx_dialect;
pub(crate) mod load;
pub(crate) mod resolve;
pub(crate) mod select_ltx_dialect;

pub use crate::dltx_dialect::DltxDialect;
pub use crate::select_ltx_dialect::select_ltx_dialect;

#[cfg(test)]
mod tests;
