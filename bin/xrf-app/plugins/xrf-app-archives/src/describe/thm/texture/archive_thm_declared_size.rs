use serde::Serialize;
use xrf_app_core::assets::AssetTextureShape;
use xrf_thm::{ThmTextureParamChunk, ThmTextureType};

/// A declared size the texture beside the descriptor does not match.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmDeclaredSize {
  pub width: u32,
  pub height: u32,
  /// Whether the declaration is a cube map's source strip: six faces wide, one face tall.
  pub is_cube_strip: bool,
}

impl ArchiveThmDeclaredSize {
  /// The declared size, when there is a file to compare it against and the two differ.
  pub(super) fn of(
    texture_type: ThmTextureType,
    parameters: Option<&ThmTextureParamChunk>,
    shape: Option<&AssetTextureShape>,
  ) -> Option<Self> {
    let parameters: &ThmTextureParamChunk = parameters?;
    let shape: &AssetTextureShape = shape?;

    if parameters.width == shape.width && parameters.height == shape.height {
      return None;
    }

    Some(Self {
      width: parameters.width,
      height: parameters.height,
      // Type-aware on purpose: 53 of the 54 vanilla descriptors whose declaration differs are cube maps recording
      // their six-face source strip, so a comparison blind to the type is almost all noise.
      is_cube_strip: texture_type == ThmTextureType::CubeMap
        && parameters.height == shape.height
        && parameters.width == shape.width.saturating_mul(6),
    })
  }
}

#[cfg(test)]
mod tests {
  use xrf_app_core::assets::AssetTextureShape;
  use xrf_thm::{ThmTextureParamChunk, ThmTextureType};

  use super::ArchiveThmDeclaredSize;

  fn shape(width: u32, height: u32) -> AssetTextureShape {
    AssetTextureShape {
      width,
      height,
      mipmap_levels: 1,
      format: String::from("DXT1"),
    }
  }

  fn parameters(width: u32, height: u32) -> ThmTextureParamChunk {
    ThmTextureParamChunk {
      width,
      height,
      ..ThmTextureParamChunk::default()
    }
  }

  #[test]
  fn a_descriptor_agreeing_with_its_texture_declares_nothing_extra() {
    assert_eq!(
      ArchiveThmDeclaredSize::of(
        ThmTextureType::Image,
        Some(&parameters(512, 512)),
        Some(&shape(512, 512))
      ),
      None
    );
  }

  #[test]
  fn a_cube_maps_source_strip_is_recognised_rather_than_reported_as_a_disagreement() {
    // `sky\sky_12_cube.thm` declares the six-face strip it was converted from; the file is the packed cube.
    let declared: ArchiveThmDeclaredSize = ArchiveThmDeclaredSize::of(
      ThmTextureType::CubeMap,
      Some(&parameters(3072, 512)),
      Some(&shape(512, 512)),
    )
    .expect("the sizes differ");

    assert_eq!((declared.width, declared.height), (3072, 512));
    assert!(declared.is_cube_strip);
  }

  #[test]
  fn a_plain_disagreement_is_carried_without_an_explanation() {
    // `ui\ui_grid.thm`, the one vanilla descriptor that genuinely disagrees with the texture beside it.
    let declared: ArchiveThmDeclaredSize =
      ArchiveThmDeclaredSize::of(ThmTextureType::Image, Some(&parameters(128, 64)), Some(&shape(256, 64)))
        .expect("the sizes differ");

    assert!(!declared.is_cube_strip);
  }

  #[test]
  fn a_six_wide_declaration_is_only_a_strip_for_a_cube_map() {
    let declared: ArchiveThmDeclaredSize = ArchiveThmDeclaredSize::of(
      ThmTextureType::Image,
      Some(&parameters(3072, 512)),
      Some(&shape(512, 512)),
    )
    .expect("the sizes differ");

    assert!(!declared.is_cube_strip, "the type is what makes the strip meaningful");
  }

  #[test]
  fn nothing_is_declared_without_a_texture_to_compare_against() {
    assert_eq!(
      ArchiveThmDeclaredSize::of(ThmTextureType::Image, Some(&parameters(512, 512)), None),
      None
    );
  }
}
