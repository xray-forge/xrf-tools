use std::collections::HashMap;
use std::sync::{Arc, PoisonError, RwLock};

use xrf_environment::WeatherDescriptor;
use xrf_error::{XrfError, XrfResult};
use xrf_material::{XraySurfaceDescriptor, XrayTextureScope};
use xrf_renderer::{
  RenderAssetSource, RenderLevelDetails, RenderLevelSource, RenderLevelSpawn, RenderLevelWeather, RenderModelSkeleton,
  RenderMotion, RenderSpawnCategory, RenderSpawnLighting, RenderSpawnModel, RenderSpawnModels, RenderSpawnObject,
};
use xrf_visual::{
  FLOATS_PER_BONE, LightsDescription, SectorPackage, VisualDescription, VisualMotionPose, VisualPackage, VisualPoser,
  VisualTransform,
};

use crate::core::assets::{AssetMountState, read_located_asset};
use crate::core::session::SessionSnapshot;
use crate::plugins::levels::textures::resolve_reference;
use crate::plugins::visuals::pose::bake_named_motion;
use crate::plugins::visuals::state::SelectedVisual;

/// The one object an asset scene stands: the open model at the origin.
const MODEL_OBJECT: u32 = 0;

/// Where it stands, sixteen floats column by column.
const IDENTITY: [f32; 16] = [
  1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

/// The open model as the native renderer draws it: a scene of one object and no level, its textures read from the
/// roots it was opened in, its skin kept for the renderer to pose, and its motions baked when a pose names one.
pub struct VisualRenderSource {
  visual: Arc<SessionSnapshot<SelectedVisual>>,
  assets: AssetMountState,
  /// How far down each submesh's collapse chain to draw: zero its finest level, one its coarsest.
  detail: f32,
  /// Where each texture reference resolved once asked for; `None` for one resolving to nothing.
  textures: RwLock<HashMap<String, Option<String>>>,
}

impl VisualRenderSource {
  pub fn new(visual: Arc<SessionSnapshot<SelectedVisual>>, assets: AssetMountState, detail: f32) -> Self {
    Self {
      visual,
      assets,
      detail,
      textures: RwLock::new(HashMap::new()),
    }
  }
}

impl RenderAssetSource for VisualRenderSource {
  fn get_texture_scope(&self) -> String {
    format!("{:?} {:?}", self.visual.roots, XrayTextureScope::shared())
  }

  fn read_texture(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    let known: Option<Option<String>> = self
      .textures
      .read()
      .unwrap_or_else(PoisonError::into_inner)
      .get(reference)
      .cloned();
    let located: Option<String> = match known {
      Some(located) => located,
      None => {
        let located: Option<String> = self
          .assets
          .with_probe(&self.visual.roots, |probe| {
            resolve_reference(probe, &XrayTextureScope::shared(), reference)
          })
          .map_err(XrfError::new_asset_error)?;

        self
          .textures
          .write()
          .unwrap_or_else(PoisonError::into_inner)
          .insert(reference.to_owned(), located.clone());

        located
      }
    };
    let Some(logical_path) = located else {
      return Ok(None);
    };

    self
      .assets
      .with_probe(&self.visual.roots, |probe| read_located_asset(probe, &logical_path))
      .map_err(XrfError::new_asset_error)?
      .map(Some)
  }
}

impl RenderLevelSource for VisualRenderSource {
  fn get_sector_count(&self) -> u32 {
    0
  }

  fn pack_sector(&self, sector: u32) -> XrfResult<SectorPackage> {
    Err(XrfError::new_not_found_error(format!("A model has no sector {sector}")))
  }

  fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
    &[]
  }

  fn read_lights(&self) -> XrfResult<LightsDescription> {
    Ok(LightsDescription::default())
  }

  fn read_weather(&self) -> XrfResult<RenderLevelWeather> {
    Ok(RenderLevelWeather::default())
  }

  fn read_weather_cycle(&self, name: &str) -> XrfResult<Vec<WeatherDescriptor>> {
    Err(XrfError::new_not_found_error(format!(
      "A model plays no weather '{name}'"
    )))
  }

  fn read_spawn(&self) -> XrfResult<RenderLevelSpawn> {
    Ok(RenderLevelSpawn {
      visuals: vec![self.visual.source.label().to_owned()],
      objects: vec![RenderSpawnObject {
        index: MODEL_OBJECT,
        category: RenderSpawnCategory::Props,
        visual: 0,
        transform: IDENTITY,
      }],
      detail: self.detail,
    })
  }

  fn read_spawn_models(&self, names: &[String]) -> XrfResult<RenderSpawnModels> {
    let package: &VisualPackage = &self.visual.package;
    let skeleton: Option<RenderModelSkeleton> = to_skeleton(&package.description);

    Ok(RenderSpawnModels {
      models: names
        .iter()
        .map(|name| RenderSpawnModel {
          name: name.clone(),
          // Its skin kept where the renderer can pose it, and baked away as stored where it cannot.
          package: if skeleton.is_some() {
            VisualPackage {
              description: package.description.clone(),
              buffer: package.buffer.clone(),
            }
          } else {
            VisualPoser::pose(package, None)
          },
          surfaces: self.visual.surfaces.clone(),
          skeleton: skeleton.clone(),
        })
        .collect(),
      // Open sky on every side: no level stands around it to shade its hemisphere.
      lighting: vec![RenderSpawnLighting {
        object: MODEL_OBJECT,
        cube: [1.0; 6],
        sky: 1.0,
      }],
      failures: Vec::new(),
    })
  }

  fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>> {
    Ok(None)
  }

  fn read_motion(&self, name: &str) -> XrfResult<RenderMotion> {
    let Some(skeleton) = self.visual.skeleton.as_ref() else {
      return Err(XrfError::new_not_found_error(
        "The open visual carries no bind pose to animate",
      ));
    };
    // The bake `open_motion` just made of it, when that is the one asked for, rather than composing it again.
    let baked: Option<VisualMotionPose> = self
      .visual
      .posed
      .get()
      .ok()
      .flatten()
      .filter(|posed| posed.description.name == name)
      .map(|posed| VisualMotionPose::clone(&posed));
    let posed: VisualMotionPose = match baked {
      Some(posed) => posed,
      None => self
        .assets
        .with_probe(&self.visual.roots, |probe| {
          bake_named_motion(probe, skeleton, &self.visual.dependencies, name)
        })
        .map_err(XrfError::new_asset_error)?
        .map_err(XrfError::new_asset_error)?,
    };
    let floats: usize = posed.description.floats_per_bone as usize;

    // Twelve floats a bone, whatever the bake's stride: the basis' three columns, then the translation.
    let transforms: Vec<f32> = if floats == FLOATS_PER_BONE {
      posed.transforms
    } else {
      posed
        .transforms
        .chunks(floats.max(1))
        .flat_map(|bone| bone.iter().copied().chain(std::iter::repeat(0.0)).take(FLOATS_PER_BONE))
        .collect()
    };

    Ok(RenderMotion {
      frames: posed.description.frame_count,
      bones: posed.description.bone_count,
      transforms,
    })
  }
}

/// A model's skeleton as the renderer poses it, where every bone carries its bind and a submesh hangs from them; none
/// for one drawn as stored.
fn to_skeleton(description: &VisualDescription) -> Option<RenderModelSkeleton> {
  let is_skinned: bool = description
    .submeshes
    .iter()
    .any(|submesh| submesh.geometry().is_some_and(|geometry| geometry.skin.is_some()));
  let binds: Vec<&VisualTransform> = description
    .bones
    .iter()
    .map(|bone| bone.bind_transform.as_ref())
    .collect::<Option<_>>()?;

  (is_skinned && !binds.is_empty()).then(|| RenderModelSkeleton {
    binds: binds
      .iter()
      .flat_map(|bind| {
        [
          bind.i.x, bind.i.y, bind.i.z, bind.j.x, bind.j.y, bind.j.z, bind.k.x, bind.k.y, bind.k.z, bind.c.x, bind.c.y,
          bind.c.z,
        ]
      })
      .collect(),
    pairs: description
      .bones
      .iter()
      .enumerate()
      .filter_map(|(child, bone)| bone.parent_index.map(|parent| (child as u16, parent as u16)))
      .collect(),
  })
}
