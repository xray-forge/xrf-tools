use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use rayon::prelude::*;
use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelCformTracer, LevelLight, LevelLightsChunk, LevelLightsFile};
use xrf_math::Vector3d;
use xrf_vfs::XraySearch;
use xrf_visual::{HemiEstimator, VisualSphere, VisualTransform};

use crate::level_report::LevelReport;
use crate::parts::{LevelCollisionPart, LevelSpawnPart};
use crate::state::LevelSpawnCategory;
use crate::state::{LIGHTS_FILE, LevelSource, LevelSpawn, LevelSpawnObjectHemi, LevelSpawnVisual, SelectedLevel};

/// What a change event calls a level's hemisphere lighting when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_HEMI_PART: &str = "hemi";

/// How the level lights its spawned objects and the camera: `CROS_impl`'s estimate, over its collision form and its
/// compiled lights.
pub(crate) struct LevelHemiPart;

impl SessionPart for LevelHemiPart {
  type Value = Result<Arc<HemiEstimator>, String>;

  const NAME: &'static str = LEVEL_HEMI_PART;
}

impl LevelHemiPart {
  /// The open level's estimator, built the first time anything asks and kept.
  ///
  /// # Errors
  ///
  /// Returns why the collision form cannot be read, which every later ask answers with again.
  pub fn get(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Result<Arc<HemiEstimator>, String> {
    current.parts.get_or_record::<Self>(search, |search| {
      Self::read(current, search).inspect_err(|error| LevelReport::missing_hemi(&current.source, error))
    })
  }

  /// The cube of every spawned object the viewer draws standing as one of `spheres`' visuals, where it stands, in
  /// renderer space.
  pub fn estimate_spawn(
    estimator: &HemiEstimator,
    spawn: &LevelSpawn,
    spheres: &HashMap<&str, &VisualSphere>,
  ) -> Vec<LevelSpawnObjectHemi> {
    spawn
      .objects
      .par_iter()
      .enumerate()
      .filter_map(|(index, object)| {
        let (_, visual) = LevelSpawnCategory::get_drawn_visual(object)?;
        let sphere: &VisualSphere = spheres.get(visual)?;
        // The visual's sphere is in renderer space, and so is where the object stands it; the form is the engine's.
        let centre: Vector3d =
          VisualTransform::of_spawn(&object.position, &object.direction).apply_to_point(&sphere.center);
        let estimate = estimator.estimate(&Vector3d::new(centre.x, centre.y, -centre.z), sphere.radius);

        Some(LevelSpawnObjectHemi {
          cube: estimate.cube.to_renderer_space().faces,
          sky: estimate.sky,
          index: index as u32,
        })
      })
      .collect()
  }

  /// How the level lights each object standing as one of the visuals read, none where its spawn cannot be read.
  pub fn estimate_visuals(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    estimator: &HemiEstimator,
    read: &[(&str, Arc<LevelSpawnVisual>)],
  ) -> Vec<LevelSpawnObjectHemi> {
    let Ok(spawn) = LevelSpawnPart::get(current, search) else {
      return Vec::new();
    };
    let spheres: HashMap<&str, &VisualSphere> = read
      .iter()
      .map(|(name, visual)| (*name, &visual.package.description.declared_bounds.bounding_sphere))
      .collect();

    Self::estimate_spawn(estimator, &spawn, &spheres)
  }

  fn read(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Result<Arc<HemiEstimator>, String> {
    let started: Instant = Instant::now();
    let tracer: Arc<LevelCformTracer> = LevelCollisionPart::get(current, search)?;
    let estimator: HemiEstimator = HemiEstimator::new(tracer, &Self::read_lights(&current.source, search));

    LevelReport::hemi(&current.source, estimator.get_light_count(), started);

    Ok(Arc::new(estimator))
  }

  /// The compiled lights `CLight_DB::LoadHemi` reads, none for a level without `build.lights` or with one unreadable.
  fn read_lights(source: &LevelSource, search: &XraySearch) -> Vec<LevelLight> {
    let file: Result<Option<LevelLightsFile>, String> =
      source.read_optional_file(search, LIGHTS_FILE).and_then(|bytes| {
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
        LevelReport::unreadable_lights(source, &error);

        Vec::new()
      }
    }
  }
}
