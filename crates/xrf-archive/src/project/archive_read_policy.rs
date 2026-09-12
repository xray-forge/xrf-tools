use std::path::Path;

use serde::Serialize;
use xrf_error::{XrfError, XrfResult};

use crate::project::constants::{
  ALLOWED_AUDIO_EXTENSIONS, ALLOWED_AUDIO_SIZE, ALLOWED_IMAGE_EXTENSIONS, ALLOWED_IMAGE_SIZE, ALLOWED_TEXT_EXTENSIONS,
  ALLOWED_TEXT_SIZE,
};

/// What a viewer may read out of a mounted tree, by extension and size.
///
/// A gate for interactive consumers rather than a format rule: [`crate::ArchiveProject::read_file_bytes`] ignores it,
/// while every text read asks [`Self::require_text_read`] first. Both of the archives explorer's subjects answer to
/// this one policy, so a file too large to preview is refused the same way whichever tree it came from.
///
/// Only the text lists are enforced here. The picture and sound lists are routing hints for the viewer, which reads
/// both through the shared mounted assets and so answers to no limit of its own.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveReadPolicy {
  pub extensions: &'static [&'static str],
  pub maximum_size: u32,
  /// Extensions decoded into a picture. Compression does not apply: it is undone before decoding.
  pub image_extensions: &'static [&'static str],
  pub maximum_image_size: u32,
  /// Extensions played by the webview itself, so the backend only has to hand over the bytes.
  pub audio_extensions: &'static [&'static str],
  pub maximum_audio_size: u32,
}

impl ArchiveReadPolicy {
  /// Whether this file is one the policy reads as text.
  pub fn supports_file(&self, filename: &str) -> bool {
    Self::has_extension(filename, self.extensions)
  }

  /// Admits a text read of `size` bytes, or names why it is refused.
  ///
  /// Asked before the bytes are fetched, so an enormous entry is never held in memory to be rejected.
  ///
  /// # Errors
  ///
  /// Returns a read error when the extension is not one this policy reads as text, or the file exceeds its limit.
  pub fn require_text_read(&self, filename: &str, size: u32) -> XrfResult {
    if !self.supports_file(filename) {
      return Err(XrfError::new_read_error(format!(
        "File '{filename}' cannot be read, file extension is not allowed to be read"
      )));
    }

    if size > self.maximum_size {
      return Err(XrfError::new_read_error(format!(
        "File '{filename}' is too big to be read - {size}, {} is maximum allowed",
        self.maximum_size
      )));
    }

    Ok(())
  }

  fn has_extension(filename: &str, extensions: &[&str]) -> bool {
    Path::new(filename)
      .extension()
      .and_then(|extension| extension.to_str())
      .is_some_and(|extension| extensions.iter().any(|allowed| extension.eq_ignore_ascii_case(allowed)))
  }
}

impl Default for ArchiveReadPolicy {
  fn default() -> Self {
    Self {
      extensions: ALLOWED_TEXT_EXTENSIONS,
      maximum_size: ALLOWED_TEXT_SIZE,
      image_extensions: ALLOWED_IMAGE_EXTENSIONS,
      maximum_image_size: ALLOWED_IMAGE_SIZE,
      audio_extensions: ALLOWED_AUDIO_EXTENSIONS,
      maximum_audio_size: ALLOWED_AUDIO_SIZE,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveReadPolicy;

  #[test]
  fn default_policy_recognizes_supported_extensions_case_insensitively() {
    let policy: ArchiveReadPolicy = ArchiveReadPolicy::default();

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

  #[test]
  fn a_text_read_is_refused_by_extension_and_by_size() {
    let policy: ArchiveReadPolicy = ArchiveReadPolicy::default();

    assert!(
      policy
        .require_text_read("configs\\system.ltx", policy.maximum_size)
        .is_ok()
    );
    assert!(policy.require_text_read("textures\\a.dds", 1).is_err());
    assert!(
      policy
        .require_text_read("configs\\system.ltx", policy.maximum_size + 1)
        .is_err()
    );
  }
}
