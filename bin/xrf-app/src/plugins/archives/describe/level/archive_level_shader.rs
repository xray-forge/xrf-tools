use serde::Serialize;
use xrf_shaders::ShaderLibraryFile;

use crate::plugins::archives::describe::archive_reference::ArchiveReferenceStatus;

/// The blender a surface draws with, and whether the subject being browsed defines it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelShader {
  pub name: String,
  pub status: ArchiveReferenceStatus,
}

impl ArchiveLevelShader {
  /// One shader name, asked of the library when the subject holds one.
  pub fn of(name: &str, library: Option<&ShaderLibraryFile>) -> Self {
    Self {
      name: name.to_owned(),
      status: match library {
        None => ArchiveReferenceStatus::Unknown,
        Some(library) if library.contains_blender(name) => ArchiveReferenceStatus::Present,
        Some(_) => ArchiveReferenceStatus::Absent,
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_shaders::ShaderLibraryFile;

  use super::ArchiveLevelShader;
  use crate::plugins::archives::describe::archive_reference::ArchiveReferenceStatus;

  #[test]
  fn a_name_no_library_can_be_asked_about_is_unknown_rather_than_absent() {
    // An absence nobody looked for is the signal decision 10 exists to keep out of the view.
    assert_eq!(
      ArchiveLevelShader::of("default", None).status,
      ArchiveReferenceStatus::Unknown
    );
  }

  #[test]
  fn a_name_an_open_library_does_not_define_is_absent() {
    assert_eq!(
      ArchiveLevelShader::of("default", Some(&ShaderLibraryFile::default())).status,
      ArchiveReferenceStatus::Absent
    );
  }
}
