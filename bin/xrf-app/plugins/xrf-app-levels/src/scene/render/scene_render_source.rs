use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_render::{ResolvedTexture, TextureResolver};
use xrf_chunk::XRayByteOrder;
use xrf_editable_object::{EditableObjectFile, EditableOccluder};
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
use crate::scene::{
  SceneDrawnObject, SceneDrawnObjects, SceneHelpersPart, SceneOccluderLook, SceneRenderId, SceneSpawnModels,
  SceneSpawnModelsPart, SceneSpawnVisualsPart, SelectedScene,
};
use crate::visuals::SpawnVisual;

/// An open scene as the world plays it and the native renderer draws it: no sectors and no level, every scene object
/// placing a library object found in its objects roots stood as a model compiled from that `.object`, an occluder's
/// faces drawn as its fill ([`SceneOccluderLook`]), every spawn point whose packet names a visual found in the scene's
/// game data stood as that model in its rest pose, each lit by the open sky as a model alone is, and their textures
/// read from the scene's game data; what has no geometry drawn as its helper. Each drawn thing is named by its object's
/// [`SceneRenderId`].
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
  /// The category every scene object stands in, so the page hides any of them one by one, never by group.
  const CATEGORY: RenderSpawnCategory = RenderSpawnCategory::Props;

  /// The category every spawn point's model stands in, apart from the scene objects', so the page switches them all
  /// with the spawn points' helpers by its group alone: the renderer's items group, which nothing else of a scene
  /// stands in, whatever the entity's class.
  const SPAWN_CATEGORY: RenderSpawnCategory = RenderSpawnCategory::Items;

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

  /// A search over the scene's game data, which its textures, shaders and spawn points' visuals are read through.
  fn search_game(&self) -> XrfResult<XraySearch> {
    self
      .assets
      .search(&self.scene.game.roots)
      .map_err(XrfError::new_asset_error)
  }

  /// The spawn points drawn as models, looked for the first time anything asks.
  ///
  /// # Errors
  ///
  /// Returns an error when the scene's game data cannot be searched the first time.
  fn get_spawn_models(&self) -> XrfResult<Arc<SceneSpawnModels>> {
    Ok(SceneSpawnModelsPart::get(&self.scene, &self.search_game()?))
  }

  /// How a model standing as `object` is lit: by the open sky on every side, no level standing around a scene's
  /// objects to shade their hemisphere.
  fn light_open(object: SceneRenderId) -> RenderSpawnLighting {
    RenderSpawnLighting {
      object: object.0,
      cube: [1.0; 6],
      sky: 1.0,
    }
  }

  /// The model the spawn points standing as `visual` draw as, posed as they stand still, its textures known to the
  /// resolver as a level's spawned models' are.
  fn pose_visual(&self, name: &str, visual: &SpawnVisual) -> RenderSpawnModel {
    self.textures.extend(visual.textures.iter().map(ResolvedTexture::from));

    RenderSpawnModel {
      name: name.to_owned(),
      package: VisualPoser::pose(&visual.package, visual.rest.as_deref()),
      surfaces: visual.surfaces.clone(),
      skeleton: None,
    }
  }

  /// The model `object`'s library object compiles to, posed as stored, each of its submeshes dressed as `surfaces`
  /// answers it, or as the occluder's fill where the library object is flagged an occluder, as the editor draws it
  /// whatever its surfaces.
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
    let occluder: Option<EditableOccluder> = file.get_occluder();
    // A scene's object compiles with the CLI's default tangent method, which only a skeleton's compile reads.
    let mut visual: ObjectVisual = ObjectVisual::compile(
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

    let surfaces: Vec<XraySurfaceDescriptor> = match occluder {
      Some(occluder) => SceneOccluderLook::dress(occluder, &mut visual.package.description),
      None => VisualSurfaces::describe(surfaces, &visual.package.description),
    };

    Ok(RenderSpawnModel {
      name: object.reference.clone(),
      package: VisualPoser::pose(&visual.package, None),
      surfaces,
      skeleton: None,
    })
  }
}

impl RenderAssetSource for SceneRenderSource {
  fn get_texture_scope(&self) -> &str {
    self.textures.get_scope_key()
  }

