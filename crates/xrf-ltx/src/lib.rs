#![doc = include_str!("../README.md")]

pub(crate) mod binding;
pub(crate) mod condlist;
pub(crate) mod dialect;
pub(crate) mod document;
pub(crate) mod finding;
pub(crate) mod ltx;
pub(crate) mod project;
pub(crate) mod scheme;
pub(crate) mod source;
pub(crate) mod syntax;

pub use crate::binding::ltx_field::{read_ltx_field, read_ltx_optional_field};
pub use crate::binding::ltx_import_export::{FileImportExport, LtxImportExport, META_TYPE_FIELD};
pub use crate::condlist::{Condlist, CondlistBranch, CondlistCondition};
pub use crate::dialect::{
  LtxDialect, LtxFieldOrigin, LtxProvenance, LtxResolution, LtxResolutionDiagnostic, LtxResolveRequest,
  LtxStandardDialect, LtxTextInterner,
};
pub use crate::document::{LtxCheck, LtxDocument, LtxItem, LtxItemKind, LtxKeyOperation, LtxSectionOperation, LtxSpan};
pub use crate::finding::{LtxFinding, LtxFindingKind};
pub use crate::ltx::{Ltx, LtxSectionFieldSchemes, LtxSectionSchemes, Section};
pub use crate::project::{
  LTX_PHASE_CHECK, LTX_PHASE_FORMAT, LTX_PHASE_VERIFY, LtxEntryVerification, LtxFilesFormatter, LtxFormatOptions,
  LtxProject, LtxProjectFormatResult, LtxProjectOptions, LtxProjectVerifyResult, LtxReadCountersSnapshot,
  LtxVerifyOptions,
};
pub use crate::scheme::{LtxFieldDataType, LtxFieldScheme, LtxSchemeParser, LtxSectionScheme};
pub use crate::source::LtxDocumentSource;
pub use crate::syntax::ltx_value::{read_engine_bool, read_engine_float, read_engine_floats, scan_engine_float};
pub use crate::syntax::{LTX_SCHEME_FIELD, LTX_SYMBOL_ANY, LTX_SYMBOL_INCLUDE_WILDCARD, LTX_SYMBOL_SCHEME};
