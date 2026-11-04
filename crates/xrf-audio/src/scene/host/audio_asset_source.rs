use xrf_error::XrfResult;

/// Where a scene's sounds come from: the host reads a sound's bytes by its reference, as the renderer's asset source
/// reads a texture. Called from the audio's loader thread.
pub trait AudioAssetSource: Send + Sync + 'static {
  /// What names the sounds it reads, so two sources reading the same files share what was decoded of them.
  fn get_sound_scope(&self) -> &str;

  /// The sound's file, `sounds\` relative and without its extension; none where it is missing, which refuses it.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read.
  fn read_sound(&self, reference: &str) -> XrfResult<Option<Vec<u8>>>;
}
