use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelAiFile, LevelAiHeader, LevelAiNode, LevelAiNodes};
use xrf_renderer::{RenderAiMapNode, RenderLevelAiMap};
use xrf_vfs::XraySearch;

use crate::level::{LevelSource, SelectedLevel};

/// What a change event calls what a level's AI map was read from when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_AI_MAP_PART: &str = "aiMap";

/// What a level's AI map was read from, `level.ai`; the renderer holds the map its debug views show.
pub(crate) struct LevelAiMapPart;

impl SessionPart for LevelAiMapPart {
  type Value = ();

  const NAME: &'static str = LEVEL_AI_MAP_PART;

  const IS_POOLED: bool = false;
}

impl LevelAiMapPart {
  /// The open level's AI map as the renderer shows it, what it is read from recorded as its AI map part, or `None` for a
  /// level that ships none.
  ///
  /// # Errors
  ///
  /// Returns why the map cannot be read.
  pub(crate) fn read(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
  ) -> TauriResult<Option<RenderLevelAiMap>> {
    current
      .parts
      .record_read::<Self, _>(search, |search| Self::read_recorded(&current.source, search))
  }

  /// The map read, through `search`.
  fn read_recorded(source: &LevelSource, search: &XraySearch) -> TauriResult<Option<RenderLevelAiMap>> {
    let Some(bytes) = source.read_optional_file(search, LevelAiFile::FILE_NAME)? else {
      return Ok(None);
    };
    let (file, nodes) = LevelAiFile::read_with_nodes_from_bytes::<XRayByteOrder>(bytes)
      .map_err(|error| source.describe_read_failure(LevelAiFile::FILE_NAME, &error))?;

    Ok(Some(Self::describe(&file.header, &nodes)))
  }

  /// The map of `header` and `nodes` as its debug views read it.
  fn describe(header: &LevelAiHeader, nodes: &LevelAiNodes) -> RenderLevelAiMap {
    let pack_cover = |cover: [u8; 4]| {
      cover.iter().zip([0, 4, 8, 12]).fold(0u16, |packed, (value, shift)| {
        packed | (u16::from(*value & 0xF) << shift)
      })
    };

    RenderLevelAiMap {
      min_x: header.bounds.min.x,
      min_z: header.bounds.min.z,
      cell_size: header.size,
      row_length: header.get_row_length(),
      column_length: header.get_column_length(),
      floor: header.bounds.min.y,
      height: header.size_y,
      nodes: nodes
        .iter_nodes::<XRayByteOrder>()
        .map(|node: LevelAiNode| RenderAiMapNode {
          cell: node.packed_xz,
          height: node.packed_y,
          high_cover: pack_cover(node.high_cover),
          low_cover: pack_cover(node.low_cover),
          missing_links: (0..4)
            .filter(|side| node.links[*side] >= header.count)
            .fold(0u8, |missing, side| missing | (1 << side)),
        })
        .collect(),
    }
  }
}
