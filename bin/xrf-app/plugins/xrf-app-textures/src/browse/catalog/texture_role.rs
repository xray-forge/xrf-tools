use serde::Serialize;
use xrf_material::XrayBumpNaming;

/// What a texture name is by convention, before any descriptor has been read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextureRole {
  /// A texture a mesh or a level binds by name.
  Texture,
  /// The first half of a bump pair: packed normal and gloss.
  Bump,
  /// The second half of a bump pair: packed error and height.
  BumpCompanion,
}

impl TextureRole {
  /// The role a reference's own name gives it, by the engine's bump naming.
  pub fn of_reference(reference: &str) -> Self {
    if XrayBumpNaming::is_companion(reference) {
      Self::BumpCompanion
    } else if XrayBumpNaming::is_conventional_bump(reference) {
      Self::Bump
    } else {
      Self::Texture
    }
  }
}
