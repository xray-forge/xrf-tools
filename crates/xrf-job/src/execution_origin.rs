use serde::Serialize;

/// Where a plan's worker count came from.
///
/// Carried because the same number means different things to an operation that knows its own costs. Under [`Self::Auto`]
/// nobody chose it, so an operation that has measured a reason to use less of the machine — overlapping phases whose
/// peak memory would collide, say — may do so. Under [`Self::Requested`] a person named the number, and going under it
/// would be overriding an instruction rather than exercising judgement.
///
/// Restraint is the exception either way. An operation that spends less than its plan allows owes a measurement, not a
/// preference.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExecutionOrigin {
  /// Resolved from the machine, because the caller expressed no preference.
  #[default]
  Auto,
  /// Named by the caller, as a worker count or as a share of the machine.
  Requested,
}
