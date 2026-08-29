//! Synthetic archive projects the unpack tests are written against.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crc32fast::hash;
use xrf_archive::ArchiveFileDescriptor;
use xrf_archive::ArchiveProject;
use xrf_archive::ArchiveProjectReadPolicy;

pub(crate) struct Entry {
  name: &'static str,
  contents: &'static [u8],
  /// Stored LZO compressed, the way real archives hold most of their payload.
  is_compressed: bool,
}

impl Entry {
  pub(crate) fn stored(name: &'static str, contents: &'static [u8]) -> Self {
    Self {
      name,
      contents,
      is_compressed: false,
    }
  }

  pub(crate) fn compressed(name: &'static str, contents: &'static [u8]) -> Self {
    Self {
      name,
      contents,
      is_compressed: true,
    }
  }
}

/// Lay out entries end to end in one file and describe them, the way an archive stores its payload.
pub(crate) fn create_project(directory: &Path, entries: &[Entry]) -> ArchiveProject {
  let source: PathBuf = directory.join("files.db0");

  let mut payload: Vec<u8> = Vec::new();
  let mut files: HashMap<String, ArchiveFileDescriptor> = HashMap::new();

  for entry in entries {
    let offset: u32 = payload.len() as u32;

    // A compressed entry stores fewer bytes than it yields, which is exactly the case the reader used
    // to get wrong by copying `size_real` bytes straight out of the archive.
    let stored: Vec<u8> = if entry.is_compressed {
      lzokay::compress::compress(entry.contents).expect("lzo compression")
    } else {
      entry.contents.to_vec()
    };

    payload.extend_from_slice(&stored);

    let mut descriptor: ArchiveFileDescriptor = ArchiveFileDescriptor::new(
      hash(entry.contents),
      entry.name.into(),
      offset,
      stored.len() as u32,
      entry.contents.len() as u32,
    );

    descriptor.source = source.clone();

    files.insert(entry.name.into(), descriptor);
  }

  fs::File::create(&source)
    .expect("test archive file")
    .write_all(&payload)
    .expect("test archive payload");

  ArchiveProject {
    archives: Vec::new(),
    files,
    read_policy: ArchiveProjectReadPolicy::default(),
    root: directory.into(),
    size_real: payload.len() as u64,
  }
}

pub(crate) fn create_temporary_directory(name: &str) -> PathBuf {
  let directory: PathBuf = std::env::temp_dir().join(format!("xrf-archive-{name}"));

  let _ = fs::remove_dir_all(&directory);

  fs::create_dir_all(&directory).expect("temporary directory");

  directory
}
