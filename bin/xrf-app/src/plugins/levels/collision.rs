//! The level's collision form as rays test it, read once and shared by everything that traces it.

use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelCformFile, LevelCformGeometry, LevelCformTracer};
use xrf_vfs::XraySearch;

use crate::core::session::{SessionPart, SessionSnapshot};
use crate::plugins::levels::read::read_file;
use crate::plugins::levels::report::report_collision;
use crate::plugins::levels::state::{COLLISION_FILE, LevelSource, SelectedLevel};

/// The collision form as rays test it, which lights spawned objects and collides particles. It must stay pooled, as
/// parts are by default: its tracer orders the form's triangles over Rayon.
struct LevelCollisionPart;

impl SessionPart for LevelCollisionPart {
  type Value = Result<Arc<LevelCformTracer>, String>;
}

/// The open level's collision form in its tracer's hierarchy, built the first time anything asks and kept.
///
/// # Errors
///
/// Returns why the collision form cannot be read, which every later ask answers with again.
pub fn get_level_collision(
  current: &SessionSnapshot<SelectedLevel>,
  search: &XraySearch,
) -> Result<Arc<LevelCformTracer>, String> {
  current
    .parts
    .get_or_record::<LevelCollisionPart>(search, |search| read_collision(&current.source, search))
}

fn read_collision(source: &LevelSource, search: &XraySearch) -> Result<Arc<LevelCformTracer>, String> {
  let started: Instant = Instant::now();
  let geometry: LevelCformGeometry =
    LevelCformFile::read_geometry_from_bytes::<XRayByteOrder>(read_file(source, search, COLLISION_FILE)?).map_err(
      |error| {
        format!(
          "Failed to read '{COLLISION_FILE}' of level '{}': {error}",
          source.get_label()
        )
      },
    )?;
  let read: Duration = started.elapsed();
  let tracer: LevelCformTracer = LevelCformTracer::new(&geometry);

  report_collision(source, tracer.get_triangle_count(), read, started);

  Ok(Arc::new(tracer))
}
