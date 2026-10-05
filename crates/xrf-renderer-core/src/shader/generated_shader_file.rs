use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};

/// A checked-in WGSL file written from Rust declarations, as the TypeScript bindings are: a test calls `sync`, which
/// rewrites the file when it is stale and says so, so the test fails once and passes on the next run with the file
/// committed.
pub struct GeneratedShaderFile {
  path: PathBuf,
}

impl GeneratedShaderFile {
  /// The line every generated file opens with.
  pub const HEADER: &'static str =
    "// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.\n\n";

  pub fn new(path: impl Into<PathBuf>) -> Self {
    Self { path: path.into() }
  }

  pub fn get_path(&self) -> &Path {
    &self.path
  }

  /// Writes `wgsl` under the header when the file holds anything else, and answers whether it did.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read or written.
  pub fn sync(&self, wgsl: &str) -> XrfResult<bool> {
    let contents: String = format!("{}{wgsl}", Self::HEADER);

    match std::fs::read_to_string(&self.path) {
      Ok(existing) if existing.replace("\r\n", "\n") == contents => return Ok(false),
      Ok(_) => {}
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
      Err(error) => {
        return Err(XrfError::new_read_error(format!(
          "Cannot read generated shader '{}': {error}",
          self.path.display()
        )));
      }
    }

    if let Some(parent) = self.path.parent() {
      std::fs::create_dir_all(parent)
        .map_err(|error| XrfError::new_unexpected_error(format!("Cannot create '{}': {error}", parent.display())))?;
    }

    std::fs::write(&self.path, contents).map_err(|error| {
      XrfError::new_unexpected_error(format!(
        "Cannot write generated shader '{}': {error}",
        self.path.display()
      ))
    })?;

    Ok(true)
  }
}
