//! Synthetic archive projects the unpack tests are written against.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crc32fast::hash;
use xrf_archive::{ArchiveDescriptor, ArchiveFileDescriptor, ArchiveProject, ArchiveReadPolicy};
use xrf_error::XrfResult;
use xrf_job::ExecutionRequest;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::unpack::archive_extract_options::ArchiveExtractOptions;
use crate::unpack::archive_unpack_result::ArchiveUnpackResult;
use crate::unpack::archive_unpacker::ArchiveUnpacker;

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
  let mut files: HashMap<Arc<str>, ArchiveFileDescriptor> = HashMap::new();

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

    let descriptor: ArchiveFileDescriptor = ArchiveFileDescriptor::new(
      hash(entry.contents),
      entry.name.into(),
      offset,
      stored.len() as u32,
      entry.contents.len() as u32,
    );

    files.insert(entry.name.into(), descriptor);
  }

  fs::File::create(&source)
    .expect("test archive file")
    .write_all(&payload)
    .expect("test archive payload");

  ArchiveProject {
    // The volume the entries were laid out in. A project describes its volumes and its entries address them by
    // position, so a fixture that skipped this would be describing a set no reader could produce.
    archives: vec![ArchiveDescriptor {
      created_at: None,
      entries: files.len(),
      modified_at: None,
      output_root_path: PathBuf::new(),
      path: source.clone(),
      size_compressed: payload.len() as u64,
      size_real: payload.len() as u64,
    }],
    files,
    read_policy: ArchiveReadPolicy::default(),
    root: directory.into(),
    shadowed: Vec::new(),
    size_real: payload.len() as u64,
  }
}

pub(crate) fn create_temporary_directory(name: &str) -> PathBuf {
  let directory: PathBuf = build_absolute_generated_test_resource_path(&format!("unpack/{name}"));

  let _ = fs::remove_dir_all(&directory);

  fs::create_dir_all(&directory).expect("temporary directory");

  directory
}

/// Unpacks bounded to `workers`, the way a binary bounds itself.
///
/// The unpack owns no pool of its own, so a test that needs a known width installs one around the call rather than
/// passing a number in. That is not test scaffolding for its own sake — it is the same thing the CLI and the desktop
/// application do, so these tests exercise the arrangement the bound actually relies on instead of one that exists
/// only here.
pub(crate) fn unpack_with<P: AsRef<Path> + Send>(
  workers: NonZeroUsize,
  project: &ArchiveProject,
  destination: P,
) -> XrfResult<ArchiveUnpackResult> {
  ExecutionRequest::Workers(workers)
    .resolve()
    .install(|| ArchiveUnpacker::unpack_opt(project, destination, ArchiveExtractOptions::default()))
    .expect("the pool starts")
}
