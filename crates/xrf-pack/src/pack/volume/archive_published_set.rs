use std::fs;
use std::fs::{DirEntry, ReadDir};
use std::io::ErrorKind;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_utils::format_path;

use crate::pack::config::ArchivePackConfig;

/// The volumes of one named set already sitting in a destination.
///
/// Packing publishes `<name>.<ext>` and `<name>.<ext><N>`, so those names decide both questions this answers: whether
/// a run would replace an archive the caller already had, and — once a run that would not has failed — which files in
/// the destination are its own to take back.
///
/// Scoped to the configured extension. A `gamedata.xdb0` beside a `gamedata.db0` is a merged mount the engine will
/// read oddly, but it is not something packing would overwrite, so refusing it here would refuse a set this run never
/// touches.
pub(crate) struct ArchivePublishedSet {
  volumes: Vec<PathBuf>,
}

impl ArchivePublishedSet {
  /// Read what the destination already holds under this configuration's set name.
  ///
  /// A destination that is not there yet holds nothing; any other listing failure is reported, because a destination
  /// that cannot be listed cannot be packed into either and saying so here names the real problem.
  pub(crate) fn read(config: &ArchivePackConfig) -> XrfResult<Self> {
    let prefix: String = format!("{}.{}", config.name, config.volume_extension.as_str());
    let entries: ReadDir = match fs::read_dir(&config.destination) {
      Ok(entries) => entries,
      Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self { volumes: Vec::new() }),
      Err(error) => return Err(error.into()),
    };

    let mut volumes: Vec<PathBuf> = Vec::new();

    for entry in entries {
      let entry: DirEntry = entry?;

      // Only files: a directory that happens to be named like a volume is not one, and removing it on a failed run
      // would take whatever it holds with it.
      if !entry.file_type()?.is_file() {
        continue;
      }

      if Self::is_volume_named(&entry.file_name().to_string_lossy(), &prefix) {
        volumes.push(entry.path());
      }
    }

    volumes.sort();

    Ok(Self { volumes })
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.volumes.is_empty()
  }

  pub(crate) fn get_volumes(&self) -> &[PathBuf] {
    &self.volumes
  }

  /// Delete every volume of the set, reporting nothing.
  ///
  /// For a failed run whose destination held no set of this name when it began: every one of these files is then its
  /// own, and removing them puts the destination back. Best effort by design — it runs while an error is already on
  /// its way to the caller, and a cleanup that could fail the run would replace a real diagnosis with its own.
  pub(crate) fn remove_all(&self) {
    for volume in &self.volumes {
      if let Err(error) = fs::remove_file(volume) {
        log::warn!(
          "Could not remove '{}' left by a failed archive pack: {error}",
          format_path(volume)
        );
      }
    }
  }

  /// Whether a file name is `<prefix>` itself or `<prefix>` followed only by a volume index.
  ///
  /// Compared without ASCII case, matching how the destination lease and the engine's own mount treat these names, and
  /// compared as bytes so a name that is not ASCII is neither mangled nor split on a character boundary.
  fn is_volume_named(file_name: &str, prefix: &str) -> bool {
    let name: &[u8] = file_name.as_bytes();
    let prefix: &[u8] = prefix.as_bytes();

    name.len() >= prefix.len()
      && name[..prefix.len()].eq_ignore_ascii_case(prefix)
      && name[prefix.len()..].iter().all(u8::is_ascii_digit)
  }
}