  /// An occluder's fill made here, any other texture read from the scene's game data.
  fn read_texture(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    match SceneOccluderLook::read_texture(reference)? {
      Some(fill) => Ok(Some(fill)),
      None => self.textures.read(&self.assets, reference),
    }
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

  /// Every scene object drawn in the props group, then every spawn point drawn as a model in a group of their own, each
  /// named by its [`SceneRenderId`], so a pick answers it as the page names it; each library object a visual named by
  /// its reference and each spawn point's model one named by its file, in the order first placed.
  fn read_spawn(&self) -> XrfResult<RenderLevelSpawn> {
    let drawn: &SceneDrawnObjects = self.get_drawn()?;
    let models: Arc<SceneSpawnModels> = self.get_spawn_models()?;
    let mut visuals: Vec<String> = Vec::new();
    let mut indices: HashMap<String, u32> = HashMap::new();
    let mut stand = |name: String| -> u32 {
      *indices.entry(name).or_insert_with_key(|name| {
        visuals.push(name.clone());

        (visuals.len() - 1) as u32
      })
    };
    let mut objects: Vec<RenderSpawnObject> = Vec::with_capacity(drawn.objects.len() + models.objects.len());

    for object in &drawn.objects {
      if let Some(id) = SceneRenderId::of(&object.id) {
        objects.push(RenderSpawnObject {
          index: id.0,
          category: Self::CATEGORY,
          is_released: false,
          visual: stand(object.reference.clone()),
          transform: object.get_render_transform(),
        });
      }
    }

    for model in &models.objects {
      if let Some(id) = SceneRenderId::of(&model.id) {
        objects.push(RenderSpawnObject {
          index: id.0,
          category: Self::SPAWN_CATEGORY,
          is_released: false,
          visual: stand(model.get_model_name()),
          transform: model.get_render_transform(),
        });
      }
    }

    Ok(RenderLevelSpawn {
      visuals,
      objects,
      detail: 0.0,
    })
  }

  /// Each model named: a library object compiled once whatever stands as it, a spawn point's visual read once through
  /// the visuals the scene keeps; one that does not read or compile is left out and reported.
  fn read_spawn_models(&self, names: &[String]) -> XrfResult<RenderSpawnModels> {
    let drawn: &SceneDrawnObjects = self.get_drawn()?;
    let objects: XraySearch = self.search_objects()?;
    let game: XraySearch = self.search_game()?;
    let spawn_models: Arc<SceneSpawnModels> = SceneSpawnModelsPart::get(&self.scene, &game);
    let surfaces: XraySurfaceResolver = XraySurfaceResolver::open(&game, XrayTextureScope::shared());
    let mut models: Vec<RenderSpawnModel> = Vec::with_capacity(names.len());
    let mut failures: Vec<RenderLoadFailure> = Vec::new();

    SceneSpawnVisualsPart::with_reader(&self.scene, &game, |reader| {
      for name in names {
        let read: TauriResult<RenderSpawnModel> = match (drawn.find(name), spawn_models.find_model(name)) {
          (Some(object), _) => self
            .read_model(&objects, object, &surfaces)
            .inspect_err(|reason| LevelReport::undrawn_scene_object(name, reason)),
          // The reader reports a visual that does not read.
          (None, Some(model)) => reader.get(&model.visual).map(|visual| self.pose_visual(name, &visual)),
          (None, None) => Err(format!("No scene object or spawn point stands as model '{name}'"))
            .inspect_err(|reason| LevelReport::undrawn_scene_object(name, reason)),
        };

        match read {
          Ok(model) => models.push(model),
          Err(reason) => failures.push(RenderLoadFailure {
            name: name.clone(),
            reason,
          }),
        }
      }
    });

    let is_read = |name: &str| models.iter().any(|model| model.name == name);
    let lighting: Vec<RenderSpawnLighting> = drawn
      .objects
      .iter()
      .filter(|object| is_read(&object.reference))
      .filter_map(|object| SceneRenderId::of(&object.id))
      .chain(
        spawn_models
          .objects
          .iter()
          .filter(|model| is_read(&model.get_model_name()))
          .filter_map(|model| SceneRenderId::of(&model.id)),
      )
      .map(Self::light_open)
      .collect();

    Ok(RenderSpawnModels {
      models,
      lighting,
      failures,
    })
  }

  /// The helpers of what the scene holds with no geometry, each named by its scene object's [`SceneRenderId`] and
  /// switched by its tool, a spawn point standing as its model helped by its volume alone.
  fn read_helpers(&self) -> XrfResult<RenderLevelHelpers> {
    Ok(SceneHelpersPart::get(&self.scene, &self.search_game()?).helpers.clone())
  }

  fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>> {
    Ok(None)
  }

  fn read_particles(&self) -> XrfResult<Option<RenderLevelParticles>> {
    Ok(None)
  }
}
