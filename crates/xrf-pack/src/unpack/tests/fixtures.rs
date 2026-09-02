//! Synthetic archive projects the unpack tests are written against.

use std::collections::HashMap;
use std::fs;
use std::io::{Result as IoResult, Write};
use std::path::{Path, PathBuf};

use crc32fast::hash;
use xrf_archive::{ArchiveDescriptor, ArchiveFileDescriptor, ArchiveProject, ArchiveProjectReadPolicy};

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

/// Points `link` at an existing directory.
///
/// A Windows symlink needs developer mode or the create-symlink privilege, which an ordinary desktop or CI account
/// does not hold; a junction needs neither and redirects the same traversal, so it is what an unprivileged Windows run
/// actually tests against.
pub(crate) fn link_directory(target: &Path, link: &Path) -> bool {
  #[cfg(unix)]
  let result: IoResult<()> = std::os::unix::fs::symlink(target, link);
  #[cfg(windows)]
  let result: IoResult<()> = std::os::windows::fs::symlink_dir(target, link).or_else(|_| link_junction(target, link));

  report_link(result, link)
}

/// Points `link` at an existing file.
///
/// No junction stands in for this one, so a Windows account without the privilege skips rather than passing a check it
/// never made.
pub(crate) fn link_file(target: &Path, link: &Path) -> bool {
  #[cfg(unix)]
  let result: IoResult<()> = std::os::unix::fs::symlink(target, link);
  #[cfg(windows)]
  let result: IoResult<()> = std::os::windows::fs::symlink_file(target, link);

  report_link(result, link)
}

/// One raw argument, because `cmd` re-parses the line and both paths may hold spaces.
#[cfg(windows)]
fn link_junction(target: &Path, link: &Path) -> IoResult<()> {
  use std::io::Error as IoError;
  use std::os::windows::process::CommandExt;
  use std::process::{Command, Stdio};

  let status = Command::new("cmd")
    .raw_arg(format!("/C mklink /J \"{}\" \"{}\"", link.display(), target.display()))
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()?;

  if status.success() {
    Ok(())
  } else {
    Err(IoError::other(format!("mklink /J exited with {status}")))
  }
}

fn report_link(result: IoResult<()>, link: &Path) -> bool {
  match result {
    Ok(()) => true,
    Err(error) => {
      eprintln!(
        "skipping: this host cannot create the link '{}': {error}",
        link.display()
      );

      false
    }
  }
}
