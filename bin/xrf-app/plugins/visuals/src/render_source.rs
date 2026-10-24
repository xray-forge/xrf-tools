use std::sync::Arc;

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionSnapshot, Sessions};
use xrf_app_render::texture_resolver::TextureResolver;
use xrf_environment::WeatherDescriptor;
use xrf_error::{XrfError, XrfResult};
use xrf_material::{XraySurfaceDescriptor, XrayTextureScope};
use xrf_renderer::{
  RenderAssetSource, RenderLevelDetails, RenderLevelParticles, RenderLevelSource, RenderLevelSpawn, RenderLevelWeather,
  RenderModelSkeleton, RenderMotion, RenderSpawnCategory, RenderSpawnLighting, RenderSpawnModel, RenderSpawnModels,
  RenderSpawnObject,
};
use xrf_visual::{
  FLOATS_PER_BONE, LightsDescription, SectorPackage, VisualDescription, VisualMotionPose, VisualPackage, VisualPoser,
  VisualTransform,
};

use crate::posed_visual_motion::PosedVisualMotion;
use crate::state::SelectedVisual;

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
  /// The motions viewers posed their models by, the bake of one of this model's reused when a pose names it.
  poses: Sessions<PosedVisualMotion>,
  assets: AssetMountState,
  /// How far down each submesh's collapse chain to draw: zero its finest level, one its coarsest.
  detail: f32,
  /// Where each texture reference resolved once asked for, in the shared scope.
  textures: TextureResolver,
}

impl VisualRenderSource {
  pub fn new(
    visual: Arc<SessionSnapshot<SelectedVisual>>,
    poses: Sessions<PosedVisualMotion>,
    assets: AssetMountState,
    detail: f32,
  ) -> Self {
    let textures: TextureResolver = TextureResolver::new(visual.roots.clone(), XrayTextureScope::shared(), []);

    Self {
      visual,
      poses,
      assets,
      detail,
      textures,
    }
  }

  /// The bake a viewer posed this model by under `name`, which a pose naming it draws rather than composing it again.
  fn find_baked(&self, name: &str) -> Option<VisualMotionPose> {
    self
      .poses
      .list_opened()
      .ok()?
      .into_iter()
      .find(|posed| posed.model == self.visual.session_id && posed.pose.description.name == name)
      .map(|posed| posed.pose.clone())
  }
}

impl RenderAssetSource for VisualRenderSource {
  fn get_texture_scope(&self) -> &str {
    self.textures.get_scope_key()
  }

  fn read_texture(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    self.textures.read(&self.assets, reference)
  }

  fn forget_textures(&self, names: &[String]) {
    self.textures.forget(names);
  }

  fn get_texture_stamp(&self, reference: &str) -> Option<u64> {
    self.textures.get_stamp(&self.assets, reference)
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
        is_released: false,
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

  fn read_particles(&self) -> XrfResult<Option<RenderLevelParticles>> {
    Ok(None)
  }

  fn read_motion(&self, name: &str) -> XrfResult<RenderMotion> {
    let Some(motions) = self.visual.motions.as_ref() else {
      return Err(XrfError::new_not_found_error(
        "The open visual carries no bind pose to animate",
      ));
    };
    // The bake `open_motion` just made of it, when that is the one asked for, rather than composing it again.
    let posed: VisualMotionPose = match self.find_baked(name) {
      Some(posed) => posed,
      None => motions.bake_named(
        &self
          .assets
          .search(&self.visual.roots)
          .map_err(XrfError::new_asset_error)?,
        &self.visual.dependencies,
        name,
      )?,
    };
    // Twelve floats a bone, as the renderer reads them: the basis' three columns, then the translation.
    if posed.description.floats_per_bone as usize != FLOATS_PER_BONE {
      return Err(XrfError::new_invalid_error(format!(
        "Motion '{name}' is baked at {} floats a bone, not {FLOATS_PER_BONE}",
        posed.description.floats_per_bone
      )));
    }

    Ok(RenderMotion {
      frames: posed.description.frame_count,
      bones: posed.description.bone_count,
      transforms: posed.transforms,
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
