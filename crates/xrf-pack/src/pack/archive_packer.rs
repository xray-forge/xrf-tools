use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Instant;

use lzokay::compress::Dict;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, to_format_size};

use crate::pack::archive_descriptor_table::{ArchiveDescriptorTable, DescriptorName};
use crate::pack::archive_pack_config::{ArchivePackConfig, ArchivePackMode};
use crate::pack::archive_pack_result::ArchivePackResult;
use crate::pack::archive_pack_source::{ArchivePackEntry, ArchivePackSource};
use crate::pack::archive_volume_layout::ArchiveVolumeLayout;

/// Chunk carrying the `[header]` ini text, which is also what marks an archive as not being ShoC.
const CHUNK_ID_HEADER: u32 = 666;

/// Chunk carrying file payloads back to back.
const CHUNK_ID_DATA: u32 = 0;

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
  /// payload, and a descriptor table whose offsets are absolute positions in the volume.
  ///
  /// `max_volume_size` is a hard maximum on each finished file. An entry is placed only once its
  /// stored-or-compressed payload and its descriptor row are known, and a cap that cannot hold some entry in a
  /// volume of its own is refused rather than exceeded; see [`ArchiveVolumeLayout`].
  pub fn pack(config: &ArchivePackConfig) -> XrfResult<ArchivePackResult> {
    config.validate_for_packing()?;

    let started_at: Instant = Instant::now();
    let source: ArchivePackSource = ArchivePackSource::collect(config)?;

    // xrCompress refuses an empty file list too. Saying so here beats leaving the caller to puzzle out
    // a complaint from the codec about an empty descriptor table.
    if source.entries.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "Nothing to pack from '{}': {} file(s) matched, {} skipped by the configured rules",
        format_path(&config.source),
        source.entries.len(),
        source.skipped
      )));
    }

    // Both are measured before anything is created, because an unsatisfiable cap is a property of the archive rather
    // than of the file that would first overflow, and refusing it must leave no destination behind.
    let descriptors: ArchiveDescriptorTable = ArchiveDescriptorTable::of_directories(&source.directories)?;
    let layout: ArchiveVolumeLayout = ArchiveVolumeLayout::new(config, &descriptors)?;

    fs::create_dir_all(&config.destination)?;

    let mut state: PackState = PackState::new(config, layout, descriptors)?;

    state.result.files_skipped = source.skipped;
    state.result.files_total = source.entries.len();

    // The coder's working state is large, so it is built once on the heap and reused for every entry
    // rather than rebuilt per file.
    let mut dict: Box<Dict> = Dict::new();

    for entry in &source.entries {
      state.add_entry(config, &mut dict, entry)?;
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

/// The volume being written and everything accumulated for it.
struct PackState {
  writer: Option<BufWriter<File>>,
  path: PathBuf,
  /// Absolute position in the current volume, which is what descriptor offsets record.
  position: u64,
  descriptors: ArchiveDescriptorTable,
  /// Payloads already written to this volume, keyed by size and checksum.
  aliases: HashMap<(u32, u32), Vec<ArchiveAlias>>,
  layout: ArchiveVolumeLayout,
  volume_index: usize,
  result: ArchivePackResult,
}

impl PackState {
  fn new(
    config: &ArchivePackConfig,
    layout: ArchiveVolumeLayout,
    descriptors: ArchiveDescriptorTable,
  ) -> XrfResult<Self> {
    let mut state: Self = Self {
      writer: None,
      path: PathBuf::new(),
      position: 0,
      descriptors,
      aliases: HashMap::new(),
      layout,
      volume_index: 0,
      result: ArchivePackResult::default(),
    };

    state.open_volume(config)?;

    Ok(state)
  }

  /// Place one entry in a volume and write it there.
  ///
  /// Reads as it decides: measure the entry, offer it to the volume holding its twin, make room for it, then write it.
  fn add_entry(&mut self, config: &ArchivePackConfig, dict: &mut Dict, entry: &ArchivePackEntry) -> XrfResult<()> {
    let contents: Vec<u8> = fs::read(&entry.path)?;
    let size_real: u32 = u32::try_from(contents.len()).map_err(|_| {
      XrfError::new_invalid_error(format!(
        "File '{}' is {} bytes, larger than an archive entry can describe",
        entry.name,
        contents.len()
      ))
    })?;

    let crc: u32 = crc32fast::hash(&contents);
    let name: DescriptorName = DescriptorName::encode(&entry.name)?;

    self.result.size_source += u64::from(size_real);

    // An alias costs a row and no payload, but only in the volume holding the payload it points at: moving the entry
    // on turns it back into a copy, so this volume is offered it first.
    if let Some((offset, size_compressed)) = self.find_alias(&contents, size_real, crc)? {
      if self.fits(0, name.row_size()) {
        self.result.files_aliased += 1;
        self
          .descriptors
          .push_entry(&name, size_real, size_compressed, crc, offset);

        return Ok(());
      }

      self.start_next_volume(config)?;
    }

    let payload: Vec<u8> = self.compress_payload(config, dict, entry, &contents)?;
    let size_compressed: u32 = to_format_size(payload.len(), "archive entry payload")?;

    self.make_room_for(config, &entry.name, payload.len() as u64, name.row_size())?;

    let offset: u32 = u32::try_from(self.position).map_err(|_| {
      XrfError::new_invalid_error(format!(
        "Volume grew past {} bytes, which an entry offset cannot address",
        u32::MAX
      ))
    })?;

    if let Some(writer) = self.writer.as_mut() {
      writer.write_all(&payload)?;
    }

    self.position += payload.len() as u64;

    self.aliases.entry((size_real, crc)).or_default().push(ArchiveAlias {
      path: entry.path.clone(),
      offset,
      size_compressed,
    });

    self
      .descriptors
      .push_entry(&name, size_real, size_compressed, crc, offset);

    Ok(())
  }

  /// Make room in a volume for an entry of this size, closing the current one when that is what it takes.
  ///
  /// Closing is the ordinary answer to a full volume. It is not an answer to an entry no empty volume could hold: the
  /// next volume would open with the same repeated parts and reject it too, so the pack stops instead of publishing a
  /// volume past the cap it advertised.
  fn make_room_for(
    &mut self,
    config: &ArchivePackConfig,
    entry_name: &str,
    payload_size: u64,
    row_size: u64,
  ) -> XrfResult<()> {
    if !self.fits(payload_size, row_size) && self.descriptors.get_entries() > 0 {
      self.start_next_volume(config)?;
    }

    if self.fits(payload_size, row_size) {
      return Ok(());
    }

    Err(XrfError::new_invalid_error(format!(
      "File '{entry_name}' needs a volume of {} bytes to itself, past the configured maximum volume size of {} bytes",
      self.get_projected_size(payload_size, row_size),
      self.layout.get_maximum()
    )))
  }

  /// Whether the current volume can take a payload and its descriptor row and still close within the cap.
  fn fits(&self, payload_size: u64, row_size: u64) -> bool {
    self.get_projected_size(payload_size, row_size) <= self.layout.get_maximum()
  }

  /// Bytes the current volume would occupy once closed, were it to take one more payload and descriptor row.
  fn get_projected_size(&self, payload_size: u64, row_size: u64) -> u64 {
    ArchiveVolumeLayout::get_closed_size(self.position + payload_size, self.descriptors.get_size() + row_size)
  }

  fn open_volume(&mut self, config: &ArchivePackConfig) -> XrfResult<()> {
    self.path = config.destination.join(config.volume_name(self.volume_index));
    self.aliases.clear();
    self.descriptors.reset();

    let mut writer: BufWriter<File> = BufWriter::new(File::create(&self.path)?);

    if let Some(header) = self.layout.get_header() {
      writer.write_all(&CHUNK_ID_HEADER.to_le_bytes())?;
      writer.write_all(&to_format_size::<u32>(header.len(), "archive header chunk")?.to_le_bytes())?;
      writer.write_all(header)?;
    }

    // The data chunk's size is only known once every payload is in, so leave room and patch it later.
    writer.write_all(&CHUNK_ID_DATA.to_le_bytes())?;
    writer.write_all(&0u32.to_le_bytes())?;

    // Exactly the bytes just written, which is what the layout measures a volume's opening as.
    self.position = self.layout.get_opening_size();
    self.writer = Some(writer);

    Ok(())
  }

  fn close_volume(&mut self) -> XrfResult<()> {
    let Some(mut writer) = self.writer.take() else {
      return Ok(());
    };

    // Everything past the opening is payload, since the descriptor chunk has yet to be written.
    let data_size: u64 = self.position - self.layout.get_opening_size();

    self.position += self.descriptors.write_to(&mut writer)?;

    // Patch the data chunk's size now that the payloads are behind it. The cap keeps a volume well inside what the
    // field can declare, so this narrowing cannot lose a payload.
    writer.seek(SeekFrom::Start(self.layout.get_data_size_position()))?;
    writer.write_all(&(data_size as u32).to_le_bytes())?;
    writer.flush()?;

    self.result.volumes.push(self.path.clone());
    self.result.size_written += self.position;
    self.volume_index += 1;

    Ok(())
  }

  /// Close the volume being written and open the next one of the set.
  fn start_next_volume(&mut self, config: &ArchivePackConfig) -> XrfResult<()> {
    self.close_volume()?;

    self.open_volume(config)
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
}
