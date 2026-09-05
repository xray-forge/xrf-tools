use serde::Serialize;
use xrf_vfs::XrayAsset;

use crate::data::xray_bump_outcome::XrayBumpOutcome;
use crate::data::xray_material_bump::XrayMaterialBump;
use crate::data::xray_material_declaration::XrayMaterialDeclaration;
use crate::data::xray_material_detail::XrayMaterialDetail;

/// The material the renderer builds for one texture, resolved.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayMaterialDescriptor {
  /// The `.thm` the declaration was read from, or `None` when no root holds one.
  pub descriptor: Option<XrayAsset>,
  pub declaration: XrayMaterialDeclaration,
  /// The bound pair, present exactly when the declaration is [`XrayMaterialDeclaration::Declared`].
  pub bump: Option<XrayMaterialBump>,
  pub outcome: XrayBumpOutcome,
  /// The detail association the descriptor names, when the type gate lets the engine read it and it names one.
  pub detail: Option<XrayMaterialDetail>,
}

impl XrayMaterialDescriptor {
  /// A texture with no descriptor anywhere, which is most of them.
  pub fn undeclared() -> Self {
    Self::flat(None, XrayMaterialDeclaration::NoDescriptor)
  }

  /// A descriptor the engine reads nothing from: no pair is bound and no detail is associated.
  pub fn flat(descriptor: Option<XrayAsset>, declaration: XrayMaterialDeclaration) -> Self {
    Self {
      descriptor,
      declaration,
      bump: None,
      outcome: XrayBumpOutcome::Flat,
      detail: None,
    }
  }
}
