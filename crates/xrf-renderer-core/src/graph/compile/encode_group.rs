use std::ops::Range;

/// A run of compiled passes recorded into one encoder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodeGroup {
  pub name: &'static str,
  pub passes: Range<usize>,
}
