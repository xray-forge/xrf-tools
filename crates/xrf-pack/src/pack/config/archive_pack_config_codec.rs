use std::fs;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::Ltx;
use xrf_utils::{format_path, write_file_staged};

use crate::pack::config::ArchivePackConfig;
use crate::pack::config::ArchivePackConfigFormat;
use crate::pack::config::ArchivePackConfigJson;

impl ArchivePackConfig {
  /// Apply a packing configuration file, in whichever format its extension names.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for a path whose extension names no supported format, and a read or parse error for a
  /// file that cannot be read as the format it claims.
  pub fn with_config_file<P: AsRef<Path>>(self, path: P) -> XrfResult<Self> {
    let path: &Path = path.as_ref();

    match ArchivePackConfigFormat::from_path(path)? {
      ArchivePackConfigFormat::Ltx => self.with_ltx(&Ltx::read_from_file_standard(path)?),
      ArchivePackConfigFormat::Json => {
        let json: ArchivePackConfigJson = ArchivePackConfigJson::parse(&fs::read(path)?).map_err(|error| {
          XrfError::new_parsing_error(format!(
            "Failed to parse packing configuration '{}': {error}",
            format_path(path)
          ))
        })?;

        Ok(self.with_json(&json))
      }
    }
  }

  /// Write the file-owned fields out as a packing configuration file, in whichever format the path names.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for a path whose extension names no supported format, and an IO error when the
  /// configuration cannot be published.
  pub fn write_config_to_path<P: AsRef<Path>>(&self, path: P) -> XrfResult {
    let path: &Path = path.as_ref();

    let rendered: Vec<u8> = match ArchivePackConfigFormat::from_path(path)? {
      ArchivePackConfigFormat::Ltx => self.to_ltx_bytes()?,
      ArchivePackConfigFormat::Json => self.to_json().render()?.into_bytes(),
    };

    write_file_staged(path, &rendered).map_err(|error| {
      XrfError::new_io_error(
        format!(
          "Failed to write the packing configuration to '{}': {error}",
          format_path(path)
        ),
        error.kind(),
      )
    })
  }
}
