//! The render geometry a level's sectors are packed from: `level.geom`, read whole and served by range.

use std::sync::Arc;

use xrf_chunk::InMemoryChunkDataSource;
use xrf_level::LevelGeomSource;
use xrf_vfs::{XrayInputRecord, XraySearch};

use crate::core::session::{SessionPart, SessionParts, SessionSnapshot};
use crate::plugins::levels::read::read_geometry;
use crate::plugins::levels::state::SelectedLevel;

/// What a change event calls a level's render geometry when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_GEOMETRY_PART: &str = "geometry";

/// The level's render geometry, shared rather than locked: a range is read without moving the source, so sectors pack
/// side by side.
pub(crate) type LevelGeometry = LevelGeomSource<InMemoryChunkDataSource>;

/// The level's render geometry, read from `level.geom` alone, so a change to it moves the geometry and not the level.
struct LevelGeometryPart;

impl SessionPart for LevelGeometryPart {
  type Value = Result<Arc<LevelGeometry>, String>;

  const NAME: &'static str = LEVEL_GEOMETRY_PART;
}

/// The store with `geometry` kept as the level's geometry part, read from `inputs` as the level opened.
pub fn with_level_geometry(parts: SessionParts, geometry: Arc<LevelGeometry>, inputs: XrayInputRecord) -> SessionParts {
  parts.with::<LevelGeometryPart>(Ok(geometry), inputs)
}

/// The open level's render geometry, read the first time anything asks and kept.
///
/// # Errors
///
/// Returns why `level.geom` could not be read.
pub fn get_level_geometry(
  current: &SessionSnapshot<SelectedLevel>,
  search: &XraySearch,
) -> Result<Arc<LevelGeometry>, String> {
  current
    .parts
    .get_or_record::<LevelGeometryPart>(search, |search| read_geometry(&current.source, search).map(Arc::new))
}
