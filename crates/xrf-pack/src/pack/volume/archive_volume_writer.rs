use std::borrow::Cow;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;

use lzokay::compress::Dict;
use xrf_archive::{CHUNK_ID_DATA, CHUNK_ID_METADATA};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::to_format_size;

use crate::pack::config::{ArchivePackConfig, ArchivePackMode};
use crate::pack::source::ArchivePackEntry;
use crate::pack::volume::{
  ArchiveAlias, ArchiveAliasCandidate, ArchiveAliasTable, ArchiveDescriptorTable, ArchiveVolumeLayout, DescriptorName,
};
use crate::pack::{ArchivePackEntryOutcome, ArchivePackNarrator, ArchivePackResult};

/// Extensions the engine expects to find compressed; everything else is stored.
///
/// `testVFS` in `xrCompress.cpp` inverts the intuitive rule: only text the engine parses is worth the
/// LZO round trip, because meshes, textures, and sounds are already compressed in their own formats.
const COMPRESSED_EXTENSIONS: [&str; 3] = ["xml", "ltx", "script"];

/// Compression must save more than this to be worth keeping, matching xrCompress.
const COMPRESSION_MARGIN: usize = 16;

/// Writes the volumes of one archive set, one at a time.
///
/// Entries arrive one by one and the writer decides where each lands, so nothing here knows the source tree or how
/// long the run took: [`Self::finish`] reports only what was observed writing, and the caller composes the rest.
///
/// Every placement is counted and said once, from the one outcome that decided it, so the summary and the transcript
/// cannot describe the same entry two different ways.
pub(crate) struct ArchiveVolumeWriter<'a> {
  config: &'a ArchivePackConfig,
  narrator: &'a ArchivePackNarrator<'a>,
  layout: ArchiveVolumeLayout,
  file: Option<BufWriter<File>>,
  path: PathBuf,
  /// Absolute position in the current volume, which is what descriptor offsets record.
  position: u64,
  descriptors: ArchiveDescriptorTable,
  /// Borrows the entries written so far; the source table they belong to outlives the writer.
  aliases: ArchiveAliasTable<'a>,
  /// The coder's working state, large enough to be worth building once on the heap and reusing for every entry.
  dict: Box<Dict>,
  volume_index: usize,
  result: ArchivePackResult,
}

impl<'a> ArchiveVolumeWriter<'a> {
  /// Open the first volume of the set, seeded with the directory rows every volume of it repeats.
  pub(crate) fn open(
    config: &'a ArchivePackConfig,
    narrator: &'a ArchivePackNarrator<'a>,
    layout: ArchiveVolumeLayout,
    descriptors: ArchiveDescriptorTable,
  ) -> XrfResult<Self> {
    let mut writer: Self = Self {
      config,
      narrator,
      layout,
      file: None,
      path: PathBuf::new(),
      position: 0,
      descriptors,
      aliases: ArchiveAliasTable::default(),
      dict: Dict::new(),
      volume_index: 0,
      result: ArchivePackResult::default(),
    };

    writer.open_volume()?;

    Ok(writer)
  }

