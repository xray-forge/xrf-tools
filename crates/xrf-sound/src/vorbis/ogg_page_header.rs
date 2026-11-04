/// What an Ogg page's header says of the page, read without its body or checksum: where it lies, which stream it
/// belongs to, and the granule position it places.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OggPageHeader {
  /// Where the page starts in the bytes it was read from.
  pub(crate) start: usize,
  /// Where the page ends, after its body.
  pub(crate) end: usize,
  pub(crate) serial: u32,
  /// The granule position of the last packet ending on the page, or none where no packet ends.
  pub(crate) granule: Option<u64>,
  /// Whether a packet ending on the page also starts on it, so that a reader starting at the page is handed one.
  pub(crate) has_whole_packet: bool,
}

impl OggPageHeader {
  /// The capture pattern every Ogg page starts with, so an Ogg stream's first bytes.
  pub(crate) const CAPTURE: &'static [u8] = b"OggS";

  const FIXED_LENGTH: usize = 27;

  const CONTINUED_FLAG: u8 = 0x01;

  /// The header of a version 0 page starting at `start` in `bytes`, when the whole page lies in them.
  pub(crate) fn read(bytes: &[u8], start: usize) -> Option<Self> {
    let page: &[u8] = bytes.get(start..)?;

    if !page.starts_with(Self::CAPTURE) || *page.get(4)? != 0 {
      return None;
    }

    let lacing: &[u8] = page.get(Self::FIXED_LENGTH..Self::FIXED_LENGTH + usize::from(*page.get(26)?))?;
    let length: usize = Self::FIXED_LENGTH + lacing.len() + lacing.iter().map(|&size| usize::from(size)).sum::<usize>();
    let granule: u64 = u64::from_le_bytes(page[6..14].try_into().ok()?);
    let ends_count: usize = lacing.iter().filter(|&&size| size < 255).count();
    let is_continued: bool = page[5] & Self::CONTINUED_FLAG != 0;

    (length <= page.len()).then_some(Self {
      start,
      end: start + length,
      serial: u32::from_le_bytes(page[14..18].try_into().ok()?),
      // A granule of all ones marks a page where no packet ends.
      granule: (granule != u64::MAX).then_some(granule),
      has_whole_packet: ends_count > usize::from(is_continued),
    })
  }

  /// The first page of stream `serial` starting at or after `offset` in `bytes`.
  pub(crate) fn find(bytes: &[u8], offset: usize, serial: u32) -> Option<Self> {
    let mut start: usize = offset;

    while let Some(found) = bytes
      .get(start..)?
      .windows(Self::CAPTURE.len())
      .position(|window| window == Self::CAPTURE)
    {
      // Another stream's page is stepped into rather than over: a capture pattern inside a page's audio could pass
      // for a header claiming more bytes than the page holds.
      match Self::read(bytes, start + found) {
        Some(header) if header.serial == serial => return Some(header),
        _ => start += found + 1,
      }
    }

    None
  }
}
