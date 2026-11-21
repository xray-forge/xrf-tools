use serde::Serialize;
use xrf_material::XrayBumpNaming;
use xrf_vfs::XrayAssetType;

/// Which of a texture's files one write of a copy is.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum TextureSaveAsRole {
  /// The `.dds` holding the pixels.
  Texture,
  /// The `.thm` descriptor, written as edited.
  Descriptor,
  /// The `_bump.dds` the bump generator wrote.
  Bump,
  /// The `_bump#.dds` beside the bump.
  Companion,
  /// The bump's own `.thm`, which an SDK rebuild of the pair reads.
  BumpDescriptor,
}

impl TextureSaveAsRole {
  /// Every role, in write order: the descriptor after the texture it describes.
  pub(crate) const ALL: [Self; 5] = [
    Self::Texture,
    Self::Descriptor,
    Self::Bump,
    Self::Companion,
    Self::BumpDescriptor,
  ];

  /// Whether this file belongs to the bump map rather than to the texture itself.
  pub(crate) const fn is_bump(self) -> bool {
    matches!(self, Self::Bump | Self::Companion | Self::BumpDescriptor)
  }

  pub(crate) const fn get_asset_type(self) -> XrayAssetType {
    match self {
      Self::Texture | Self::Bump | Self::Companion => XrayAssetType::Dds,
      Self::Descriptor | Self::BumpDescriptor => XrayAssetType::Thm,
    }
  }

  /// The name of this file for the texture named `texture`, as the SDK names a pair after its texture.
  pub(crate) fn name_for(self, texture: &str) -> String {
    match self {
      Self::Texture | Self::Descriptor => texture.to_owned(),
      Self::Bump | Self::BumpDescriptor => XrayBumpNaming::bump_of(texture),
      Self::Companion => XrayBumpNaming::companion_of(&XrayBumpNaming::bump_of(texture)),
    }
  }
}
