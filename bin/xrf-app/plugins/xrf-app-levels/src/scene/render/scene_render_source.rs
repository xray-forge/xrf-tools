use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_render::TextureResolver;
use xrf_chunk::XRayByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_environment::WeatherDescriptor;
use xrf_error::{XrfError, XrfResult};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};
use xrf_object_visual::{ObjectVisual, VisualSurfaces};
use xrf_rawdata::{ObjectCompileOptions, ObjectCompileSource};
use xrf_renderer::{
  RenderAssetSource, RenderLevelDetails, RenderLevelHelpers, RenderLevelParticles, RenderLevelSource, RenderLevelSpawn,
  RenderLoadFailure, RenderSpawnCategory, RenderSpawnLighting, RenderSpawnModel, RenderSpawnModels, RenderSpawnObject,
};
use xrf_vfs::XraySearch;
use xrf_visual::{LightsDescription, SectorPackage, VisualPoser};
use xrf_world::{WorldLevelSource, WorldLevelWeather};

use crate::report::LevelReport;
use crate::scene::{SceneDrawnObject, SceneDrawnObjects, SceneHelpersPart, SelectedScene};

/// An open scene as the world plays it and the native renderer draws it: no sectors and no level, every scene object
/// placing a library object found in its objects roots stood as a model compiled from that `.object`, each lit by the
/// open sky as a model alone is, and their textures read from the scene's game data; what has no geometry drawn as
/// its helper.
pub struct SceneRenderSource {
  scene: Arc<SessionSnapshot<SelectedScene>>,
  assets: AssetMountState,
  /// Where each texture reference resolved once asked for, in the scene's game data and the shared scope; nothing
  /// resolves for a scene with no game data, whose surfaces draw the renderer's placeholder.
  textures: TextureResolver,
  /// The scene objects drawn, read the first time the renderer asks for its spawn.
  drawn: OnceLock<SceneDrawnObjects>,
}

impl SceneRenderSource {
  /// The one category every scene object stands in, so the page hides any of them one by one, never by group.
  const CATEGORY: RenderSpawnCategory = RenderSpawnCategory::Props;

  /// The scene `scene` holds as the renderer draws it.
  pub fn new(scene: Arc<SessionSnapshot<SelectedScene>>, assets: AssetMountState) -> Self {
    let textures: TextureResolver = TextureResolver::new(scene.game.roots.clone(), XrayTextureScope::shared(), []);

    Self {
      scene,
      assets,
      textures,
      drawn: OnceLock::new(),
    }
  }

  /// What every source of a scene session shares its scene by, whichever of its snapshots it draws.
  pub fn format_scene_key(session_id: SessionId) -> String {
    format!("scene {session_id}")
  }

  /// The snapshot it draws.
  pub fn get_scene(&self) -> &Arc<SessionSnapshot<SelectedScene>> {
    &self.scene
  }

  /// The scene objects drawn, read once.
  ///
  /// # Errors
  ///
  /// Returns an error when the scene's objects roots cannot be searched the first time.
  fn get_drawn(&self) -> XrfResult<&SceneDrawnObjects> {
    if let Some(drawn) = self.drawn.get() {
      return Ok(drawn);
    }

    let objects: XraySearch = self.search_objects()?;

    Ok(
      self
        .drawn
        .get_or_init(|| SceneDrawnObjects::read(&self.scene, &objects)),
    )
  }

  /// A search over the scene's objects roots, which its library objects are read through.
  fn search_objects(&self) -> XrfResult<XraySearch> {
    self
      .assets
      .search(&self.scene.objects_roots)
      .map_err(XrfError::new_asset_error)
  }

  /// The model `object`'s library object compiles to, posed as stored, and how each of its submeshes draws as
  /// `surfaces` answers it.
  ///
  /// # Errors
  ///
  /// Returns why the library object does not read, or does not compile.
  fn read_model(
    &self,
    objects: &XraySearch,
    object: &SceneDrawnObject,
    surfaces: &XraySurfaceResolver,
  ) -> TauriResult<RenderSpawnModel> {
    let file: EditableObjectFile = objects
      .read(&object.logical_path)
      .and_then(EditableObjectFile::read_from_bytes::<XRayByteOrder>)
      .map_err(|error| format!("Library object '{}' does not read: {error}", object.logical_path))?;
    // A scene's object compiles with the CLI's default tangent method, which only a skeleton's compile reads.
    let visual: ObjectVisual = ObjectVisual::compile(
      &file,
      &ObjectCompileSource {
        source_file: object.logical_path.clone(),
        convertor: String::from("xrf-app levels"),
        built_at: 0,
      },
      &ObjectCompileOptions::default(),
    )
    .map_err(|error| format!("Library object '{}' does not compile: {error}", object.logical_path))?;

    // Let go before posing copies the package, which keeps a large shell's peak to the model and its copy.
    drop(file);

    Ok(RenderSpawnModel {
      name: object.reference.clone(),
      package: VisualPoser::pose(&visual.package, None),
      surfaces: VisualSurfaces::describe(surfaces, &visual.package.description),
      skeleton: None,
    })
  }
}

