//! How the level lights its spawned objects: `CROS_impl`'s estimate, from its collision form and compiled lights.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelCformFile, LevelCformGeometry, LevelCformTracer, LevelLight, LevelLightsChunk, LevelLightsFile};
use xrf_math::Vector3d;
use xrf_vfs::XrayProbe;
use xrf_visual::{HemiEstimator, VisualSphere, VisualTransform};

use crate::plugins::levels::read::{read_file, read_optional_file};
use crate::plugins::levels::report::{report_hemi, report_missing_hemi, report_unreadable_lights};
use crate::plugins::levels::spawn_objects::get_drawn_visual;
use crate::plugins::levels::state::{
  COLLISION_FILE, LIGHTS_FILE, LevelSource, LevelSpawn, LevelSpawnObjectHemi, SelectedLevel,
};

/// The open level's estimator, built the first time a batch asks and held until every visual is described.
///
/// # Errors
///
/// Returns why the collision form cannot be read, which every later ask while it is held answers with again.
pub fn get_level_hemi(current: &SelectedLevel, probe: &XrayProbe) -> Result<Arc<HemiEstimator>, String> {
  current
    .spawn_lighting
    .get_or_build(|| read_hemi(&current.source, probe).inspect_err(|error| report_missing_hemi(&current.source, error)))
}

/// The cube of every spawned object the viewer draws standing as one of `spheres`' visuals, where it stands, in
/// renderer space.
pub fn estimate_spawn_hemi(
  estimator: &HemiEstimator,
  spawn: &LevelSpawn,
  spheres: &HashMap<&str, &VisualSphere>,
) -> Vec<LevelSpawnObjectHemi> {
  spawn
    .objects
    .par_iter()
    .enumerate()
    .filter_map(|(index, object)| {
      let (_, visual) = get_drawn_visual(object)?;
      let sphere: &VisualSphere = spheres.get(visual)?;
      // The visual's sphere is in renderer space, and so is where the object stands it; the form is the engine's.
      let centre: Vector3d =
        VisualTransform::of_spawn(&object.position, &object.direction).apply_to_point(&sphere.center);
      let cube = estimator.estimate(&Vector3d::new(centre.x, centre.y, -centre.z), sphere.radius);

      Some(LevelSpawnObjectHemi {
        cube: cube.to_renderer_space().faces,
        index: index as u32,
      })
    })
    .collect()
}

fn read_hemi(source: &LevelSource, probe: &XrayProbe) -> Result<Arc<HemiEstimator>, String> {
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
  let lights: Vec<LevelLight> = read_hemi_lights(source, probe);

  report_hemi(source, tracer.get_triangle_count(), lights.len(), read, started);

  Ok(Arc::new(HemiEstimator::new(tracer, &lights)))
}

/// The compiled lights `CLight_DB::LoadHemi` reads, none for a level without `build.lights` or with one unreadable.
fn read_hemi_lights(source: &LevelSource, probe: &XrayProbe) -> Vec<LevelLight> {
  let file: Result<Option<LevelLightsFile>, String> =
    read_optional_file(source, probe, LIGHTS_FILE).and_then(|bytes| {
      bytes
        .map(|bytes| LevelLightsFile::read_from_bytes::<XRayByteOrder>(bytes).map_err(|error| error.to_string()))
        .transpose()
    });

  match file {
    Ok(file) => file
      .iter()
      .flat_map(|file| &file.chunks)
      .filter(|chunk| chunk.get_id() == LevelLightsChunk::HEMI_CHUNK_ID)
      .flat_map(LevelLightsChunk::get_lights)
      .cloned()
      .collect(),
    Err(error) => {
      report_unreadable_lights(source, &error);

      Vec::new()
    }
  }
}
