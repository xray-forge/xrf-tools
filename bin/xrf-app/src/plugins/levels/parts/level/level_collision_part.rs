use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelCformFile, LevelCformGeometry, LevelCformTracer};
use xrf_vfs::XraySearch;

use crate::core::session::{SessionPart, SessionSnapshot};
use crate::plugins::levels::report::report_collision;
use crate::plugins::levels::state::{COLLISION_FILE, LevelSource, SelectedLevel};

/// What a change event calls a level's collision form when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_COLLISION_PART: &str = "collision";

/// The level's collision form as rays test it, read once and shared by everything that traces it: it lights spawned
/// objects and collides particles. It must stay pooled, as parts are by default: its tracer orders the form's
/// triangles over Rayon.
pub(crate) struct LevelCollisionPart;

impl SessionPart for LevelCollisionPart {
  type Value = Result<Arc<LevelCformTracer>, String>;

  const NAME: &'static str = LEVEL_COLLISION_PART;
}

impl LevelCollisionPart {
  /// The open level's collision form in its tracer's hierarchy, built the first time anything asks and kept.
  ///
  /// # Errors
  ///
  /// Returns why the collision form cannot be read, which every later ask answers with again.
  pub fn get(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Result<Arc<LevelCformTracer>, String> {
    current
      .parts
      .get_or_record::<Self>(search, |search| Self::read(&current.source, search))
  }

  /// The level's collision form as the grass is planted onto it, every triangle with its material.
  ///
  /// # Errors
  ///
  /// Returns why the collision form cannot be read.
  pub fn read_geometry(source: &LevelSource, search: &XraySearch) -> Result<LevelCformGeometry, String> {
    LevelCformFile::read_geometry_from_bytes::<XRayByteOrder>(source.read_file(search, COLLISION_FILE)?)
      .map_err(|error| source.describe_read_failure(COLLISION_FILE, &error))
  }

  fn read(source: &LevelSource, search: &XraySearch) -> Result<Arc<LevelCformTracer>, String> {
    let started: Instant = Instant::now();
    let geometry: LevelCformGeometry = Self::read_geometry(source, search)?;
    let read: Duration = started.elapsed();
    let tracer: LevelCformTracer = LevelCformTracer::new(&geometry);

    report_collision(source, tracer.get_triangle_count(), read, started);

    Ok(Arc::new(tracer))
  }
}
