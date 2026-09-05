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

  /// The pair of references the renderer will try to bind, when the declaration is one it reads.
  pub fn declared_bump_pair(&self) -> Option<(&str, &str)> {
    self
      .bump
      .as_ref()
      .map(|bump| (bump.bump.reference.as_str(), bump.companion.reference.as_str()))
  }

  /// Whether the descriptor's texture type makes `LoadTHM` skip it whole, bump declaration included.
  pub fn is_engine_skipped(&self) -> bool {
    matches!(self.declaration, XrayMaterialDeclaration::TypeDisqualified { .. })
  }

  /// Whether a `.thm` was located and could not be read as one.
  pub fn is_unreadable(&self) -> bool {
    matches!(self.declaration, XrayMaterialDeclaration::Unreadable { .. })
  }

  /// Whether a detail texture is named and one of the two flags that switch it on is set, so the engine applies it.
  pub fn is_detail_associated(&self) -> bool {
    self.detail.as_ref().is_some_and(|detail| detail.usage.is_some())
  }
}
