use std::path::Path;

use serde::Serialize;

use crate::project::constants::{
  ALLOWED_PROJECT_AUDIO_EXTENSIONS, ALLOWED_PROJECT_AUDIO_SIZE, ALLOWED_PROJECT_IMAGE_EXTENSIONS,
  ALLOWED_PROJECT_IMAGE_SIZE, ALLOWED_PROJECT_READ_EXTENSIONS, ALLOWED_PROJECT_READ_SIZE,
};

/// What an archive viewer may read out of a project, by extension and size.
///
/// A gate for interactive consumers rather than a format rule: [`crate::ArchiveProject::read_file_bytes`] ignores it,
/// while [`crate::ArchiveProject::read_file_as_string`] refuses what the policy does not cover.
///
/// Only the text lists are enforced here. The picture and sound lists are routing hints for the viewer, which reads
/// both through the shared mounted assets rather than through this project, and so answers to no limit of its own.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveProjectReadPolicy {
  pub extensions: &'static [&'static str],
  pub maximum_size: u32,
  /// Extensions decoded into a picture. Compression does not apply: it is undone before decoding.
  pub image_extensions: &'static [&'static str],
  pub maximum_image_size: u32,
  /// Extensions played by the webview itself, so the backend only has to hand over the bytes.
  pub audio_extensions: &'static [&'static str],
  pub maximum_audio_size: u32,
}

impl ArchiveProjectReadPolicy {
  /// Whether this file is one the policy reads as text.
  pub fn supports_file(&self, filename: &str) -> bool {
    Self::has_extension(filename, self.extensions)
  }

  fn has_extension(filename: &str, extensions: &[&str]) -> bool {
    Path::new(filename)
      .extension()
      .and_then(|extension| extension.to_str())
      .is_some_and(|extension| extensions.iter().any(|allowed| extension.eq_ignore_ascii_case(allowed)))
  }
}

impl Default for ArchiveProjectReadPolicy {
  fn default() -> Self {
    Self {
      extensions: ALLOWED_PROJECT_READ_EXTENSIONS,
      maximum_size: ALLOWED_PROJECT_READ_SIZE,
      image_extensions: ALLOWED_PROJECT_IMAGE_EXTENSIONS,
      maximum_image_size: ALLOWED_PROJECT_IMAGE_SIZE,
      audio_extensions: ALLOWED_PROJECT_AUDIO_EXTENSIONS,
      maximum_audio_size: ALLOWED_PROJECT_AUDIO_SIZE,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveProjectReadPolicy;

  #[test]
  fn default_policy_recognizes_supported_extensions_case_insensitively() {
    let policy: ArchiveProjectReadPolicy = ArchiveProjectReadPolicy::default();

    for extension in policy.extensions {
      assert!(policy.supports_file(&format!("preview.{}", extension)));
      assert!(policy.supports_file(&format!("preview.{}", extension.to_uppercase())));
    }

    // Textures and sounds are routed to the viewer's own reads, never read as text, which is why they are listed
    // separately and why the text gate must keep refusing them.
    assert!(!policy.supports_file("preview.dds"));
    assert!(!policy.supports_file("ambient.ogg"));
    assert!(!policy.supports_file("preview"));
  }
}
