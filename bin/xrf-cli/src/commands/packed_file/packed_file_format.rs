use std::path::Path;

use xrf_error::XrfResult;
use xrf_particles::ParticlesFile;
use xrf_spawn::{AnySpawnFile, LevelSpawnExport};

use crate::commands::packed_file::PackedFileUnpacked;

/// A file the game reads packed into one binary, which the tools unpack into a folder of text files and pack back: a
/// particle library or a spawn file. The `pack`, `unpack`, `repack`, `re-unpack` and `verify` commands are written once
/// over it.
pub(crate) trait PackedFileFormat: Sized + Send + Sync + 'static {
  /// What one is called in messages and help, such as `particles file`.
  const NOUN: &'static str;

  /// How the packed file is named, such as `particles.xr`.
  const FILE_NAME: &'static str;

  /// The check `verify` reports reading one under, and the rule a file that would not read breaks.
  const READ_CHECK: &'static str;

  /// What an export says beyond the folder it wrote.
  type Unpacked: PackedFileUnpacked;

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

  /// Writes it unpacked into the folder `path`, answering what the export says beyond the folder.
  ///
  /// # Errors
  ///
  /// Returns an error when a file of the folder cannot be written.
  fn export(&self, path: &Path) -> XrfResult<Self::Unpacked>;
}

impl PackedFileFormat for ParticlesFile {
  const NOUN: &'static str = "particles file";
  const FILE_NAME: &'static str = Self::FILE_NAME;
  const READ_CHECK: &'static str = "particles.read";

  type Unpacked = ();

  fn read(path: &Path) -> XrfResult<Self> {
    Self::read_from_path(&path)
  }

  fn import(path: &Path) -> XrfResult<Self> {
    Self::import_from_path(&path)
  }

  fn write(&self, path: &Path) -> XrfResult {
    self.write_to_path(&path)
  }

  fn export(&self, path: &Path) -> XrfResult {
    self.export_to_path(&path)
  }
}

/// Either kind of `.spawn` reads, verifies, unpacks and packs, its file or its folder telling which kind it is.
impl PackedFileFormat for AnySpawnFile {
  const NOUN: &'static str = "spawn file";
  const FILE_NAME: &'static str = "*.spawn";
  const READ_CHECK: &'static str = "spawn.read";

  type Unpacked = Option<LevelSpawnExport>;

  fn read(path: &Path) -> XrfResult<Self> {
    Self::read_from_path(&path)
  }

  fn import(path: &Path) -> XrfResult<Self> {
    Self::import_from_path(&path)
  }

  fn write(&self, path: &Path) -> XrfResult {
    match self {
      Self::Root(file) => file.write_to_path(&path),
      Self::Level(file) => file.write_to_path(&path),
    }
  }

  fn export(&self, path: &Path) -> XrfResult<Self::Unpacked> {
    match self {
      Self::Root(file) => file.export_to_path(&path).map(|()| None),
      Self::Level(file) => file.export_to_path(&path).map(Some),
    }
  }
}