  /// Place one entry in a volume and write it there.
  ///
  /// Reads as it decides: measure the entry, offer it to the volume holding its twin, make room for it, write it.
  ///
  /// The payload borrows the bytes that were read whenever it is stored rather than compressed, which is every entry
  /// the engine does not expect compressed. Copying them would double the largest entry's cost in memory and memcpy
  /// the whole archive on the way past.
  pub(crate) fn write_entry(&mut self, entry: &'a ArchivePackEntry) -> XrfResult<()> {
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
    if let Some(ArchiveAliasCandidate { source, alias }) = self.aliases.find(&contents, size_real, crc)? {
      if self.fits(0, name.get_row_size()) {
        self.place(
          entry,
          ArchivePackEntryOutcome::Aliased { source: &source.name },
          size_real,
          alias.size_compressed,
        );
        self
          .descriptors
          .push_entry(&name, size_real, alias.size_compressed, crc, alias.offset);

        return Ok(());
      }

      self.start_next_volume()?;
    }

    let (payload, outcome): (Cow<'_, [u8]>, ArchivePackEntryOutcome) = self.compress_payload(entry, &contents)?;
    let size_compressed: u32 = to_format_size(payload.len(), "archive entry payload")?;

    self.make_room_for(&entry.name, payload.len() as u64, name.get_row_size())?;

    let offset: u32 = u32::try_from(self.position).map_err(|_| {
      XrfError::new_invalid_error(format!(
        "Volume grew past {} bytes, which an entry offset cannot address",
        u32::MAX
      ))
    })?;

    if let Some(file) = self.file.as_mut() {
      file.write_all(&payload)?;
    }

    self.position += payload.len() as u64;

    self.place(entry, outcome, size_real, size_compressed);

    self.aliases.record(
      entry,
      size_real,
      crc,
      ArchiveAlias {
        offset,
        size_compressed,
      },
    );

    self
      .descriptors
      .push_entry(&name, size_real, size_compressed, crc, offset);

    Ok(())
  }

  /// Close the volume still open and report what the set came to.
  pub(crate) fn finish(mut self) -> XrfResult<ArchivePackResult> {
    self.close_volume()?;

    Ok(self.result)
  }

  /// What was written so far, abandoning the volume in progress.
  ///
  /// For a run that stopped between entries. The open volume is intentionally not closed: closing it would write a
  /// descriptor table and produce a structurally valid archive that is missing entries, which is a worse thing to
  /// leave behind than an obviously unfinished file.
  pub(crate) fn abandon(self) -> ArchivePackResult {
    self.result
  }

  /// Book one entry as placed: count it and say it, from the one outcome that decided both.
  ///
  /// Called once per entry, after its bytes are wherever they are going to be, so a run that fails writing them has
  /// neither counted nor claimed the entry.
  fn place(
    &mut self,
    entry: &ArchivePackEntry,
    outcome: ArchivePackEntryOutcome,
    size_real: u32,
    size_compressed: u32,
  ) {
    self.result.record_outcome(outcome);
    self
      .narrator
      .describe_entry(&entry.name, outcome, size_real, size_compressed);
  }

  /// Make room in a volume for an entry of this size, closing the current one when that is what it takes.
  ///
  /// Closing is the ordinary answer to a full volume. It is not an answer to an entry no empty volume could hold: the
  /// next volume would open with the same repeated parts and reject it too, so the pack stops instead of publishing a
  /// volume past the cap it advertised.
  fn make_room_for(&mut self, entry_name: &str, payload_size: u64, row_size: u64) -> XrfResult<()> {
    if !self.fits(payload_size, row_size) && self.descriptors.get_entries() > 0 {
      self.start_next_volume()?;
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

  fn open_volume(&mut self) -> XrfResult<()> {
    self.path = self.config.destination.join(self.config.volume_name(self.volume_index));
    self.aliases.reset();
    self.descriptors.reset();

    // Recorded and said before the file is created rather than after it is closed: from here on the path exists and
    // has replaced whatever stood there, so a run that stops has still touched it.
    self.result.volumes_opened.push(self.path.clone());
    self.narrator.describe_opened_volume(&self.path);

    let mut file: BufWriter<File> = BufWriter::new(File::create(&self.path)?);

    if let Some(header) = self.layout.get_header() {
      file.write_all(&CHUNK_ID_METADATA.to_le_bytes())?;
      file.write_all(&to_format_size::<u32>(header.len(), "archive header chunk")?.to_le_bytes())?;
      file.write_all(header)?;
    }

    // The data chunk's size is only known once every payload is in, so leave room and patch it later.
    file.write_all(&CHUNK_ID_DATA.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;

    // Exactly the bytes just written, which is what the layout measures a volume's opening as.
    self.position = self.layout.get_opening_size();
    self.file = Some(file);

    Ok(())
  }

  fn close_volume(&mut self) -> XrfResult<()> {
    let Some(mut file) = self.file.take() else {
      return Ok(());
    };

    // Everything past the opening is payload, since the descriptor chunk has yet to be written.
    let data_size: u64 = self.position - self.layout.get_opening_size();

    self.position += self.descriptors.write_to(&mut file)?;

    // Patch the data chunk's size now that the payloads are behind it. The cap keeps a volume well inside what the
    // field can declare, so this narrowing cannot lose a payload.
    file.seek(SeekFrom::Start(self.layout.get_data_size_position()))?;
    file.write_all(&(data_size as u32).to_le_bytes())?;
    file.flush()?;

    self
      .narrator
      .describe_closed_volume(&self.path, self.position, self.descriptors.get_entries());

    self.result.volumes.push(self.path.clone());
    self.result.size_written += self.position;
    self.volume_index += 1;

    Ok(())
  }

  /// Close the volume being written and open the next one of the set.
  fn start_next_volume(&mut self) -> XrfResult<()> {
    self.close_volume()?;

    self.open_volume()
  }

  /// Compress a payload when the engine expects it compressed and the result is actually smaller.
  ///
  /// Answers the payload with how it came to be, since the sizes alone cannot tell a stored payload from one that
  /// reverted or was empty. Deciding only, never counting: what a decision is worth to the summary is
  /// [`ArchivePackResult::record_outcome`], so the three that all end up stored say so in one place.
  fn compress_payload<'contents>(
    &mut self,
    entry: &ArchivePackEntry,
    contents: &'contents [u8],
  ) -> XrfResult<(Cow<'contents, [u8]>, ArchivePackEntryOutcome<'static>)> {
    let is_expected_compressed: bool = self.config.mode == ArchivePackMode::Compress
      && entry
        .name
        .rsplit_once('.')
        .is_some_and(|(_, extension)| COMPRESSED_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()));

    if !is_expected_compressed {
      return Ok((Cow::Borrowed(contents), ArchivePackEntryOutcome::Stored));
    }

    if contents.is_empty() {
      return Ok((Cow::Borrowed(contents), ArchivePackEntryOutcome::Empty));
    }

    let compressed: Vec<u8> = lzokay::compress::compress_with_dict(contents, &mut self.dict)
      .map_err(|error| XrfError::new_unexpected_error(format!("Failed to compress '{}': {error}", entry.name)))?;

    // A payload that barely shrinks costs more to decompress than it saves, so it reverts to stored.
    // The reader tells the two apart by the sizes alone, so this must stay a real size difference.
    if compressed.len() + COMPRESSION_MARGIN >= contents.len() {
      return Ok((Cow::Borrowed(contents), ArchivePackEntryOutcome::Reverted));
    }

    Ok((Cow::Owned(compressed), ArchivePackEntryOutcome::Compressed))
  }
}