impl RenderAssetSource for SceneRenderSource {
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

impl RenderLevelSource for SceneRenderSource {
  fn get_sector_count(&self) -> u32 {
    0
  }

  fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
    &[]
  }
}

impl WorldLevelSource for SceneRenderSource {
  /// The scene's session: every viewport showing one opening shares its scene.
  fn get_scene_key(&self) -> Option<String> {
    Some(Self::format_scene_key(self.scene.session_id))
  }

  fn pack_sector(&self, sector: u32) -> XrfResult<SectorPackage> {
    Err(XrfError::new_not_found_error(format!("A scene has no sector {sector}")))
  }

  fn read_lights(&self) -> XrfResult<LightsDescription> {
    Ok(LightsDescription::default())
  }

  fn read_weather(&self) -> XrfResult<WorldLevelWeather> {
    Ok(WorldLevelWeather::default())
  }

  fn read_weather_cycle(&self, name: &str) -> XrfResult<Vec<WeatherDescriptor>> {
    Err(XrfError::new_not_found_error(format!(
      "A scene plays no weather '{name}'"
    )))
  }

  /// Every scene object drawn, named by its place among the scene's objects, so a pick answers it as the page lists
  /// it; each library object a visual, named by its reference, in the order first placed.
  fn read_spawn(&self) -> XrfResult<RenderLevelSpawn> {
    let drawn: &SceneDrawnObjects = self.get_drawn()?;
    let mut visuals: Vec<String> = Vec::new();
    let mut indices: HashMap<&str, u32> = HashMap::new();
    let objects: Vec<RenderSpawnObject> = drawn
      .objects
      .iter()
      .map(|object| RenderSpawnObject {
        index: object.id.index,
        category: Self::CATEGORY,
        is_released: false,
        visual: *indices.entry(object.reference.as_str()).or_insert_with(|| {
          visuals.push(object.reference.clone());

          (visuals.len() - 1) as u32
        }),
        transform: object.get_render_transform(),
      })
      .collect();

    Ok(RenderLevelSpawn {
      visuals,
      objects,
      detail: 0.0,
    })
  }

  /// Each library object named, compiled once whatever stands as it; one that does not read or compile is left out
  /// and reported.
  fn read_spawn_models(&self, names: &[String]) -> XrfResult<RenderSpawnModels> {
    let drawn: &SceneDrawnObjects = self.get_drawn()?;
    let objects: XraySearch = self.search_objects()?;
    let surfaces: XraySurfaceResolver = XraySurfaceResolver::open(
      &self
        .assets
        .search(&self.scene.game.roots)
        .map_err(XrfError::new_asset_error)?,
      XrayTextureScope::shared(),
    );
    let mut models: Vec<RenderSpawnModel> = Vec::with_capacity(names.len());
    let mut failures: Vec<RenderLoadFailure> = Vec::new();

    for name in names {
      let read: TauriResult<RenderSpawnModel> = match drawn.find(name) {
        Some(object) => self.read_model(&objects, object, &surfaces),
        None => Err(format!("No scene object places library object '{name}'")),
      };

      match read {
        Ok(model) => models.push(model),
        Err(reason) => {
          LevelReport::undrawn_scene_object(name, &reason);
          failures.push(RenderLoadFailure {
            name: name.clone(),
            reason,
          });
        }
      }
    }

    let lighting: Vec<RenderSpawnLighting> = drawn
      .objects
      .iter()
      .filter(|object| models.iter().any(|model| model.name == object.reference))
      .map(|object| RenderSpawnLighting {
        object: object.id.index,
        // Open sky on every side: no level stands around a scene's objects to shade their hemisphere.
        cube: [1.0; 6],
        sky: 1.0,
      })
      .collect();

    Ok(RenderSpawnModels {
      models,
      lighting,
      failures,
    })
  }

  /// The helpers of what the scene holds with no geometry, each named by its scene object's [`SceneHelperId`] and
  /// switched by its tool.
  ///
  /// [`SceneHelperId`]: crate::scene::SceneHelperId
  fn read_helpers(&self) -> XrfResult<RenderLevelHelpers> {
    Ok(SceneHelpersPart::get(&self.scene).helpers.clone())
  }

  fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>> {
    Ok(None)
  }

  fn read_particles(&self) -> XrfResult<Option<RenderLevelParticles>> {
    Ok(None)
  }
}
