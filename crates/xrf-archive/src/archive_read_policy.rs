use serde::Serialize;
use xrf_error::{XrfError, XrfResult};
use xrf_extension::{XrayExtension, XrayExtensionOf};

/// What a viewer may read out of a mounted tree, by extension and size, passed to each read it gates.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveReadPolicy {
  /// Extensions read as text and decoded from Windows-1251 for display.
  pub text_extensions: &'static [XrayExtension],
  /// Ceiling on a text read, which holds the whole entry and its decoded copy in memory.
  pub maximum_text_size: u32,
  /// Extensions decoded into a picture. Compression does not apply: it is undone before decoding.
  pub texture_extensions: &'static [XrayExtension],
  /// Ceiling on a texture entry, which is held whole in memory to decode.
  pub maximum_texture_size: u32,
  /// Extensions the webview renders itself, so the backend hands the bytes over rather than decoding them.
  pub image_extensions: &'static [XrayExtension],
  /// Ceiling on a picture handed over whole, the same bound a decoded one answers to.
  pub maximum_image_size: u32,
  /// Extensions played by the webview itself, so the backend only has to hand over the bytes.
  pub audio_extensions: &'static [XrayExtension],
  /// Ceiling on an audio entry, which is held whole in memory on the way to the webview.
  pub maximum_audio_size: u32,
  /// Ceiling on an entry read whole to describe its format.
  pub maximum_describe_size: u32,
  /// Ceiling on an entry read whole only to walk the container it is, which buys far less and so admits far less.
  pub maximum_chunk_tree_size: u32,
}

impl ArchiveReadPolicy {
  /// Whether this file is one the policy reads as text, judged by its extension alone.
  pub fn is_text_file(&self, filename: &str) -> bool {
    XrayExtensionOf::of(filename)
      .known()
      .is_some_and(|extension| self.text_extensions.contains(&extension))
  }

  /// Whether an entry of `size` bytes may be read whole so its format can be described.
  pub const fn allows_describe_read(&self, size: u32) -> bool {
    size <= self.maximum_describe_size
  }

  /// Whether an entry of `size` bytes may be read whole only to walk the container it is.
  pub const fn allows_chunk_tree_read(&self, size: u32) -> bool {
    size <= self.maximum_chunk_tree_size
  }

  /// Admits a text read of `size` bytes, or names why it is refused.
  ///
  /// # Errors
  ///
  /// Returns a read error when the extension is not one this policy reads as text, or the file exceeds its limit.
  pub fn assert_can_read_as_text(&self, filename: &str, size: u32) -> XrfResult {
    if !self.is_text_file(filename) {
      return Err(XrfError::new_read_error(format!(
        "File '{filename}' cannot be read, file extension is not allowed to be read"
      )));
    }

    if size > self.maximum_text_size {
      return Err(XrfError::new_read_error(format!(
        "File '{filename}' is too big to be read - {size}, {} is maximum allowed",
        self.maximum_text_size
      )));
    }

    Ok(())
  }
}

/// The policy the archive viewers read under, on both sides of IPC: the backend gates its reads by it, and the
/// frontend's bindings carry it as the preview rules.
///
/// `dds` is the texture the engine loads, which the backend transcodes for display; images are pictures a webview draws
/// as they stand.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const ARCHIVE_READ_POLICY: ArchiveReadPolicy = ArchiveReadPolicy {
  text_extensions: &[
    XrayExtension::Bat,
    XrayExtension::Cmd,
    XrayExtension::Cs,
    XrayExtension::Ds,
    XrayExtension::Gs,
    XrayExtension::H,
    XrayExtension::Hlsl,
    XrayExtension::Hs,
    XrayExtension::Ini,
    XrayExtension::Json,
    XrayExtension::Log,
    XrayExtension::Ltx,
    XrayExtension::Lua,
    XrayExtension::Md,
    XrayExtension::Ps,
    XrayExtension::Py,
    XrayExtension::S,
    XrayExtension::Script,
    XrayExtension::Seq,
    XrayExtension::Txt,
    XrayExtension::Vs,
    XrayExtension::Xml,
  ],
  maximum_text_size: 10 * 1024 * 1024,
  texture_extensions: &[XrayExtension::Dds],
  maximum_texture_size: 32 * 1024 * 1024,
  image_extensions: &[
    XrayExtension::Bmp,
    XrayExtension::Jpeg,
    XrayExtension::Jpg,
    XrayExtension::Png,
  ],
  maximum_image_size: 32 * 1024 * 1024,
  audio_extensions: &[XrayExtension::Ogg, XrayExtension::Wav],
  maximum_audio_size: 64 * 1024 * 1024,
  maximum_describe_size: 64 * 1024 * 1024,
  maximum_chunk_tree_size: 8 * 1024 * 1024,
};

