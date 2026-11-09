use std::collections::HashMap;
use std::fs::{File, Metadata};
use std::io::ErrorKind::UnexpectedEof;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use byteorder::ReadBytesExt;
use xrf_chunk::{CHUNK_ID_COMPRESSED_MASK, CHUNK_ID_MASK, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_lzhuf::LzhufDecoder;
use xrf_utils::{
  XRayEncoding, decode_bytes_to_string_without_bom_handling, format_path, new_declared_vec, new_windows1251_encoder,
};

use crate::volume::archive_constants::{
  CHUNK_ID_FILE_DESCRIPTORS_READ, CHUNK_ID_METADATA_READ, DESCRIPTOR_ROW_FIELDS_SIZE, MAXIMUM_ENTRY_NAME_SIZE,
};
use crate::volume::archive_header::ArchiveHeader;
use crate::volume::archive_header_cipher::ArchiveHeaderCipher;
use crate::volume::archive_metadata::ArchiveMetadata;
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

  /// Reads a volume's header chunks into a descriptor, returned alongside its entries, which the project merges and
  /// then owns alone.
  ///
  /// # Errors
  ///
  /// Returns an error when the volume cannot be read, declares chunk or entry sizes its bytes cannot hold, or contains
  /// no file descriptors chunk. Malformed volumes are errors, never panics: a corrupt `.db` must become a skipped or
  /// reported mount rather than aborting the tool.
  pub(crate) fn read_archive(&mut self) -> XrfResult<(ArchiveDescriptor, HashMap<Arc<str>, ArchiveFileDescriptor>)> {
    let metadata: Metadata = self.file.metadata()?;
    let header: ArchiveHeader = self.read_header(metadata.len())?;
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
        mount_root: header.mount_root,
        path: self.path.clone(),
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

  /// Walks the volume's top-level chunks, keeping the name table and the metadata and skipping the rest.
  fn read_header(&mut self, volume_size: u64) -> XrfResult<ArchiveHeader> {
    // The name table is kept as stored until the walk ends: whether it is enciphered depends on whether any chunk
    // carried metadata, wherever that chunk stands.
    let mut descriptors: Option<(Vec<u8>, bool)> = None;
    let mut root_path: Option<String> = None;

    loop {
      let raw_chunk_id: u32 = match self.file.read_u32::<XRayByteOrder>() {
        Ok(data) => data,
        Err(error) if error.kind() == UnexpectedEof => break,
        Err(error) => return Err(error.into()),
      };
      let chunk_size: u32 = self.file.read_u32::<XRayByteOrder>()?;

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
      let is_compressed: bool = (raw_chunk_id & CHUNK_ID_COMPRESSED_MASK) != 0;

      if CHUNK_ID_FILE_DESCRIPTORS_READ.contains(&chunk_id) {
        descriptors = Some((self.read_stored_chunk(chunk_size)?, is_compressed));
      } else if CHUNK_ID_METADATA_READ.contains(&chunk_id) {
        let chunk: Vec<u8> = self.read_chunk(chunk_size, is_compressed)?;
        let metadata: String = decode_bytes_to_string_without_bom_handling(&chunk, Self::header_encoding())?;

        root_path = Some(ArchiveMetadata::find_root_path(&metadata).ok_or_else(|| {
          XrfError::new_read_error(format!(
            "archive {} has a metadata chunk without an [header] entry_point",
            format_path(&self.path)
          ))
        })?);
      } else {
        // A volume may carry chunks this reader has no use for; skipping keeps an unknown one from ending the walk.
        self.file.seek(SeekFrom::Current(i64::from(chunk_size)))?;
      }
    }

    let (stored, is_compressed): (Vec<u8>, bool) = descriptors.ok_or_else(|| {
      XrfError::new_read_error(format!(
        "archive {} holds no file descriptors chunk",
        format_path(&self.path)
      ))
    })?;
    let files: HashMap<Arc<str>, ArchiveFileDescriptor> =
      self.read_name_table(&stored, is_compressed, root_path.is_some())?;

    Ok(ArchiveHeader {
      mount_root: ArchiveMetadata::resolve_mount_root(root_path.as_deref()),
      output_root_path: root_path.map(PathBuf::from).unwrap_or_default(),
      files,
    })
  }

  /// Reads the payload of the chunk the file is positioned at, decompressing it when its id says so.
  fn read_chunk(&mut self, size: u32, is_compressed: bool) -> XrfResult<Vec<u8>> {
    let buffer: Vec<u8> = self.read_stored_chunk(size)?;

    if is_compressed {
      LzhufDecoder::decompress(&buffer)
    } else {
      Ok(buffer)
    }
  }

  /// Reads the payload of the chunk the file is positioned at, as stored.
  fn read_stored_chunk(&mut self, size: u32) -> XrfResult<Vec<u8>> {
    let mut buffer: Vec<u8> = new_declared_vec(size as usize, "an archive header chunk")?;

    self.file.read_exact(buffer.as_mut_slice())?;

    Ok(buffer)
  }

  /// The entries a stored name table lists. A compressed one in a volume with no metadata, other than an `.xdb`, is
  /// Shadow of Chornobyl's and enciphered, as `CLocatorAPI::LoadArchive` assumes: deciphered with the worldwide key,
  /// else the Russian one, the key that yields a table that decompresses and reads whole winning.
  fn read_name_table(
    &self,
    stored: &[u8],
    is_compressed: bool,
    has_metadata: bool,
  ) -> XrfResult<HashMap<Arc<str>, ArchiveFileDescriptor>> {
    let read = |bytes: &[u8]| -> XrfResult<HashMap<Arc<str>, ArchiveFileDescriptor>> {
      let table: Vec<u8> = if is_compressed {
        LzhufDecoder::decompress(bytes)?
      } else {
        bytes.to_vec()
      };

      Self::read_file_descriptors(&mut Cursor::new(table.as_slice()))
    };
    let is_xdb: bool = self
      .path
      .extension()
      .is_some_and(|extension| extension.to_string_lossy().to_ascii_lowercase().starts_with("xdb"));

    if !is_compressed || has_metadata || is_xdb {
      return read(stored);
    }

    let mut first_error: Option<XrfError> = None;

    for cipher in ArchiveHeaderCipher::ALL {
      match read(&cipher.decode(stored)) {
        Ok(files) => return Ok(files),
        Err(error) => {
          first_error.get_or_insert(error);
        }
      }
    }

    // A volume no key opens may still be a plain one some packer wrote without metadata.
    read(stored).map_err(|error| first_error.unwrap_or(error))
  }

  fn read_file_descriptors<T: Read>(reader: &mut T) -> XrfResult<HashMap<Arc<str>, ArchiveFileDescriptor>> {
    let mut file_descriptors: HashMap<Arc<str>, ArchiveFileDescriptor> = HashMap::new();
    let mut name_buffer: [u8; MAXIMUM_ENTRY_NAME_SIZE] = [0u8; MAXIMUM_ENTRY_NAME_SIZE];

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

      // Strictly under the buffer: the engine reads a name into it and terminates it, so a full one has no room.
      if usize::from(name_size) >= MAXIMUM_ENTRY_NAME_SIZE {
        return Err(XrfError::new_read_error(format!(
          "archive entry name declares {name_size} bytes, more than the engine's {MAXIMUM_ENTRY_NAME_SIZE}-byte buffer \
           holds with its terminator"
        )));
      }

      let name_bytes: &mut [u8] = &mut name_buffer[..usize::from(name_size)];

      reader.read_exact(name_bytes)?;

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
  use xrf_chunk::CHUNK_ID_COMPRESSED_MASK;
  use xrf_lzhuf::LzhufEncoder;
  use xrf_test_utils::GeneratedResources;

  use super::ArchiveReader;
  use crate::volume::archive_header_cipher::ArchiveHeaderCipher;

  /// Writes raw bytes as a volume file, since malformed input is a byte-level condition.
  fn volume(name: &str, bytes: &[u8]) -> PathBuf {
    let root: PathBuf = GeneratedResources::path("archive_reader");

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

  #[test]
  fn a_metadata_chunk_naming_no_entry_point_is_an_error() {
    let metadata: &[u8] = b"[header]\r\nauto_load = true\r\n";
    let mut bytes: Vec<u8> = Vec::new();

    bytes.write_u32::<LittleEndian>(666).expect("chunk id");
    bytes
      .write_u32::<LittleEndian>(metadata.len() as u32)
      .expect("chunk size");
    bytes.extend_from_slice(metadata);
    bytes.write_u32::<LittleEndian>(1).expect("chunk id");
    bytes.write_u32::<LittleEndian>(0).expect("chunk size");

    let path: PathBuf = volume("no_entry_point.db0", &bytes);
    let error = ArchiveReader::from_path(&path)
      .expect("reader opens")
      .read_archive()
      .expect_err("metadata without an entry point");

    assert!(error.to_string().contains("entry_point"), "got: {error}");
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

  #[test]
  fn reads_a_name_table_shadow_of_chornobyl_enciphered() {
    // One entry, compressed and enciphered, in a volume with no metadata: a Russian release's volume.
    let name: &[u8] = b"meshes\\crate.ogf";
    let mut descriptors: Vec<u8> = Vec::new();

    descriptors
      .write_u16::<LittleEndian>(16 + name.len() as u16)
      .expect("header size");
    descriptors.write_u32::<LittleEndian>(4).expect("size real");
    descriptors.write_u32::<LittleEndian>(4).expect("size compressed");
    descriptors.write_u32::<LittleEndian>(7).expect("crc");
    descriptors.extend_from_slice(name);
    descriptors.write_u32::<LittleEndian>(0).expect("offset");

    let stored: Vec<u8> =
      ArchiveHeaderCipher::Russian.encode(&LzhufEncoder::compress(&descriptors).expect("table compresses"));
    let mut bytes: Vec<u8> = Vec::new();

    bytes
      .write_u32::<LittleEndian>(CHUNK_ID_COMPRESSED_MASK | 1)
      .expect("chunk id");
    bytes
      .write_u32::<LittleEndian>(stored.len() as u32)
      .expect("chunk size");
    bytes.extend_from_slice(&stored);

    let path: PathBuf = volume("enciphered.db0", &bytes);
    let (descriptor, files) = ArchiveReader::from_path(&path)
      .expect("reader opens")
      .read_archive()
      .expect("the name table deciphers");

    assert_eq!(descriptor.entries, 1);
    assert!(files.contains_key("meshes\\crate.ogf"), "{:?}", files.keys());
  }

  #[test]
  fn a_name_the_engine_buffer_cannot_hold_is_an_error() {
    // A 520-byte name fills the engine's buffer and leaves no room for its terminator.
    let mut descriptors: Vec<u8> = Vec::new();

    descriptors.write_u16::<LittleEndian>(16 + 520).expect("header size");
    descriptors.write_u32::<LittleEndian>(0).expect("size real");
    descriptors.write_u32::<LittleEndian>(0).expect("size compressed");
    descriptors.write_u32::<LittleEndian>(0).expect("crc");
    descriptors.extend_from_slice(&[b'a'; 520]);
    descriptors.write_u32::<LittleEndian>(0).expect("offset");

    let mut bytes: Vec<u8> = Vec::new();

    bytes.write_u32::<LittleEndian>(1).expect("chunk id");
    bytes
      .write_u32::<LittleEndian>(descriptors.len() as u32)
      .expect("chunk size");
    bytes.extend_from_slice(&descriptors);

    let path: PathBuf = volume("long_name.db0", &bytes);
    let error = ArchiveReader::from_path(&path)
      .expect("reader opens")
      .read_archive()
      .expect_err("name over the engine's buffer");

    assert!(error.to_string().contains("520-byte buffer"), "got: {error}");
  }
}
