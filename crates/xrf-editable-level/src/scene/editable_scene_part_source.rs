use xrf_error::XrfResult;

/// Where a scene's part files are read from: the directory beside its `.level` on the host, or any other store that
/// lists and reads them, such as a mounted search.
pub trait EditableScenePartSource {
  /// The names of the files in the scene's part directory, in any order; none where it has none.
  ///
  /// # Errors
  ///
  /// Returns an error when the directory exists and cannot be listed.
  fn list_files(&self) -> XrfResult<Vec<String>>;

  /// The bytes of the part directory's file `name`.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read.
  fn read_file(&self, name: &str) -> XrfResult<Vec<u8>>;
}
