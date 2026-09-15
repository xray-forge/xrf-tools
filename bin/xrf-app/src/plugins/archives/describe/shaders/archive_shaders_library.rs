use std::collections::HashSet;

use serde::Serialize;

use crate::plugins::archives::describe::archive_reference::ArchiveReferenceStatus;
use crate::plugins::archives::describe::shaders::archive_shaders_blender::ArchiveShadersBlender;

/// What the library holds, taken over the whole of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveShadersLibrary {
  pub blenders: usize,
  /// Distinct class tags, of which the workspace libraries all use fifteen.
  pub classes: usize,
  /// Distinct textures the blenders bind by name, slots the renderer fills excluded.
  pub textures: usize,
  /// Distinct named textures the subject being browsed does not hold.
  pub absent_textures: usize,
}

impl ArchiveShadersLibrary {
  /// The totals of a library, taken off the blenders already described.
  pub fn of(blenders: &[ArchiveShadersBlender]) -> Self {
    let mut classes: HashSet<&str> = HashSet::new();
    let mut textures: HashSet<&str> = HashSet::new();
    let mut absent: HashSet<&str> = HashSet::new();

    for blender in blenders {
      classes.insert(blender.class.as_str());

      for property in &blender.properties {
        if let Some(texture) = &property.texture {
          textures.insert(texture.name.as_str());

          if texture.status != ArchiveReferenceStatus::Present {
            absent.insert(texture.name.as_str());
          }
        }
      }
    }

    Self {
      blenders: blenders.len(),
      classes: classes.len(),
      textures: textures.len(),
      absent_textures: absent.len(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveShadersLibrary;
  use crate::plugins::archives::describe::archive_reference::{ArchiveReference, ArchiveReferenceStatus};
  use crate::plugins::archives::describe::shaders::archive_shaders_blender::ArchiveShadersBlender;
  use crate::plugins::archives::describe::shaders::archive_shaders_property::ArchiveShadersProperty;

  fn blender(class: &str, texture: Option<&str>, status: ArchiveReferenceStatus) -> ArchiveShadersBlender {
    ArchiveShadersBlender {
      name: String::from("models\\model"),
      class: String::from(class),
      version: 2,
      computer: String::new(),
      time: 0,
      properties: vec![ArchiveShadersProperty {
        name: String::from("Name"),
        kind: String::from("Texture"),
        value: texture.unwrap_or_default().to_owned(),
        texture: texture.map(|name| ArchiveReference {
          name: name.to_owned(),
          path: Some(format!("textures\\{name}.dds")),
          entry: None,
          status,
        }),
      }],
    }
  }

  #[test]
  fn a_library_counts_its_classes_and_its_textures_once_each() {
    let library: ArchiveShadersLibrary = ArchiveShadersLibrary::of(&[
      blender("MODEL", Some("detail\\grass"), ArchiveReferenceStatus::Present),
      blender("MODEL", Some("detail\\grass"), ArchiveReferenceStatus::Present),
      blender("LM", Some("detail\\gone"), ArchiveReferenceStatus::Absent),
      blender("LM", None, ArchiveReferenceStatus::Present),
    ]);

    assert_eq!(library.blenders, 4);
    assert_eq!(library.classes, 2);
    assert_eq!(library.textures, 2);
    assert_eq!(library.absent_textures, 1);
  }
}
