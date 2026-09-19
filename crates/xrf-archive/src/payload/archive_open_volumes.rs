use std::fs::File;
use std::path::Path;

use xrf_error::XrfResult;

use crate::payload::archive_located_entry::ArchiveLocatedEntry;
use crate::payload::archive_open_volume::ArchiveOpenVolume;
use crate::{ArchiveDescriptor, ArchiveFileDescriptor, ArchiveProject};

/// Every volume of one project, open, positioned to serve any entry of it.
pub struct ArchiveOpenVolumes<'a> {
  project: &'a ArchiveProject,
  /// One open handle per volume, positionally matching [`ArchiveProject::archives`].
  files: Vec<ArchiveOpenVolume>,
}

impl<'a> ArchiveOpenVolumes<'a> {
  pub(crate) fn open(project: &'a ArchiveProject) -> XrfResult<Self> {
    let mut files: Vec<ArchiveOpenVolume> = Vec::with_capacity(project.archives.len());

    for archive in &project.archives {
      files.push(ArchiveOpenVolume::open(&archive.path)?);
    }

    Ok(Self { project, files })
  }

  /// The project these volumes belong to.
  pub fn get_project(&self) -> &ArchiveProject {
    self.project
  }

  /// Where an entry unpacks to, relative to a destination root, from its volume's `entry_point`.
  ///
  /// # Errors
  ///
  /// Returns a read error when the entry names a volume outside this project.
  pub fn get_unpack_root_of(&self, descriptor: &ArchiveFileDescriptor) -> XrfResult<&Path> {
    Ok(&self.project.get_volume_of(descriptor)?.output_root_path)
  }

  /// Reads one entry into memory, decompressing it when it is stored compressed.
  ///
  /// # Errors
  ///
  /// Returns a read error when the entry declares bytes past its volume's end, its payload cannot be decompressed, or
  /// it fails the checksum its volume recorded.
  pub fn read_bytes(&self, descriptor: &ArchiveFileDescriptor) -> XrfResult<Vec<u8>> {
    self.locate(descriptor)?.read_bytes()
  }

  /// Copies one entry into an already opened target, decompressing when it is stored compressed.
  ///
  /// # Errors
  ///
  /// Returns a read error when the entry declares bytes past its volume's end, and an IO error when the copy fails.
  pub fn write_contents(&self, target: &mut File, descriptor: &ArchiveFileDescriptor) -> XrfResult {
    self.locate(descriptor)?.write_contents(target)
  }

  fn locate<'b>(&'b self, descriptor: &'b ArchiveFileDescriptor) -> XrfResult<ArchiveLocatedEntry<'b>> {
    let volume: &ArchiveDescriptor = self.project.get_volume_of(descriptor)?;

    self.files[descriptor.volume as usize].locate(volume, descriptor)
  }
}
