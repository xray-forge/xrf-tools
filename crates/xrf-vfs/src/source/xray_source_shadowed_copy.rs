use crate::XrayAssetContainer;

/// One engine path a source answers with more than one copy, and the stack behind the one it answers with.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XraySourceOverride {
  /// Engine identity, in the source's own terms and before any mount base is applied.
  pub logical_path: String,
  /// Where the copy a lookup reaches physically sits.
  pub container: XrayAssetContainer,
  /// Payload bytes of that copy, once unpacked.
  pub size: u64,
  /// Copies behind it, in precedence order.
  pub shadowed: Vec<XraySourceShadowedCopy>,
}

/// One copy of an engine path a source holds behind the copy it answers with.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XraySourceShadowedCopy {
  /// Engine identity this copy would answer for, in the source's own terms and before any mount base is applied.
  pub logical_path: String,
  /// Where the copy physically sits inside the source.
  pub container: XrayAssetContainer,
  /// Payload bytes once unpacked.
  pub size: u64,
}
