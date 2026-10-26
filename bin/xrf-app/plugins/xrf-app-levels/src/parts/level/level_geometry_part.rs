use std::sync::Arc;

use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionPart, SessionParts, SessionSnapshot};
use xrf_chunk::{InMemoryChunkDataSource, XRayByteOrder};
use xrf_level::{LevelGeomFile, LevelGeomSource};
use xrf_vfs::{XrayInputRecord, XraySearch};

use crate::level::{LevelSource, SelectedLevel};

/// What a change event calls a level's render geometry when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_GEOMETRY_PART: &str = "geometry";

/// The render geometry a level's sectors are packed from, `level.geom` read whole and served by range, so a change to
/// it moves the geometry and not the level. Shared rather than locked: a range is read without moving the source, so
/// sectors pack side by side.
pub(crate) struct LevelGeometryPart;

impl SessionPart for LevelGeometryPart {
  type Value = Result<Arc<LevelGeomSource<InMemoryChunkDataSource>>, String>;

  const NAME: &'static str = LEVEL_GEOMETRY_PART;
}

impl LevelGeometryPart {
  /// The store with `geometry` kept as the level's geometry part, read from `inputs` as the level opened.
  pub fn with(
    parts: SessionParts,
    geometry: Arc<LevelGeomSource<InMemoryChunkDataSource>>,
    inputs: XrayInputRecord,
  ) -> SessionParts {
    parts.with::<Self>(Ok(geometry), inputs)
  }

  /// The open level's render geometry, read the first time anything asks and kept.
  ///
  /// # Errors
  ///
  /// Returns why `level.geom` could not be read, which every later ask answers with again.
  pub fn get(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
  ) -> Result<Arc<LevelGeomSource<InMemoryChunkDataSource>>, String> {
    current
      .parts
      .get_or_record::<Self>(search, |search| Self::read(&current.source, search).map(Arc::new))
  }

  /// Reads a level's render geometry, its payloads left on the heap where they were read.
  ///
  /// # Errors
  ///
  /// Returns why `level.geom` could not be read.
  pub fn read(source: &LevelSource, search: &XraySearch) -> TauriResult<LevelGeomSource<InMemoryChunkDataSource>> {
    LevelGeomSource::open_from_bytes::<XRayByteOrder>(source.read_file(search, LevelGeomFile::FILE_NAME)?)
      .map_err(|error| source.describe_read_failure(LevelGeomFile::FILE_NAME, &error))
  }
}
