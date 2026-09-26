use serde::Serialize;

use crate::XrayAsset;

/// What one reference lookup came to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum XrayResolution {
  /// The reference itself resolved.
  Resolved { step: String, assets: Vec<XrayAsset> },
  /// The reference did not resolve, but the fallback the caller offered did.
  Substituted {
    step: String,
    fallback: String,
    assets: Vec<XrayAsset>,
  },
  /// Nothing resolved, across every step of the probe.
  Missing { roots: Vec<String> },
  /// There was nothing to search: the probe had no step, or no step selected a mounted source.
  NoScope,
  /// The reference could not be turned into a lookup at all, so none was attempted.
  Rejected { reason: String },
}

impl XrayResolution {
  /// The located assets, empty unless the reference or its fallback resolved.
  pub fn get_assets(&self) -> &[XrayAsset] {
    match self {
      Self::Resolved { assets, .. } | Self::Substituted { assets, .. } => assets,
      Self::Missing { .. } | Self::NoScope | Self::Rejected { .. } => &[],
    }
  }

  /// The first located asset, for a reference that cannot be a mask.
  pub fn get_asset(&self) -> Option<&XrayAsset> {
    self.get_assets().first()
  }

  /// The probe step that answered, for a located outcome.
  pub fn get_step(&self) -> Option<&str> {
    match self {
      Self::Resolved { step, .. } | Self::Substituted { step, .. } => Some(step),
      Self::Missing { .. } | Self::NoScope | Self::Rejected { .. } => None,
    }
  }

  /// Whether anything was located, by the reference or by its fallback.
  pub fn is_located(&self) -> bool {
    !self.get_assets().is_empty()
  }
}
