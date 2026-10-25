/// Something the parser could read but the schema does not describe.
///
/// Reading never fails on these: shipped dialog data holds all of them, and a reader that refused
/// would be unable to open the files it exists to inspect. Validation turns them into findings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialogParseIssueKind {
  /// An element the schema does not define at this level.
  UnknownElement,
  /// An attribute the schema does not define at this level.
  UnknownAttribute,
  /// A `dialog` or `phrase` with no `id`, which cannot be addressed and is skipped.
  MissingId,
  /// A `priority` attribute that is not an integer. The dialog is kept without one.
  InvalidPriority,
}
