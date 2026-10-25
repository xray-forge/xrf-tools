use std::collections::HashMap;
use std::fs::File;
use std::io::ErrorKind::UnexpectedEof;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use byteorder::ReadBytesExt;
use xrf_chunk::{CHUNK_ID_COMPRESSED_MASK, CHUNK_ID_MASK, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_lzhuf::decompress;
use xrf_utils::{
  XRayEncoding, assert, decode_bytes_to_string_without_bom_handling, format_path, new_declared_vec,
  new_windows1251_encoder,
};

use crate::volume::archive_constants::{
  CHUNK_ID_FILE_DESCRIPTORS_READ, CHUNK_ID_METADATA_READ, DESCRIPTOR_ROW_FIELDS_SIZE, MAXIMUM_ENTRY_NAME_SIZE,
};
use crate::volume::archive_header::ArchiveHeader;
use crate::volume::{ArchiveDescriptor, ArchiveFileDescriptor};

/// Reads one archive volume's header chunks.
pub(crate) struct ArchiveReader {
  path: PathBuf,
  file: File,
}

impl ArchiveReader {
  /// Opens a volume.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened.
  pub(crate) fn from_path(path: impl AsRef<Path>) -> XrfResult<Self> {
    let path: &Path = path.as_ref();

    match File::open(path) {
      Ok(file) => Ok(Self {
        file,
        path: path.into(),
      }),
      Err(error) => Err(XrfError::new_read_error(format!(
        "Failed to read archive file {}, {error}",
        format_path(path)
      ))),
    }
  }

  /// Encoding of a volume's header strings.
  fn header_encoding() -> XRayEncoding {
    new_windows1251_encoder()
  }

  /// Reads a volume's header chunks into a descriptor.
  ///
  /// # Errors
  ///
  /// Returns an error when the volume cannot be read, declares chunk or entry sizes its bytes cannot hold, or contains
  /// no file descriptors chunk. Malformed volumes are errors, never panics: a corrupt `.db` must become a skipped or
  /// reported mount rather than aborting the tool.
  /// Returns the volume alongside its entries, which the project merges and then owns alone.
  pub(crate) fn read_archive(&mut self) -> XrfResult<(ArchiveDescriptor, HashMap<Arc<str>, ArchiveFileDescriptor>)> {
    let header: ArchiveHeader = self.read_archive_header()?.ok_or_else(|| {
      XrfError::new_read_error(format!(
        "archive {} holds no file descriptors chunk",
        format_path(&self.path)
      ))
    })?;
    let metadata = self.file.metadata()?;
    let files: HashMap<Arc<str>, ArchiveFileDescriptor> = header.files;

    // Summed here because this is the last point the volume's own entries are known: after the merge a later volume
    // may shadow one of them, and a per-volume total counts what the volume holds rather than what survives.
    let size_compressed: u64 = files.values().map(|file| u64::from(file.size_compressed)).sum();
    let size_real: u64 = files.values().map(|file| u64::from(file.size_real)).sum();

    Ok((
      ArchiveDescriptor {
        created_at: Self::timestamp_millis(metadata.created().ok()),
        entries: files.len(),
        modified_at: Self::timestamp_millis(metadata.modified().ok()),
        output_root_path: header.output_root_path,
        path: header.archive_path,
        size_compressed,
        size_real,
      },
      files,
    ))
  }

  fn timestamp_millis(timestamp: Option<SystemTime>) -> Option<u64> {
    timestamp?
      .duration_since(UNIX_EPOCH)
      .ok()
      .and_then(|duration| u64::try_from(duration.as_millis()).ok())
  }

  fn read_archive_header(&mut self) -> XrfResult<Option<ArchiveHeader>> {
    let mut file_descriptors = None;
    let mut root_path: String = String::new();

    let volume_size: u64 = self.file.metadata()?.len();

    loop {
      let raw_chunk_id: u32 = match self.file.read_u32::<XRayByteOrder>() {
        Ok(data) => data,
        Err(error) if error.kind() == UnexpectedEof => break,
        Err(error) => return Err(XrfError::new_read_error(error.to_string())),
      };
      let chunk_size: u32 = self.file.read_u32::<XRayByteOrder>()?;
      let chunk_usize: usize = usize::try_from(chunk_size)
        .map_err(|error| XrfError::new_read_error(format!("Failed to read archive header chunk size: {error}")))?;

      // A chunk's payload lives in this file, so a declared size beyond it is corruption — checked before the size can
      // reach an allocation.
      let position: u64 = self.file.stream_position()?;

      if u64::from(chunk_size) > volume_size.saturating_sub(position) {
        return Err(XrfError::new_read_error(format!(
          "archive {} declares a {chunk_size}-byte chunk at {position}, beyond its {volume_size}-byte end",
          format_path(&self.path)
        )));
      }

      let chunk_id: u32 = raw_chunk_id & CHUNK_ID_MASK;
      let compressed: bool = (raw_chunk_id & CHUNK_ID_COMPRESSED_MASK) != 0;

      if CHUNK_ID_FILE_DESCRIPTORS_READ.contains(&chunk_id) {
        let chunk_data: Vec<u8> = Self::read_chunk(&mut self.file, chunk_usize, compressed)?;
        let mut reader: Cursor<&[u8]> = Cursor::new(chunk_data.as_slice());

        file_descriptors = Some(Self::read_file_descriptors(&mut reader)?);
      } else if CHUNK_ID_METADATA_READ.contains(&chunk_id) {
        let chunk_data: Vec<u8> = Self::read_chunk(&mut self.file, chunk_usize, compressed)?;

        root_path = Self::read_root_path(chunk_data.as_slice())?.ok_or_else(|| {
          XrfError::new_read_error(format!(
            "archive {} has a metadata chunk without an [header] entry_point",
            format_path(&self.path)
          ))
        })?;
      } else {
        // A volume may carry chunks this reader has no use for; skipping keeps an unknown one from ending the walk.
        self.file.seek(SeekFrom::Current(i64::from(chunk_size)))?;
      }
    }

    Ok(file_descriptors.map(|file_descriptors| ArchiveHeader {
      archive_path: self.path.clone(),
      output_root_path: root_path.into(),
      files: file_descriptors,
    }))
  }

  // todo: read the metadata as `CInifile::Load` does: strip `;` and `//` comments outside quotes, open a section only on
  // a line starting with `[` and name it up to its first `]` in any case, and trim and unquote the value.
  fn read_root_path(chunk_data: &[u8]) -> XrfResult<Option<String>> {
    let text: String = decode_bytes_to_string_without_bom_handling(chunk_data, Self::header_encoding())?;
    let mut section: &str = "";

    for line in text.lines() {
      match (Self::parse_section_name(line), section) {
        (None, "header") => {
          if let Some(("entry_point", value)) = Self::parse_variable(line) {
            return Ok(Some(Self::strip_root_alias(value).to_owned()));
          }
        }
        (Some(name), _) => {
          section = name;
        }
        _ => {}
      }
    }

    Ok(None)
  }

  /// The name of a line ending in `[name]`, whatever precedes the bracket.
  fn parse_section_name(line: &str) -> Option<&str> {
    let (_, name) = line.strip_suffix(']')?.rsplit_once('[')?;

    name.chars().all(Self::is_word_char).then_some(name)
  }

  /// The `name = value` of a line, the value with its leading whitespace trimmed and its trailing whitespace kept.
  fn parse_variable(line: &str) -> Option<(&str, &str)> {
    let (name, value) = line.split_once('=')?;
    let name: &str = name.trim();
    let value: &str = value.trim_start();

    (!name.is_empty() && name.chars().all(Self::is_word_char) && !value.is_empty()).then_some((name, value))
  }

  /// The entry point without its leading `$alias$\`, the root alias an unpack does not recreate.
  fn strip_root_alias(entry_point: &str) -> &str {
    entry_point
      .strip_prefix('$')
      .and_then(|rest| rest.split_once('$'))
      .filter(|(alias, _)| !alias.is_empty() && alias.chars().all(Self::is_word_char))
      .and_then(|(_, path)| path.strip_prefix('\\'))
      .unwrap_or(entry_point)
  }

  fn is_word_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
  }

  fn read_chunk<T: Read>(file: &mut T, chunk_usize: usize, compressed: bool) -> XrfResult<Vec<u8>> {
    let mut buffer: Vec<u8> = new_declared_vec(chunk_usize, "an archive header chunk")?;

    file.read_exact(buffer.as_mut_slice())?;

    if compressed { decompress(&buffer) } else { Ok(buffer) }
  }

  fn read_file_descriptors<T: Read>(reader: &mut T) -> XrfResult<HashMap<Arc<str>, ArchiveFileDescriptor>> {
    let mut file_descriptors: HashMap<Arc<str>, ArchiveFileDescriptor> = HashMap::new();
    let mut name_buf: [u8; MAXIMUM_ENTRY_NAME_SIZE] = [0u8; MAXIMUM_ENTRY_NAME_SIZE];

    loop {
      let header_size: u16 = match reader.read_u16::<XRayByteOrder>() {
        Ok(data) => data,
        Err(error) if error.kind() == UnexpectedEof => break,
        Err(error) => return Err(error.into()),
      };

      let size_real: u32 = reader.read_u32::<XRayByteOrder>()?;
      let size_compressed: u32 = reader.read_u32::<XRayByteOrder>()?;
      let crc: u32 = reader.read_u32::<XRayByteOrder>()?;

      // Checked: a corrupt header smaller than its own fixed prefix must be an error, not an underflow.
      let name_size: u16 = header_size.checked_sub(DESCRIPTOR_ROW_FIELDS_SIZE).ok_or_else(|| {
        XrfError::new_read_error(format!(
          "archive entry header declares {header_size} bytes, under its fixed {DESCRIPTOR_ROW_FIELDS_SIZE}-byte prefix"
        ))
      })?;

      let name_bytes = {
        assert((name_size as usize) < name_buf.len(), "Name is too long")?;

        reader.read_exact(&mut name_buf[..(name_size as usize)])?;

        &name_buf[..(name_size as usize)]
      };

      let offset: u32 = reader.read_u32::<XRayByteOrder>()?;
      let name: Arc<str> = Arc::from(decode_bytes_to_string_without_bom_handling(
        name_bytes,
        Self::header_encoding(),
      )?);

      file_descriptors.insert(
        Arc::clone(&name),
        ArchiveFileDescriptor::new(crc, name, offset, size_compressed, size_real),
      );
    }

    Ok(file_descriptors)
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use byteorder::{LittleEndian, WriteBytesExt};
  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use super::ArchiveReader;

  /// Writes raw bytes as a volume file, since malformed input is a byte-level condition.
  fn volume(name: &str, bytes: &[u8]) -> PathBuf {
    let root: PathBuf = build_absolute_generated_test_resource_path("archive_reader");

    fs::create_dir_all(&root).expect("scratch root");

    let path: PathBuf = root.join(name);

    fs::write(&path, bytes).expect("volume written");

    path
  }

  /// A corrupt volume must come back as an error the mount planner can skip and report — with `panic = "abort"` in
  /// release builds, a panic here would take the whole tool down over one bad file.

  #[test]
  fn a_volume_with_no_descriptor_chunk_is_an_error_not_a_panic() {
    let path: PathBuf = volume("empty.db0", b"");

    assert!(
      ArchiveReader::from_path(&path)
        .expect("reader opens")
        .read_archive()
        .is_err()
    );
  }

  #[test]
  fn a_chunk_size_beyond_the_volume_is_an_error_not_an_allocation() {
    // Chunk id 1, declared size u32::MAX: the size must be rejected against the file length before any allocation.
    let mut bytes: Vec<u8> = Vec::new();

    bytes.write_u32::<LittleEndian>(1).expect("chunk id");
    bytes.write_u32::<LittleEndian>(u32::MAX).expect("chunk size");

    let path: PathBuf = volume("absurd_size.db0", &bytes);
    let error = ArchiveReader::from_path(&path)
      .expect("reader opens")
      .read_archive()
      .expect_err("declared size exceeds the volume");

    assert!(error.to_string().contains("beyond its"), "got: {error}");
  }

  #[test]
  fn an_entry_header_smaller_than_its_prefix_is_an_error_not_an_underflow() {
    // Chunk id 1 holding one descriptor whose header_size (8) is smaller than the fixed 16-byte prefix.
    let mut descriptors: Vec<u8> = Vec::new();

    descriptors.write_u16::<LittleEndian>(8).expect("header size");
    descriptors.write_u32::<LittleEndian>(0).expect("size real");
    descriptors.write_u32::<LittleEndian>(0).expect("size compressed");
    descriptors.write_u32::<LittleEndian>(0).expect("crc");

    let mut bytes: Vec<u8> = Vec::new();

    bytes.write_u32::<LittleEndian>(1).expect("chunk id");
    bytes
      .write_u32::<LittleEndian>(descriptors.len() as u32)
      .expect("chunk size");
    bytes.extend_from_slice(&descriptors);

    let path: PathBuf = volume("short_header.db0", &bytes);
    let error = ArchiveReader::from_path(&path)
      .expect("reader opens")
      .read_archive()
      .expect_err("header smaller than its prefix");

    assert!(error.to_string().contains("16-byte prefix"), "got: {error}");
  }

  /// Reads the root path of a metadata chunk holding `text`.
  fn root_path_of(text: &str) -> Option<String> {
    ArchiveReader::read_root_path(text.as_bytes()).expect("metadata decodes")
  }

  #[test]
  fn the_entry_point_loses_its_root_alias() {
    assert_eq!(
      root_path_of("[header]\nauto_load = true\nentry_point = $game_data$\\levels\\\n").as_deref(),
      Some("levels\\")
    );
    assert_eq!(
      root_path_of("[header]\n  entry_point=gamedata").as_deref(),
      Some("gamedata")
    );
    assert_eq!(
      root_path_of("[header]\r\nentry_point = $fs_root$\\\r\n").as_deref(),
      Some("")
    );
  }

  #[test]
  fn an_entry_point_outside_the_header_section_is_ignored() {
    assert_eq!(root_path_of("entry_point = gamedata\n[header]\nauto_load = true"), None);
    assert_eq!(root_path_of("[header]\n[other]\nentry_point = gamedata"), None);
  }

  /// Unlike `CInifile`, which drops the comment and so never opens the section.
  #[test]
  fn a_comment_ending_in_a_section_name_opens_that_section() {
    assert_eq!(
      root_path_of("; comment [header]\nentry_point = gamedata").as_deref(),
      Some("gamedata")
    );
  }

  #[test]
  fn header_lines_split_into_sections_variables_and_aliases() {
    assert_eq!(ArchiveReader::parse_section_name("[header]"), Some("header"));
    assert_eq!(ArchiveReader::parse_section_name("[]"), Some(""));
    assert_eq!(ArchiveReader::parse_section_name("[head er]"), None);
    assert_eq!(ArchiveReader::parse_section_name("[header] "), None);

    assert_eq!(
      ArchiveReader::parse_variable(" name = a = b "),
      Some(("name", "a = b "))
    );
    assert_eq!(ArchiveReader::parse_variable("name ="), None);
    assert_eq!(ArchiveReader::parse_variable("two words = value"), None);

    assert_eq!(ArchiveReader::strip_root_alias("$game_data$\\meshes"), "meshes");
    assert_eq!(ArchiveReader::strip_root_alias("$$\\meshes"), "$$\\meshes");
    assert_eq!(
      ArchiveReader::strip_root_alias("$game data$\\meshes"),
      "$game data$\\meshes"
    );
    assert_eq!(
      ArchiveReader::strip_root_alias("$game_data$meshes"),
      "$game_data$meshes"
    );
  }

  #[test]
  fn a_truncated_descriptor_chunk_is_an_error_not_a_panic() {
    // A descriptor declaring a 20-character name the chunk does not contain.
    let mut descriptors: Vec<u8> = Vec::new();

    descriptors.write_u16::<LittleEndian>(36).expect("header size");
    descriptors.write_u32::<LittleEndian>(4).expect("size real");
    descriptors.write_u32::<LittleEndian>(4).expect("size compressed");
    descriptors.write_u32::<LittleEndian>(0).expect("crc");
    descriptors.extend_from_slice(b"tru"); // 3 bytes where 20 were declared.

    let mut bytes: Vec<u8> = Vec::new();

    bytes.write_u32::<LittleEndian>(1).expect("chunk id");
    bytes
      .write_u32::<LittleEndian>(descriptors.len() as u32)
      .expect("chunk size");
    bytes.extend_from_slice(&descriptors);

    let path: PathBuf = volume("truncated.db0", &bytes);

    assert!(
      ArchiveReader::from_path(&path)
        .expect("reader opens")
        .read_archive()
        .is_err()
    );
  }
}
