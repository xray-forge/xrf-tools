use std::path::Path;

/// What a format crate's `write_to_path` promises when its staged write fails: the file a caller had is still the file
/// they have, and no hidden sibling is left for them to find. Pair it with `xrf_utils::StagingFaults`, which arms the
/// failure.
pub struct RefusedPublication;

impl RefusedPublication {
  /// Assert that a publication refused part way left `path` holding `previous` and nothing staged beside it.
  ///
  /// # Panics
  ///
  /// Panics when the file changed or its directory holds anything else.
  pub fn assert_left_previous(path: &Path, previous: &[u8]) {
    assert_eq!(
      std::fs::read(path).expect("Expected the previous file to remain"),
      previous,
      "a refused publication must leave the previous file"
    );

    let names: Vec<String> = std::fs::read_dir(path.parent().expect("Parent directory"))
      .expect("Expected the directory to be readable")
      .filter_map(Result::ok)
      .map(|entry| entry.file_name().to_string_lossy().to_string())
      .collect();

    assert_eq!(
      names,
      vec![path.file_name().expect("File name").to_string_lossy().to_string()],
      "a refused publication must clean up what it staged"
    );
  }
}
