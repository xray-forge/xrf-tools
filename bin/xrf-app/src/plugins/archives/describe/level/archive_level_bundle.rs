use std::collections::HashSet;

use serde::Serialize;

use crate::plugins::archives::describe::archive_reference::{ArchiveReference, ArchiveReferenceStatus};
use crate::plugins::archives::describe::level::archive_level_surface::{ArchiveLevelEntry, ArchiveLevelSurface};

/// What the bundle holds, taken over the whole of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelBundle {
  /// Compiler version the bundle was built by, `hdrLEVEL::XRLC_version`.
  pub xrlc_version: u16,
  /// Compiler quality the build ran at, `hdrLEVEL::XRLC_quality`.
  pub xrlc_quality: u16,
  /// Rows of the shader table, which is what a face addresses by position.
  pub surfaces: usize,
  /// Distinct shader names the surfaces draw with.
  pub shaders: usize,
  /// Distinct shader names no open library defines. Zero while there is no library to ask.
  pub undefined_shaders: usize,
  /// Distinct textures the surfaces bind.
  pub textures: usize,
  /// Distinct textures the subject being browsed does not hold.
  pub absent_textures: usize,
  /// The blender library the shader names were asked of, absent when the subject holds none.
  pub library: Option<ArchiveReference>,
  /// Whether the file declares a shader table at all.
  pub has_shader_table: bool,
}

impl ArchiveLevelBundle {
  /// The totals of a bundle, taken off the surfaces already described.
  pub fn of(
    xrlc_version: u16,
    xrlc_quality: u16,
    surfaces: &[ArchiveLevelSurface],
    library: Option<ArchiveReference>,
    has_shader_table: bool,
  ) -> Self {
    let mut shaders: HashSet<&str> = HashSet::new();
    let mut undefined: HashSet<&str> = HashSet::new();
    let mut textures: HashSet<&str> = HashSet::new();
    let mut absent: HashSet<&str> = HashSet::new();

    for surface in surfaces {
      if let ArchiveLevelEntry::Drawn {
        shader,
        textures: bound,
      } = &surface.entry
      {
        shaders.insert(shader.name.as_str());

        if shader.status == ArchiveReferenceStatus::Absent {
          undefined.insert(shader.name.as_str());
        }

        for texture in bound {
          textures.insert(texture.name.as_str());

          if texture.status != ArchiveReferenceStatus::Present {
            absent.insert(texture.name.as_str());
          }
        }
      }
    }

    Self {
      xrlc_version,
      xrlc_quality,
      surfaces: surfaces.len(),
      shaders: shaders.len(),
      undefined_shaders: undefined.len(),
      textures: textures.len(),
      absent_textures: absent.len(),
      library,
      has_shader_table,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveLevelBundle;
  use crate::plugins::archives::describe::archive_reference::{ArchiveReference, ArchiveReferenceStatus};
  use crate::plugins::archives::describe::level::archive_level_shader::ArchiveLevelShader;
  use crate::plugins::archives::describe::level::archive_level_surface::{ArchiveLevelEntry, ArchiveLevelSurface};

  fn texture(name: &str, status: ArchiveReferenceStatus) -> ArchiveReference {
    ArchiveReference {
      name: name.to_owned(),
      path: Some(format!("textures\\{name}.dds")),
      entry: None,
      status,
    }
  }

  fn drawn(
    index: usize,
    shader: &str,
    status: ArchiveReferenceStatus,
    textures: Vec<ArchiveReference>,
  ) -> ArchiveLevelSurface {
    ArchiveLevelSurface {
      index,
      entry: ArchiveLevelEntry::Drawn {
        shader: ArchiveLevelShader {
          name: shader.to_owned(),
          status,
        },
        textures,
      },
    }
  }

  #[test]
  fn a_bundle_counts_every_name_once_however_many_surfaces_use_it() {
    let bundle: ArchiveLevelBundle = ArchiveLevelBundle::of(
      14,
      2,
      &[
        ArchiveLevelSurface {
          index: 0,
          entry: ArchiveLevelEntry::Skipped,
        },
        drawn(
          1,
          "default",
          ArchiveReferenceStatus::Present,
          vec![texture("wall", ArchiveReferenceStatus::Present)],
        ),
        drawn(
          2,
          "default",
          ArchiveReferenceStatus::Present,
          vec![texture("wall", ArchiveReferenceStatus::Present)],
        ),
        drawn(
          3,
          "flora\\leaf_wave",
          ArchiveReferenceStatus::Absent,
          vec![texture("leaf", ArchiveReferenceStatus::Absent)],
        ),
      ],
      None,
      true,
    );

    assert_eq!(bundle.surfaces, 4);
    assert_eq!(bundle.shaders, 2);
    assert_eq!(bundle.undefined_shaders, 1);
    assert_eq!(bundle.textures, 2);
    assert_eq!(bundle.absent_textures, 1);
  }

  #[test]
  fn a_shader_nothing_was_asked_about_is_not_counted_as_undefined() {
    let bundle: ArchiveLevelBundle = ArchiveLevelBundle::of(
      14,
      2,
      &[drawn(0, "default", ArchiveReferenceStatus::Unknown, Vec::new())],
      None,
      true,
    );

    assert_eq!(bundle.shaders, 1);
    assert_eq!(bundle.undefined_shaders, 0);
  }
}
