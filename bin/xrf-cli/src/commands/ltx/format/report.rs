use serde::Serialize;
use xrf_ltx::LtxProjectFormatResult;

/// What a formatting run, or a check of one, did: the formatter's result, and the archived configs it declined, which a
/// run over an installation cannot rewrite in place.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxFormatReport<'a> {
  #[serde(flatten)]
  result: &'a LtxProjectFormatResult,
  declined: &'a [String],
}

impl<'a> LtxFormatReport<'a> {
  /// The report of `result`, having declined `declined`.
  pub fn new(result: &'a LtxProjectFormatResult, declined: &'a [String]) -> Self {
    Self { result, declined }
  }
}
