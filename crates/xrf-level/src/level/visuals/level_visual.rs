use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Matrix4x4;
use xrf_ogf::{
  OgfChildrenLinkChunk, OgfFastpathChunk, OgfGeometryContainerChunk, OgfHeaderChunk, OgfLodDefinitionChunk,
  OgfModelType, OgfSwiContainerChunk, OgfSwiDataChunk, OgfTextureChunk, OgfTreeDefinitionChunk,
};

use crate::geom::level_geom_file::LevelGeomFile;
use crate::geom::window::level_geom_slide_window::LevelGeomSlideWindow;
use crate::geom::window::level_geom_slide_window_item::LevelGeomSlideWindowItem;
use crate::level::visuals::level_visual_windows::LevelVisualWindows;

/// One visual of a compiled level, as `fsL_VISUALS` stores it.
#[derive(Debug)]
pub struct LevelVisual {
  pub header: OgfHeaderChunk,
  /// The shader and texture the visual is dressed with, absent when the visual takes them from the level shader
  /// table by `header.shader_id` instead.
  pub texture: Option<OgfTextureChunk>,
  /// Where the visual's geometry sits in the buffers of `level.geom`.
  pub geometry: Option<OgfGeometryContainerChunk>,
  /// The same for `level.geomX`, with a progressive visual's windows over it, present only for a visual the compiler
  /// gave a fast path.
  pub fastpath: Option<OgfFastpathChunk>,
  /// Visuals this one composes, by their index in the run, for a hierarchy visual.
  pub children: Vec<u32>,
  /// Where a tree stands and how it is lit, absent for every visual that is not one.
  pub tree: Option<OgfTreeDefinitionChunk>,
  /// The impostor a `MT_LOD` visual draws in place of the trees it composes, seen from far enough away, or why it could
  /// not be read: a drawing detail, which costs the visual that part rather than the level.
  pub lod: Option<XrfResult<OgfLodDefinitionChunk>>,
  /// The windows of a progressive static mesh, stored with the visual, or why they could not be read.
  pub swi: Option<XrfResult<OgfSwiDataChunk>>,
  /// Which of `level.geom`'s window tables a progressive tree draws from, or why that could not be read.
  pub swi_container: Option<XrfResult<OgfSwiContainerChunk>>,
}

impl LevelVisual {
  /// Reads one visual from a reader positioned at its chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when the visual carries no header, or when a chunk it does carry does not end where its record
  /// says it should; its impostor and slide windows are kept as read or as the reason they were not instead.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Ok(Self {
      header: ChunkReader::find_required_by_id(&chunks, OgfHeaderChunk::CHUNK_ID)?.read_xr::<T, _>()?,
      texture: match ChunkReader::find_optional_by_id(&chunks, OgfTextureChunk::CHUNK_ID) {
        Some(mut chunk) => Some(chunk.read_xr::<T, _>()?),
        None => None,
      },
      children: match ChunkReader::find_optional_by_id(&chunks, OgfChildrenLinkChunk::CHUNK_ID) {
        Some(mut chunk) => chunk.read_xr::<T, OgfChildrenLinkChunk>()?.children,
        None => Vec::new(),
      },
      geometry: match ChunkReader::find_optional_by_id(&chunks, OgfGeometryContainerChunk::CHUNK_ID) {
        Some(mut chunk) => Some(chunk.read_xr::<T, _>()?),
        None => None,
      },
      fastpath: match ChunkReader::find_optional_by_id(&chunks, OgfFastpathChunk::CHUNK_ID) {
        Some(mut chunk) => Some(OgfFastpathChunk::read_from_chunk::<T, _>(&mut chunk)?),
        None => None,
      },
      tree: match ChunkReader::find_optional_by_id(&chunks, OgfTreeDefinitionChunk::CHUNK_ID) {
        Some(mut chunk) => Some(chunk.read_xr::<T, _>()?),
        None => None,
      },
      lod: ChunkReader::find_optional_by_id(&chunks, OgfLodDefinitionChunk::CHUNK_ID)
        .map(|mut chunk| chunk.read_xr::<T, _>()),
      swi: ChunkReader::find_optional_by_id(&chunks, OgfSwiDataChunk::CHUNK_ID)
        .map(|mut chunk| chunk.read_xr::<T, _>()),
      swi_container: ChunkReader::find_optional_by_id(&chunks, OgfSwiContainerChunk::CHUNK_ID)
        .map(|mut chunk| chunk.read_xr::<T, _>()),
    })
  }

  /// Whether the visual draws from the level's shared buffers at all.
  pub const fn is_drawable(&self) -> bool {
    self.geometry.is_some()
  }

  /// The window of the visual's indices holding its full detail, the first of its slide windows (see
  /// [`Self::get_windows`]); `None` for a visual that is not progressive, whose every index draws.
  ///
  /// # Errors
  ///
  /// As [`Self::get_windows`].
  pub fn get_full_detail_window(&self, geometry: &LevelGeomFile) -> XrfResult<Option<LevelGeomSlideWindow>> {
    Ok(
      self
        .get_windows(geometry)?
        .and_then(|windows| windows.get_full_detail()),
    )
  }

  /// The visual's slide windows, its levels of detail finest first: a static progressive mesh stores them itself, a
  /// progressive tree in the table of `geometry` its container names. `None` for a visual carrying none, which draws
  /// every index.
  ///
  /// # Errors
  ///
  /// Returns an error when the windows could not be read, the table named is not one `geometry` holds, or a progressive
  /// visual carries no window: the engine asserts it has them (`FProgressive::Load`, `FTreeVisual_PM::Load`), and its
  /// whole index run lays every level of detail end to end.
  pub fn get_windows<'a>(&'a self, geometry: &'a LevelGeomFile) -> XrfResult<Option<LevelVisualWindows<'a>>> {
    let windows: Option<LevelVisualWindows<'_>> = match (&self.swi, &self.swi_container) {
      (Some(swi), _) => {
        let swi: &OgfSwiDataChunk = swi
          .as_ref()
          .map_err(|error| XrfError::new_invalid_error(format!("A visual's slide windows were not read: {error}")))?;

        Some(LevelVisualWindows::Own(&swi.windows))
      }
      (None, Some(container)) => {
        let container: &OgfSwiContainerChunk = container.as_ref().map_err(|error| {
          XrfError::new_invalid_error(format!("A visual's slide window table was not read: {error}"))
        })?;
        // `CRender::getSWI` indexes the level's tables unchecked.
        let table: &LevelGeomSlideWindowItem = geometry
          .slide_windows
          .get(container.ext_swib_index as usize)
          .ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "A visual draws from slide window table {} of the {} the level holds",
              container.ext_swib_index,
              geometry.slide_windows.len()
            ))
          })?;

        Some(LevelVisualWindows::Table(&table.windows))
      }
      (None, None) => None,
    }
    .filter(|windows| !windows.is_empty());

    if windows.is_none() && OgfModelType::from_raw(self.header.model_type).is_some_and(OgfModelType::is_progressive) {
      return Err(XrfError::new_invalid_error(format!(
        "A visual is {} and carries no slide window, so its full detail is unknown",
        OgfModelType::label(self.header.model_type)
      )));
    }

    Ok(windows)
  }

  /// The transform placing this visual in the level, for one stored in its own space.
  pub const fn get_placement(&self) -> Option<&Matrix4x4> {
    match &self.tree {
      Some(definition) => Some(&definition.transform),
      None => None,
    }
  }
}
