use crate::parse::extern_source_location::ExternSourceLocation;
use crate::parse::parsed_extern::ParsedExtern;

/// One top-level `extern(key, value)` call: the key it assigns, where it is written, and the names it publishes, which
/// are the key itself or, for an object literal, one name per property under it and none for an empty one.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ExternStatement {
  pub key: String,
  pub location: ExternSourceLocation,
  /// Whether the value is an object literal, which leaves a table at the key a later call can add to.
  pub is_table: bool,
  pub externs: Vec<ParsedExtern>,
}

impl ExternStatement {
  /// Whether `xray16/lib`'s `extern` would lose or fail one of two calls, `self` read before `later`.
  pub fn writes_over(&self, later: &Self) -> bool {
    let is_inside: bool = Self::is_within(&later.key, &self.key) && later.key != self.key;

    if !(Self::is_within(&self.key, &later.key) || Self::is_within(&later.key, &self.key)) {
      false
    } else if self.location.path != later.location.path {
      true
    } else {
      !(is_inside && self.is_table)
    }
  }

  /// Whether a key is another one or a dotted path under it.
  fn is_within(key: &str, outer: &str) -> bool {
    key
      .strip_prefix(outer)
      .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
  }
}
