use serde::Serialize;
use xrf_db::LevelGeomFile;
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_entry_reader::ArchiveEntryReader;
use crate::plugins::archives::describe::level::archive_level_geom_layout::ArchiveLevelGeomLayout;

/// Everything the viewer says about a level's render geometry.
///
/// Read down to its shape alone: the file reaches 143 MB and the vertices themselves answer nothing a description
/// asks, so each buffer's payload is stepped over rather than held.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelGeomDescription {
  /// Whether this is the detail twin, `level.geomX`, which the renderer draws distant geometry from.
  pub is_detail: bool,
  pub vertex_buffers: usize,
  pub index_buffers: usize,
  pub vertices: u64,
  pub indices: u64,
  /// Triangles the indices draw, taking them as triangle lists.
  pub triangles: u64,
  /// Meshes that drop detail with distance, which `level.geomX` carries none of.
  pub progressive_meshes: usize,
  /// Detail levels across every progressive mesh.
  pub detail_levels: usize,
  /// The vertex layouts the buffers are built from, grouped.
  pub layouts: Vec<ArchiveLevelGeomLayout>,
  pub size: u64,
}

impl ArchiveLevelGeomDescription {
  /// Reads the render geometry an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry cannot be opened, or is not render geometry this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str, is_detail: bool) -> XrfResult<Self> {
    let mut reader: ArchiveEntryReader = source.open_entry(name)?;
    let size: u64 = reader.size();
    let file: LevelGeomFile = reader.read_container()?;

    Ok(Self {
      is_detail,
      vertex_buffers: file.vertex_buffers.len(),
      index_buffers: file.index_buffers.len(),
      vertices: file.get_vertices_count(),
      indices: file.get_indices_count(),
      triangles: file.get_triangles_count(),
      progressive_meshes: file.slide_windows.len(),
      detail_levels: file.get_detail_levels_count(),
      layouts: ArchiveLevelGeomLayout::of_all(&file.vertex_buffers),
      size,
    })
  }
}
