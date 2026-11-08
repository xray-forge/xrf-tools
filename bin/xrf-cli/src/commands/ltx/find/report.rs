use serde::Serialize;
use xrf_ltx_inspect::LtxDeclarationSearch;

/// What `ltx find` answers: the query, every statement writing what it names, and the configs it could not read.
///
/// Authored statements only. The value that survives resolution is `ltx inspect`'s answer, so nothing here claims to
/// be one.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LtxFindReport<'a> {
  /// How the dialect the project was opened with names itself.
  dialect: &'a str,
  #[serde(flatten)]
  search: &'a LtxDeclarationSearch,
}

impl<'a> LtxFindReport<'a> {
  pub(crate) fn new(dialect: &'a str, search: &'a LtxDeclarationSearch) -> Self {
    Self { dialect, search }
  }
}
