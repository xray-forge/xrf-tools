use serde::Serialize;
use xrf_app_core::assets::AssetTextureShape;
use xrf_thm::{ThmTextureParamChunk, ThmTextureType};
use xrf_vfs::XrayLogicalPath;

use crate::describe::description::ArchiveReference;
use crate::describe::source::ArchiveDescribeSource;
use crate::describe::thm::archive_thm_declared_size::ArchiveThmDeclaredSize;

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

/// The `.dds` beside a descriptor, addressed by where it sits rather than by a name the descriptor carries.
fn to_sibling_texture_path(name: &str) -> Option<String> {
  let path: XrayLogicalPath = XrayLogicalPath::new(name).ok()?;

  path.as_str().strip_suffix(".thm").map(|stem| format!("{stem}.dds"))
}

#[cfg(test)]
mod tests {
  use super::to_sibling_texture_path;

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
