use xrf_error::XrfResult;

/// Where a scene's sounds come from: the host reads a sound's bytes by its reference, as the renderer's asset source
/// reads a texture, on the audio's loader thread.
pub trait AudioAssetSource: Send + Sync + 'static {
  /// What names the sounds it reads, so two sources reading the same files share what was decoded of them.
  fn get_sound_scope(&self) -> &str;

  /// Which version of the sound's file it would read now, changing whenever the file changes, comes or goes: a sound
  /// read under one version is read again when asked for under another, so a file edited, or one refused as missing
  /// that has since appeared, plays as it now is. None, the default, for a source whose files hold still while it is
  /// open. Called from the audio's control thread whenever an emitter first asks for the sound.
  fn get_sound_version(&self, reference: &str) -> Option<u64> {
    let _ = reference;

    None
  }

  /// The sound's file, `sounds\` relative and without its extension; none where it is missing, which refuses it.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read.
  fn read_sound(&self, reference: &str) -> XrfResult<Option<Vec<u8>>>;
}
