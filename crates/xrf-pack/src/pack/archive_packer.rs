use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Instant;

use lzokay::compress::Dict;
use xrf_archive::CHUNK_ID_COMPRESSED_MASK;
use xrf_error::{XrfError, XrfResult};
use xrf_lzhuf::compress;
use xrf_utils::encode_string_to_w1251_bytes;

use crate::pack::archive_pack_config::{ArchivePackConfig, ArchivePackMode};
use crate::pack::archive_pack_result::ArchivePackResult;
use crate::pack::archive_pack_source::{ArchivePackEntry, ArchivePackSource};

/// Chunk carrying the `[header]` ini text, which is also what marks an archive as not being ShoC.
const CHUNK_ID_HEADER: u32 = 666;

/// Chunk carrying file payloads back to back.
const CHUNK_ID_DATA: u32 = 0;

/// Chunk carrying the descriptor table, always written compressed like the engine writes it.
const CHUNK_ID_DESCRIPTORS: u32 = 1;

/// Bytes a chunk spends on its id and size before any payload.
const CHUNK_HEADER_SIZE: u64 = 8;

/// Fixed part of a descriptor row: the four numeric fields that surround the name.
const DESCRIPTOR_ROW_OVERHEAD: usize = 16;

/// Extensions the engine expects to find compressed; everything else is stored.
///
/// `testVFS` in `xrCompress.cpp` inverts the intuitive rule: only text the engine parses is worth the
/// LZO round trip, because meshes, textures, and sounds are already compressed in their own formats.
const COMPRESSED_EXTENSIONS: [&str; 3] = ["xml", "ltx", "script"];

/// Compression must save more than this to be worth keeping, matching xrCompress.
const COMPRESSION_MARGIN: usize = 16;

/// A payload already written to the current volume, kept so an identical file can point at it.
struct ArchiveAlias {
  path: PathBuf,
  offset: u32,
  size_compressed: u32,
}

/// Writes one volume set from a source tree.
pub struct ArchivePacker;

impl ArchivePacker {
  /// Pack the configured source into `<name>.db<N>` volumes and report what was written.
  ///
  /// Produces the exact layout the engine mounts: an optional header chunk, one data chunk holding every
  /// payload, and a compressed descriptor table whose offsets are absolute positions in the volume.
  pub fn pack(config: &ArchivePackConfig) -> XrfResult<ArchivePackResult> {
    let started_at: Instant = Instant::now();
    let source: ArchivePackSource = ArchivePackSource::collect(config)?;

    // xrCompress refuses an empty file list too. Saying so here beats leaving the caller to puzzle out
    // a complaint from the codec about an empty descriptor table.
    if source.entries.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "Nothing to pack from '{}': {} file(s) matched, {} skipped by the configured rules",
        config.source.display(),
        source.entries.len(),
        source.skipped
      )));
    }

    fs::create_dir_all(&config.destination)?;

    let mut state: PackState = PackState::new(config, source.directories.clone())?;

    state.result.files_skipped = source.skipped;
    state.result.files_total = source.entries.len();

    // The coder's working state is large, so it is built once on the heap and reused for every entry
    // rather than rebuilt per file.
    let mut dict: Box<Dict> = Dict::new();

    for entry in &source.entries {
      // The engine opens a volume per file, so a volume is closed before the file that would overrun it
      // rather than after, exactly as xrCompress decides.
      if state.position > config.max_volume_size {
        state.close_volume()?;
        state.open_volume(config)?;
      }

      state.write_entry(config, &mut dict, entry)?;
    }

    state.close_volume()?;

    let mut result: ArchivePackResult = state.result;

    // Only now is the volume count known, so a set that stayed single drops its index.
    if let [only] = result.volumes.as_slice() {
      let renamed: PathBuf = config.destination.join(config.single_volume_name());

      fs::rename(only, &renamed)?;

      result.volumes = vec![renamed];
    }

    result.duration = started_at.elapsed();

    Ok(result)
  }
}

/// The volume being written and everything accumulated for its descriptor table.
struct PackState {
  writer: Option<BufWriter<File>>,
  path: PathBuf,
  /// Absolute position in the current volume, which is what descriptor offsets record.
  position: u64,
  /// Where the data chunk's size field sits, so it can be patched once the payloads are known.
  data_size_position: u64,
  descriptors: Vec<u8>,
  /// Payloads already written to this volume, keyed by size and checksum.
  aliases: HashMap<(u32, u32), Vec<ArchiveAlias>>,
  /// Directory rows, written into every volume so any one of them can list the tree.
  ///
  /// Encoded once: they are identical in every volume, and they carry no payload, so nothing about them varies with
  /// what a volume holds.
  directory_rows: Vec<u8>,
  volume_index: usize,
  result: ArchivePackResult,
}

