use std::path::Path;

use xrf_error::XrfResult;
use xrf_particles::ParticlesFile;
use xrf_spawn::{SpawnFile, XRayByteOrder};

/// A file the game reads packed into one binary, which the tools unpack into a folder of text files and pack back: a
/// particle library or a spawn file. The `pack`, `unpack`, `repack`, `re-unpack` and `verify` commands are written once
/// over it.
pub trait PackedFileFormat: Sized + Send + Sync + 'static {
  /// What one is called in messages and help, such as `particles file`.
  const NOUN: &'static str;

  /// How the packed file is named, such as `particles.xr`.
  const FILE_NAME: &'static str;

  /// The packed file at `path`.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read or is not one.
  fn read(path: &Path) -> XrfResult<Self>;

  /// The unpacked folder at `path`.
  ///
  /// # Errors
  ///
  /// Returns an error when the folder cannot be read or does not hold one.
  fn import(path: &Path) -> XrfResult<Self>;

  /// Writes it packed to `path`, staged beside it so a failure leaves whatever was there.
  ///
  /// # Errors
  ///
  /// Returns an error when it cannot be encoded or written.
  fn write(&self, path: &Path) -> XrfResult;

  /// Writes it unpacked into the folder `path`.
  ///
  /// # Errors
  ///
  /// Returns an error when a file of the folder cannot be written.
  fn export(&self, path: &Path) -> XrfResult;
}

impl PackedFileFormat for ParticlesFile {
  const NOUN: &'static str = "particles file";
  const FILE_NAME: &'static str = "particles.xr";

  fn read(path: &Path) -> XrfResult<Self> {
    Self::read_from_path::<XRayByteOrder, _>(&path)
  }

  fn import(path: &Path) -> XrfResult<Self> {
    Self::import_from_path(&path)
  }

  fn write(&self, path: &Path) -> XrfResult {
    self.write_to_path::<XRayByteOrder, _>(&path)
  }

  fn export(&self, path: &Path) -> XrfResult {
    self.export_to_path(&path)
  }
}

impl PackedFileFormat for SpawnFile {
  const NOUN: &'static str = "spawn file";
  const FILE_NAME: &'static str = "*.spawn";

  fn read(path: &Path) -> XrfResult<Self> {
    Self::read_from_path::<XRayByteOrder, _>(&path)
  }

  fn import(path: &Path) -> XrfResult<Self> {
    Self::import_from_path::<XRayByteOrder, _>(&path)
  }

  fn write(&self, path: &Path) -> XrfResult {
    self.write_to_path::<XRayByteOrder, _>(&path)
  }

  fn export(&self, path: &Path) -> XrfResult {
    self.export_to_path::<XRayByteOrder, _>(&path)
  }
}
