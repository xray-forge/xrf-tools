use serde::Serialize;
use xrf_error::{XrfError, XrfResult};
use xrf_extension::{XrayExtension, XrayExtensionOf};

use crate::project::constants::{
  ALLOWED_AUDIO_EXTENSIONS, ALLOWED_AUDIO_SIZE, ALLOWED_DESCRIBE_SIZE, ALLOWED_IMAGE_EXTENSIONS, ALLOWED_IMAGE_SIZE,
  ALLOWED_TEXT_EXTENSIONS, ALLOWED_TEXT_SIZE,
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
  pub extensions: &'static [XrayExtension],
  pub maximum_size: u32,
  /// Extensions decoded into a picture. Compression does not apply: it is undone before decoding.
  pub image_extensions: &'static [XrayExtension],
  pub maximum_image_size: u32,
  /// Extensions played by the webview itself, so the backend only has to hand over the bytes.
  pub audio_extensions: &'static [XrayExtension],
  pub maximum_audio_size: u32,
  /// Ceiling on an entry read whole to describe its format.
  pub maximum_describe_size: u32,
}

impl ArchiveReadPolicy {
  /// Whether this file is one the policy reads as text.
  pub fn supports_file(&self, filename: &str) -> bool {
    XrayExtensionOf::of(filename)
      .known()
      .is_some_and(|extension| self.extensions.contains(&extension))
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

  /// Whether an entry of `size` bytes may be read whole so its format can be described.
  pub const fn allows_describe_read(&self, size: u32) -> bool {
    size <= self.maximum_describe_size
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
      maximum_describe_size: ALLOWED_DESCRIBE_SIZE,
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
      assert!(policy.supports_file(&format!("preview.{extension}")));
      assert!(policy.supports_file(&format!("preview.{}", extension.as_str().to_uppercase())));
    }

    // Textures and sounds are routed to the viewer's own reads, never read as text, which is why they are listed
    // separately and why the text gate must keep refusing them.
    assert!(!policy.supports_file("preview.dds"));
    assert!(!policy.supports_file("ambient.ogg"));
    assert!(!policy.supports_file("preview"));
  }

  #[test]
  fn the_shader_script_the_engine_ships_without_a_name_reads_as_text() {
    // `Path::extension` calls `shaders\r1\.s` a hidden file and refuses it; it is a Lua script the engine loads.
    assert!(ArchiveReadPolicy::default().supports_file("shaders\\r1\\.s"));
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

  #[test]
  fn a_describe_read_is_bounded_by_size_alone() {
    let policy: ArchiveReadPolicy = ArchiveReadPolicy::default();

    // The extension is not consulted: which formats have a describer is not a question this policy answers, so a
    // descriptor and a mesh are admitted on the same terms and refused on the same terms.
    assert!(policy.allows_describe_read(0));
    assert!(policy.allows_describe_read(policy.maximum_describe_size));
    assert!(!policy.allows_describe_read(policy.maximum_describe_size + 1));
  }
}
