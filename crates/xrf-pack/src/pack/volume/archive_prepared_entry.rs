use lzokay::compress::Dict;
use xrf_error::{XrfError, XrfResult};
use xrf_extension::{XrayExtension, XrayExtensionOf};

use crate::pack::ArchivePackEntryOutcome;
use crate::pack::config::{ArchivePackConfig, ArchivePackMode};
use crate::pack::source::{ArchivePackEntry, ArchivePackPayloads};

/// One entry read, measured and coded: everything about it that does not depend on where it lands.
///
/// Preparing an entry reads nothing the volume writer holds, so entries can be prepared ahead of the writer and on
/// every worker while it places them one at a time and in order; what lands in a volume is the same either way.
pub(crate) struct ArchivePreparedEntry {
  /// The bytes as read, which a stored payload is and an alias is compared against.
  pub(crate) contents: Vec<u8>,
  pub(crate) size_real: u32,
  pub(crate) crc: u32,
  /// The compressed bytes, where compressing paid.
  coded: Option<Vec<u8>>,
  /// How the payload came to be: stored, empty, reverted or compressed.
  pub(crate) outcome: ArchivePackEntryOutcome<'static>,
}

impl ArchivePreparedEntry {
  /// Extensions the engine expects to find compressed; everything else is stored.
  ///
  /// `testVFS` in `xrCompress.cpp` inverts the intuitive rule: only text the engine parses is worth the
  /// LZO round trip, because meshes, textures, and sounds are already compressed in their own formats.
  const COMPRESSED_EXTENSIONS: &[XrayExtension] = &[XrayExtension::Xml, XrayExtension::Ltx, XrayExtension::Script];

  /// Compression must save more than this to be worth keeping, matching xrCompress.
  const COMPRESSION_MARGIN: usize = 16;

  /// Whether the engine expects this entry compressed, which is what makes preparing it cost more than reading it.
  pub(crate) fn is_compressed_kind(config: &ArchivePackConfig, entry: &ArchivePackEntry) -> bool {
    config.mode == ArchivePackMode::Compress
      && XrayExtensionOf::of(&entry.name)
        .known()
        .is_some_and(|extension| Self::COMPRESSED_EXTENSIONS.contains(&extension))
  }

  /// Reads an entry to be stored as it is read, which is every entry that is not [`Self::is_compressed_kind`].
  ///
  /// # Errors
  ///
  /// Returns the read error of its payload, and an invalid error for a file larger than an entry can describe.
  pub(crate) fn read(payloads: &ArchivePackPayloads<'_>, entry: &ArchivePackEntry) -> XrfResult<Self> {
    let contents: Vec<u8> = payloads.read(entry)?;
    let size_real: u32 = u32::try_from(contents.len()).map_err(|_| {
      XrfError::new_invalid_error(format!(
        "File '{}' is {} bytes, larger than an archive entry can describe",
        entry.name,
        contents.len()
      ))
    })?;

    Ok(Self {
      crc: crc32fast::hash(&contents),
      contents,
      size_real,
      coded: None,
      outcome: ArchivePackEntryOutcome::Stored,
    })
  }

  /// Reads an entry and compresses it when the engine expects it compressed and the result is actually smaller.
  ///
  /// # Errors
  ///
  /// Returns what [`Self::read`] answers with, and an unexpected error when compression fails.
  pub(crate) fn read_coded(
    config: &ArchivePackConfig,
    payloads: &ArchivePackPayloads<'_>,
    entry: &ArchivePackEntry,
    dict: &mut Dict,
  ) -> XrfResult<Self> {
    let mut prepared: Self = Self::read(payloads, entry)?;

    (prepared.coded, prepared.outcome) = Self::code(config, entry, &prepared.contents, dict)?;

    Ok(prepared)
  }

  /// The bytes the volume holds for it: compressed where that paid, as read otherwise.
  pub(crate) fn get_payload(&self) -> &[u8] {
    self.coded.as_deref().unwrap_or(&self.contents)
  }

  /// The compressed payload where the engine expects one and compressing pays, with how the payload came to be.
  ///
  /// Answers the outcome too, since the sizes alone cannot tell a stored payload from one that reverted or was empty.
  /// Deciding only, never counting: what a decision is worth to the summary is
  /// [`crate::pack::ArchivePackResult::record_outcome`], so the three that all end up stored say so in one place.
  fn code(
    config: &ArchivePackConfig,
    entry: &ArchivePackEntry,
    contents: &[u8],
    dict: &mut Dict,
  ) -> XrfResult<(Option<Vec<u8>>, ArchivePackEntryOutcome<'static>)> {
    if !Self::is_compressed_kind(config, entry) {
      return Ok((None, ArchivePackEntryOutcome::Stored));
    }

    if contents.is_empty() {
      return Ok((None, ArchivePackEntryOutcome::Empty));
    }

    let compressed: Vec<u8> = lzokay::compress::compress_with_dict(contents, dict)
      .map_err(|error| XrfError::new_unexpected_error(format!("Failed to compress '{}': {error}", entry.name)))?;

    // A payload that barely shrinks costs more to decompress than it saves, so it reverts to stored.
    // The reader tells the two apart by the sizes alone, so this must stay a real size difference.
    if compressed.len() + Self::COMPRESSION_MARGIN >= contents.len() {
      return Ok((None, ArchivePackEntryOutcome::Reverted));
    }

    Ok((Some(compressed), ArchivePackEntryOutcome::Compressed))
  }
}