#[cfg(test)]
mod tests {
  use super::{ARCHIVE_READ_POLICY, ArchiveReadPolicy};

  #[test]
  fn default_policy_recognizes_text_extensions_case_insensitively() {
    let policy: ArchiveReadPolicy = ARCHIVE_READ_POLICY;

    for extension in policy.text_extensions {
      assert!(policy.is_text_file(&format!("preview.{extension}")));
      assert!(policy.is_text_file(&format!("preview.{}", extension.as_str().to_uppercase())));
    }

    // Textures and sounds are routed to the viewer's own reads, never read as text, which is why they are listed
    // separately and why the text gate must keep refusing them.
    assert!(!policy.is_text_file("preview.dds"));
    assert!(!policy.is_text_file("ambient.ogg"));
    assert!(!policy.is_text_file("preview"));
  }

  #[test]
  fn every_stage_of_a_shader_source_reads_as_text() {
    // Written as the whole family on purpose. `.ps`, `.vs`, `.hs` and `.ds` were readable while `.gs`, `.cs` and the
    // `.hlsl` IX-Ray ships loose - 501 of them - were not, which is an oversight no wording of the list prevents.
    let policy: ArchiveReadPolicy = ARCHIVE_READ_POLICY;

    for stage in ["vs", "hs", "ds", "gs", "ps", "cs", "hlsl"] {
      assert!(
        policy.is_text_file(&format!("shaders\\r3\\shader.{stage}")),
        "a '.{stage}' source is text like every other stage"
      );
    }
  }

  #[test]
  fn the_scripting_the_engine_runs_reads_as_text() {
    // Two spellings of one language: `.script` is what the game data uses and `.lua` is what an engine tree does.
    let policy: ArchiveReadPolicy = ARCHIVE_READ_POLICY;

    assert!(policy.is_text_file("scripts\\bind_stalker.script"));
    assert!(policy.is_text_file("scripts\\bind_stalker.lua"));
  }

  #[test]
  fn the_shader_script_the_engine_ships_without_a_name_reads_as_text() {
    // `Path::extension` calls `shaders\r1\.s` a hidden file and refuses it; it is a Lua script the engine loads.
    assert!(ARCHIVE_READ_POLICY.is_text_file("shaders\\r1\\.s"));
  }

  #[test]
  fn a_text_read_is_refused_by_extension_and_by_size() {
    let policy: ArchiveReadPolicy = ARCHIVE_READ_POLICY;

    assert!(
      policy
        .assert_can_read_as_text("configs\\system.ltx", policy.maximum_text_size)
        .is_ok()
    );
    assert!(policy.assert_can_read_as_text("textures\\a.dds", 1).is_err());
    assert!(
      policy
        .assert_can_read_as_text("configs\\system.ltx", policy.maximum_text_size + 1)
        .is_err()
    );
  }

  #[test]
  fn a_describe_read_is_bounded_by_size_alone() {
    let policy: ArchiveReadPolicy = ARCHIVE_READ_POLICY;

    // The extension is not consulted: which formats have a describer is not a question this policy answers, so a
    // descriptor and a mesh are admitted on the same terms and refused on the same terms.
    assert!(policy.allows_describe_read(0));
    assert!(policy.allows_describe_read(policy.maximum_describe_size));
    assert!(!policy.allows_describe_read(policy.maximum_describe_size + 1));
  }
}