impl PackState {
  fn new(config: &ArchivePackConfig, directories: Vec<String>) -> XrfResult<Self> {
    let mut directory_rows: Vec<u8> = Vec::new();

    for directory in &directories {
      // `CLocatorAPI` recognizes archive directories by this delimiter, not by their zero payload.
      let name: String = format!("{directory}\\");

      Self::push_descriptor_row(&mut directory_rows, &name, 0, 0, 0, 0)?;
    }

    let mut state: Self = Self {
      writer: None,
      path: PathBuf::new(),
      position: 0,
      data_size_position: 0,
      descriptors: Vec::new(),
      aliases: HashMap::new(),
      directory_rows,
      volume_index: 0,
      result: ArchivePackResult::default(),
    };

    state.open_volume(config)?;

    Ok(state)
  }

  fn open_volume(&mut self, config: &ArchivePackConfig) -> XrfResult<()> {
    self.path = config.destination.join(config.volume_name(self.volume_index));
    self.position = 0;
    self.descriptors.clear();
    self.aliases.clear();

    let mut writer: BufWriter<File> = BufWriter::new(File::create(&self.path)?);

    if let Some(header) = &config.header {
      let payload: Vec<u8> = encode_string_to_w1251_bytes(header).map_err(|error| {
        XrfError::new_encoding_error(format!("Failed to encode archive header as windows-1251: {error}"))
      })?;

      writer.write_all(&CHUNK_ID_HEADER.to_le_bytes())?;
      writer.write_all(&(payload.len() as u32).to_le_bytes())?;
      writer.write_all(&payload)?;

      self.position += CHUNK_HEADER_SIZE + payload.len() as u64;
    }

    // The data chunk's size is only known once every payload is in, so leave room and patch it later.
    writer.write_all(&CHUNK_ID_DATA.to_le_bytes())?;
    self.data_size_position = self.position + 4;
    writer.write_all(&0u32.to_le_bytes())?;

    self.position += CHUNK_HEADER_SIZE;
    self.writer = Some(writer);

    // Directory rows carry no payload, so they can be written up front.
    self.descriptors.extend_from_slice(&self.directory_rows);

    Ok(())
  }

  fn close_volume(&mut self) -> XrfResult<()> {
    let Some(mut writer) = self.writer.take() else {
      return Ok(());
    };

    let data_size: u64 = self.position - self.data_size_position - 4;

    let descriptors: Vec<u8> = compress(&self.descriptors)?;

    writer.write_all(&(CHUNK_ID_DESCRIPTORS | CHUNK_ID_COMPRESSED_MASK).to_le_bytes())?;
    writer.write_all(&(descriptors.len() as u32).to_le_bytes())?;
    writer.write_all(&descriptors)?;

    self.position += CHUNK_HEADER_SIZE + descriptors.len() as u64;

    // Patch the data chunk's size now that the payloads are behind it.
    writer.seek(SeekFrom::Start(self.data_size_position))?;
    writer.write_all(&(data_size as u32).to_le_bytes())?;
    writer.flush()?;

    self.result.volumes.push(self.path.clone());
    self.result.size_written += self.position;
    self.volume_index += 1;

    Ok(())
  }

  fn write_entry(&mut self, config: &ArchivePackConfig, dict: &mut Dict, entry: &ArchivePackEntry) -> XrfResult<()> {
    let contents: Vec<u8> = fs::read(&entry.path)?;
    let size_real: u32 = u32::try_from(contents.len()).map_err(|_| {
      XrfError::new_invalid_error(format!(
        "File '{}' is {} bytes, larger than an archive entry can describe",
        entry.name,
        contents.len()
      ))
    })?;

    let crc: u32 = crc32fast::hash(&contents);

    self.result.size_source += u64::from(size_real);

    if let Some(alias) = self.find_alias(&contents, size_real, crc)? {
      let (offset, size_compressed) = alias;

      self.result.files_aliased += 1;

      return self.push_descriptor(&entry.name, size_real, size_compressed, crc, offset);
    }

    let offset: u32 = u32::try_from(self.position).map_err(|_| {
      XrfError::new_invalid_error(format!(
        "Volume grew past {} bytes, which an entry offset cannot address",
        u32::MAX
      ))
    })?;

    let payload: Vec<u8> = self.compress_payload(config, dict, entry, &contents)?;
    let size_compressed: u32 = payload.len() as u32;

    if let Some(writer) = self.writer.as_mut() {
      writer.write_all(&payload)?;
    }

    self.position += payload.len() as u64;

    self.aliases.entry((size_real, crc)).or_default().push(ArchiveAlias {
      path: entry.path.clone(),
      offset,
      size_compressed,
    });

    self.push_descriptor(&entry.name, size_real, size_compressed, crc, offset)
  }

