use serde::Serialize;
use xrf_db::{ThmTextureParamChunk, ThmTextureType};
use xrf_vfs::XrayLogicalPath;

use crate::core::assets::AssetTextureShape;
use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;

/// The texture a descriptor sits beside, and what that file actually is.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmTexture {
  /// The `.dds` this descriptor describes, which is the file beside it rather than a name it carries.
  pub reference: ArchiveReference,
  /// What that file's header declares, when it was found and its header parsed.
  pub shape: Option<AssetTextureShape>,
  /// The size the descriptor claims, carried only when the file beside it measures something else.
  pub declared: Option<ArchiveThmDeclaredSize>,
}

/// A declared size the texture beside the descriptor does not match.
///
/// Reported as two facts rather than as a fault: the descriptor's width and height are authoring data and the file is
/// the authority, so a disagreement is worth seeing and is not by itself wrong.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmDeclaredSize {
  pub width: u32,
  pub height: u32,
  /// Whether the declaration is a cube map's source strip: six faces wide, one face tall.
  pub is_cube_strip: bool,
}

impl ArchiveThmTexture {
  /// The texture an entry's descriptor describes, resolved against the subject being browsed.
  pub fn of(
    source: &ArchiveDescribeSource,
    name: &str,
    texture_type: ThmTextureType,
    parameters: Option<&ThmTextureParamChunk>,
  ) -> Self {
    let reference: ArchiveReference = match to_sibling_texture_path(name) {
      Some(path) => ArchiveReference::of_path(source, &path),
      None => ArchiveReference::unresolvable(name),
    };
    let shape: Option<AssetTextureShape> = reference
      .entry
      .as_deref()
      .and_then(|entry| source.get_texture_shape(entry));

    Self {
      declared: ArchiveThmDeclaredSize::of(texture_type, parameters, shape.as_ref()),
      reference,
      shape,
    }
  }
}

impl ArchiveThmDeclaredSize {
  /// The declared size, when there is a file to compare it against and the two differ.
  fn of(
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

/// The `.dds` beside a descriptor, addressed by where it sits rather than by a name the descriptor carries.
///
/// A descriptor names its bump and its detail; it does not name its own texture. The engine pairs the two by path, so
/// the sibling is the same path with the loaded extension — which also answers for a descriptor outside `textures\`,
/// where no engine reference exists to resolve.
fn to_sibling_texture_path(name: &str) -> Option<String> {
  let path: XrayLogicalPath = XrayLogicalPath::new(name).ok()?;

  path.as_str().strip_suffix(".thm").map(|stem| format!("{stem}.dds"))
}

#[cfg(test)]
mod tests {
  use xrf_db::{ThmTextureParamChunk, ThmTextureType};

  use super::{ArchiveThmDeclaredSize, to_sibling_texture_path};
  use crate::core::assets::AssetTextureShape;

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

  #[test]
  fn a_descriptor_names_the_texture_it_sits_beside() {
    assert_eq!(
      to_sibling_texture_path("textures\\act\\act_arm_1.thm").as_deref(),
      Some("textures\\act\\act_arm_1.dds")
    );
    assert_eq!(
      to_sibling_texture_path("Textures\\Act\\ACT_ARM_1.THM").as_deref(),
      Some("textures\\act\\act_arm_1.dds")
    );
  }
}
