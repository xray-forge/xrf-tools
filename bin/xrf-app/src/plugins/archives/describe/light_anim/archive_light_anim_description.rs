use serde::Serialize;
use xrf_db::{LightAnimFile, XRayByteOrder};
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::light_anim::archive_light_anim_item::ArchiveLightAnimItem;

/// Everything the viewer says about the colour animation library.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLightAnimDescription {
  pub version: u16,
  /// Whether the colours are stored channel-swapped, which the engine corrects as it loads them.
  pub is_bgr: bool,
  pub items: Vec<ArchiveLightAnimItem>,
  /// Keys across every animation.
  pub keys: usize,
}

impl ArchiveLightAnimDescription {
  /// Reads the colour animation library an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not an animation library this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: LightAnimFile = LightAnimFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      version: file.version,
      is_bgr: file.is_bgr(),
      keys: file.get_keys_count(),
      items: ArchiveLightAnimItem::of_all(&file.items),
    })
  }
}
