//! The level's collision form as rays test it, read once and shared by everything that traces it.

use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelCformFile, LevelCformGeometry, LevelCformTracer};
use xrf_vfs::XrayProbe;

use crate::plugins::levels::read::read_file;
use crate::plugins::levels::report::report_collision;
use crate::plugins::levels::state::{COLLISION_FILE, LevelSource, SelectedLevel};

/// The open level's collision form in its tracer's hierarchy, built the first time anything asks and kept.
///
/// # Errors
///
/// Returns why the collision form cannot be read, which every later ask answers with again.
pub fn get_level_collision(current: &SelectedLevel, probe: &XrayProbe) -> Result<Arc<LevelCformTracer>, String> {
  current
    .collision
    .get_or_init(|| read_collision(&current.source, probe))
    .clone()
}

fn read_collision(source: &LevelSource, probe: &XrayProbe) -> Result<Arc<LevelCformTracer>, String> {
  let started: Instant = Instant::now();
  let geometry: LevelCformGeometry =
    LevelCformFile::read_geometry_from_bytes::<XRayByteOrder>(read_file(source, probe, COLLISION_FILE)?).map_err(
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
