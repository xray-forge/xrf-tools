//! Reading, resolving, verifying and formatting LTX configs.
//!
//! # Module map
//!
//! The crate is grouped by the stage each part answers for, and a config passes through them in this order:
//!
//! - [`syntax`] — what the characters and reserved names mean.
//! - [`document`] — one file as it was written: the scan, the statements, and the canonical rendering back to text.
//! - [`source`] — where a document comes from, so a dialect never learns whether configs sit on disk or in volumes.
//! - [`dialect`] — which rules turn a tree of documents into a resolved config. Standard LTX is here; the Monolith
//!   DLTX patch dialect is `xrf-dltx`, which implements the same trait from outside.
//! - [`ltx`] — the resolved config a dialect answers with, and what a caller can do with one.
//! - [`project`] — a whole tree of configs, and the verify and format passes over it.
//!
//! [`scheme`] and [`condlist`] are the two value languages a field may be written in, read on demand rather than as a
//! stage.

pub(crate) mod condlist;
pub(crate) mod dialect;
pub(crate) mod document;
pub(crate) mod ltx;
pub(crate) mod project;
pub(crate) mod scheme;
pub(crate) mod source;
pub(crate) mod syntax;

pub use crate::dialect::{LtxDialect, LtxFieldOrigin, LtxResolution, LtxResolutionDiagnostic, LtxStandardDialect};
pub use crate::document::{LtxCheck, LtxDocument, LtxItem, LtxItemKind, LtxKeyOperation, LtxSectionOperation, LtxSpan};
pub use crate::ltx::{Ltx, Section};
pub use crate::project::{
  LTX_PHASE_CHECK, LTX_PHASE_FORMAT, LTX_PHASE_VERIFY, LtxFilesFormatter, LtxFormatOptions, LtxProject,
  LtxProjectFormatResult, LtxProjectOptions, LtxProjectVerifyResult, LtxReadCountersSnapshot, LtxVerifyOptions,
};
pub use crate::source::LtxDocumentSource;
pub use crate::syntax::{LTX_EXTENSION, LTX_SYMBOL_SCHEME};