  /// Compress a payload when the engine expects it compressed and the result is actually smaller.
  fn compress_payload(
    &mut self,
    config: &ArchivePackConfig,
    dict: &mut Dict,
    entry: &ArchivePackEntry,
    contents: &[u8],
  ) -> XrfResult<Vec<u8>> {
    let is_compressible: bool = config.mode == ArchivePackMode::Compress
      && !contents.is_empty()
      && entry
        .name
        .rsplit_once('.')
        .is_some_and(|(_, extension)| COMPRESSED_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()));

    if !is_compressible {
      self.result.files_stored += 1;

      return Ok(contents.to_vec());
    }

    let compressed: Vec<u8> = lzokay::compress::compress_with_dict(contents, dict)
      .map_err(|error| XrfError::new_unexpected_error(format!("Failed to compress '{}': {error}", entry.name)))?;

    // A payload that barely shrinks costs more to decompress than it saves, so it reverts to stored.
    // The reader tells the two apart by the sizes alone, so this must stay a real size difference.
    if compressed.len() + COMPRESSION_MARGIN >= contents.len() {
      self.result.files_stored += 1;

      return Ok(contents.to_vec());
    }

    self.result.files_compressed += 1;

    Ok(compressed)
  }

  /// Find an identical payload already in this volume, confirming the match byte for byte.
  fn find_alias(&self, contents: &[u8], size_real: u32, crc: u32) -> XrfResult<Option<(u32, u32)>> {
    let Some(candidates) = self.aliases.get(&(size_real, crc)) else {
      return Ok(None);
    };

    for candidate in candidates {
      // Equal size and checksum is strong evidence, not proof; xrCompress compares the bytes too.
      if fs::read(&candidate.path)? == contents {
        return Ok(Some((candidate.offset, candidate.size_compressed)));
      }
    }

    Ok(None)
  }

  /// Append one descriptor row for this volume's table.
  fn push_descriptor(
    &mut self,
    name: &str,
    size_real: u32,
    size_compressed: u32,
    crc: u32,
    offset: u32,
  ) -> XrfResult<()> {
    Self::push_descriptor_row(&mut self.descriptors, name, size_real, size_compressed, crc, offset)
  }

  /// Append one descriptor row to a table: sizes, checksum, windows-1251 name, and the payload's absolute offset.
  ///
  /// Takes the table rather than `self` so the directory rows, which are the same in every volume, can be encoded once
  /// before any volume is open.
  fn push_descriptor_row(
    descriptors: &mut Vec<u8>,
    name: &str,
    size_real: u32,
    size_compressed: u32,
    crc: u32,
    offset: u32,
  ) -> XrfResult<()> {
    let encoded: Vec<u8> = encode_string_to_w1251_bytes(name).map_err(|error| {
      XrfError::new_encoding_error(format!(
        "File name '{name}' cannot be written as windows-1251, which the engine requires: {error}"
      ))
    })?;

    let row_size: u16 = u16::try_from(encoded.len() + DESCRIPTOR_ROW_OVERHEAD)
      .map_err(|_| XrfError::new_invalid_error(format!("File name '{name}' is too long for a descriptor row")))?;

    descriptors.extend_from_slice(&row_size.to_le_bytes());
    descriptors.extend_from_slice(&size_real.to_le_bytes());
    descriptors.extend_from_slice(&size_compressed.to_le_bytes());
    descriptors.extend_from_slice(&crc.to_le_bytes());
    descriptors.extend_from_slice(&encoded);
    descriptors.extend_from_slice(&offset.to_le_bytes());

    Ok(())
  }
}
