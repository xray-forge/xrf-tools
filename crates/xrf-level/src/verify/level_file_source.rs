use xrf_error::XrfResult;

/// Where a compiled level's files are read from, so the verifier reads the same level loose or archived.
pub trait LevelFileSource {
  /// The level's file `name`, such as `level.geom`, or none where the level ships without it.
  ///
  /// # Errors
  ///
  /// Returns an error when the file is there and cannot be read.
  fn read(&self, name: &str) -> XrfResult<Option<Vec<u8>>>;

  /// The level's file `name` as a finding names its subject.
  fn describe(&self, name: &str) -> String;
}
