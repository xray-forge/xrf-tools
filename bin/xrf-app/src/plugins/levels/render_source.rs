use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use xrf_anm::AnmFile;
use xrf_chunk::{InMemoryChunkDataSource, XRayByteOrder};
use xrf_environment::{EnvironmentCatalog, WeatherDescriptor, WeatherModifier};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{EnvModifier, LevelGeomSource, LevelSector, LevelSectorComposition};
use xrf_ltx::Ltx;
use xrf_material::{XrayReducedTextures, XraySurfaceDescriptor};
use xrf_particles::{ParticleCollider, ParticleEngineRules};
use xrf_renderer::{
  RenderAssetSource, RenderLensFlare, RenderLevelDetails, RenderLevelParticles, RenderLevelSource, RenderLevelSpawn,
  RenderLevelWeather, RenderLoadFailure, RenderRain, RenderSpawnCategory, RenderSpawnLighting, RenderSpawnModel,
  RenderSpawnModels, RenderSpawnObject, RenderThunder, RenderThunderSettings, RenderThunderbolt,
  RenderThunderboltGradient, RenderThunderboltModel, RenderWeatherModel, RenderWetSurfaces,
};
use xrf_vfs::XraySearch;
use xrf_visual::{LightsDescription, SectorPackage, SectorPacker, VisualPoser};
use xrf_world::WorldLevelReload;

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionPart, SessionSnapshot};
use crate::plugins::levels::ambients::{to_render_ambient_effects, to_render_ambients};
use crate::plugins::levels::drawn_attributes::DRAWN_ATTRIBUTES;
use crate::plugins::levels::lens_flares::to_render_lens_flare;
use crate::plugins::levels::particle_collider::LevelParticleCollider;
use crate::plugins::levels::parts::{
  LevelCollisionPart, LevelDetailsPart, LevelEnvironmentPart, LevelGeometryPart, LevelHemiPart, LevelLightsPart,
  LevelObjectMotionsPart, LevelParticlesPart, LevelSectionsPart, LevelSpawnPart, LevelSpawnVisualsPart,
  LevelSurfacesPart, LevelTexturesPart, LevelWeatherPart, PackedLevelDetails, PackedLevelLights, PackedLevelParticles,
  SpawnVisualReader,
};
use crate::plugins::levels::report::{
  report_missing_particle_collision, report_missing_sections, report_packed_sector,
};
use crate::plugins::levels::spawn_objects::describe_spawn_objects;
use crate::plugins::levels::state::selection::level_rain::LevelRain;
use crate::plugins::levels::state::selection::level_thunderbolt_gradient::LevelThunderboltGradient;
use crate::plugins::levels::state::selection::level_thunderbolts::LevelThunderbolts;
use crate::plugins::levels::state::selection::level_weather_model::LevelWeatherModel;
use crate::plugins::levels::state::selection::level_wet_surfaces::LevelWetSurfaces;
use crate::plugins::levels::state::{
  LevelEnvironment, LevelSpawnCategory, LevelSpawnVisual, LevelTextureReference, SelectedLevel,
};
use crate::plugins::render::texture_resolver::TextureResolver;

/// The open level as the native renderer draws it: sectors packed when its loader asks, textures read from the roots
/// the level was opened in.
pub struct LevelRenderSource {
  level: Arc<SessionSnapshot<SelectedLevel>>,
  assets: AssetMountState,
  /// How the level's shader table is drawn, as its surfaces part holds it for the snapshot drawn.
  surfaces: Arc<Vec<XraySurfaceDescriptor>>,
  /// Where each texture reference the level's surfaces bind resolved to at open, each projector its lights name once
  /// they are read, and anything else once asked for, as the weather's skies are; shared by the session's next
  /// sources, which keep its resolutions, a texture's file come since being loaded afresh by the texture watcher.
  textures: Arc<TextureResolver>,
  /// The textures the game loads a step coarser at the same texture quality, out of the sections read as it opened.
  reduced: XrayReducedTextures,
}

impl LevelRenderSource {
  pub fn new(level: Arc<SessionSnapshot<SelectedLevel>>, assets: AssetMountState) -> Self {
    let (surfaces, textures): (Arc<Vec<XraySurfaceDescriptor>>, Arc<Vec<LevelTextureReference>>) =
      Self::find_surfaces(&level, &assets);
    let resolver: TextureResolver = TextureResolver::new(
      level.roots.clone(),
      level.source.get_texture_scope(),
      textures
        .iter()
        .map(|texture| (texture.reference.clone(), texture.logical_path.clone())),
    );

    Self {
      reduced: Self::find_reduced(&level).unwrap_or_default(),
      surfaces,
      level,
      assets,
      textures: Arc::new(resolver),
    }
  }

  /// The level's surfaces and their textures as its parts hold them, resolved where a commit let them go; none where
  /// its roots cannot be searched, which draws every surface flat.
  fn find_surfaces(
    level: &SessionSnapshot<SelectedLevel>,
    assets: &AssetMountState,
  ) -> (Arc<Vec<XraySurfaceDescriptor>>, Arc<Vec<LevelTextureReference>>) {
    if let (Some(surfaces), Some(textures)) = (LevelSurfacesPart::find(level), LevelTexturesPart::find(level)) {
      return (surfaces, textures);
    }

    match assets.search(&level.roots) {
      Ok(search) => (
        LevelSurfacesPart::get(level, &search),
        LevelTexturesPart::get(level, &search),
      ),
      Err(error) => {
        log::warn!(
          "The surfaces of level '{}' cannot be resolved: {error}",
          level.source.get_label()
        );

        (Arc::default(), Arc::default())
      }
    }
  }

  /// The textures the sections the level holds already reduce, so no texture load waits on the configs; none where
  /// they are not read, or could not be.
  fn find_reduced(level: &SessionSnapshot<SelectedLevel>) -> Option<XrayReducedTextures> {
    LevelSectionsPart::find(level)
      .as_ref()
      .and_then(|sections| sections.section(XrayReducedTextures::SECTION))
      .map(|section| XrayReducedTextures::new(section.iter().map(|(key, _)| key)))
  }

  /// The same level drawn from `next`, a later snapshot of its session sharing its value and its surfaces, with the
  /// textures resolved so far, and the reduced ones known until `next` holds sections of its own.
  pub fn follow(&self, next: Arc<SessionSnapshot<SelectedLevel>>) -> Self {
    Self {
      reduced: Self::find_reduced(&next).unwrap_or_else(|| self.reduced.clone()),
      surfaces: LevelSurfacesPart::find(&next).unwrap_or_else(|| Arc::clone(&self.surfaces)),
      level: next,
      assets: AssetMountState::clone(&self.assets),
      textures: Arc::clone(&self.textures),
    }
  }

  /// The snapshot it draws.
  pub fn get_level(&self) -> &Arc<SessionSnapshot<SelectedLevel>> {
    &self.level
  }

  /// What every source of a session's level shares its scene by, whichever of its snapshots it draws.
  pub(crate) fn to_scene_key(session_id: SessionId) -> String {
    format!("level {session_id}")
  }

  /// Whether a level whose next snapshot let `stale_parts` go streams afresh rather than reloading packages: its sectors
  /// are packed from its geometry and drawn with its surfaces, which no package reload replaces. Textures alone keep the
  /// scene, a file come or gone being loaded afresh by the texture watcher.
  pub fn is_streamed_afresh(stale_parts: &[&str]) -> bool {
    stale_parts
      .iter()
      .any(|part| [LevelGeometryPart::NAME, LevelSurfacesPart::NAME].contains(part))
  }

  /// What the renderer reads again of a level whose next snapshot let `stale_parts` go: each package whose read asks
  /// for any of them, or recorded what it read as one of them. The sectors stream afresh instead, on their geometry or
  /// surfaces (`is_streamed_afresh`); the graph feeds nothing drawn, the map and the textures' list are the page's, and
  /// the start only matters where a viewer first stands.
  pub fn to_reload(stale_parts: &[&str]) -> WorldLevelReload {
    let is_stale = |parts: &[&str]| stale_parts.iter().any(|part| parts.contains(part));

    WorldLevelReload {
      is_spawn: is_stale(&[
        LevelSpawnPart::NAME,
        LevelSpawnVisualsPart::NAME,
        LevelCollisionPart::NAME,
        LevelHemiPart::NAME,
      ]),
      is_lights: is_stale(&[
        LevelSpawnPart::NAME,
        LevelSpawnVisualsPart::NAME,
        LevelSectionsPart::NAME,
        LevelLightsPart::NAME,
      ]),
      is_particles: is_stale(&[
        LevelSpawnPart::NAME,
        LevelSectionsPart::NAME,
        LevelEnvironmentPart::NAME,
        LevelCollisionPart::NAME,
        LevelHemiPart::NAME,
        LevelParticlesPart::NAME,
      ]),
      is_weather: is_stale(&[LevelEnvironmentPart::NAME, LevelWeatherPart::NAME]),
      is_motions: is_stale(&[LevelObjectMotionsPart::NAME]),
      is_grass: is_stale(&[LevelDetailsPart::NAME]),
    }
  }

  /// A search over the roots the level opened with, which every read the renderer asks for goes through.
  fn search(&self) -> XrfResult<XraySearch> {
    self.assets.search(&self.level.roots).map_err(XrfError::new_asset_error)
  }

  /// The sections a level's spawned objects name, none where its spawn or the configs cannot be read.
  fn read_sections(level: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Option<Arc<Ltx>> {
    LevelSectionsPart::get(level, search)
      .inspect_err(report_missing_sections)
      .ok()
  }

  /// The group a spawn category's objects are shown in.
  fn to_render_category(category: LevelSpawnCategory) -> RenderSpawnCategory {
    match category {
      LevelSpawnCategory::Props => RenderSpawnCategory::Props,
      LevelSpawnCategory::Items => RenderSpawnCategory::Items,
      LevelSpawnCategory::Weapons => RenderSpawnCategory::Weapons,
      LevelSpawnCategory::Lamps => RenderSpawnCategory::Lamps,
    }
  }

  /// What the level's weathers play with as the renderer takes them, the environment's catalog read already and the
  /// rest read through `search`.
  fn to_render_weather(
    level: &SessionSnapshot<SelectedLevel>,
    environment: &LevelEnvironment,
    search: &XraySearch,
  ) -> RenderLevelWeather {
    let source = environment.get_source(level, search);
    let rain: LevelRain = LevelRain::read(&source);
    let wet: LevelWetSurfaces = LevelWetSurfaces::read(&source);
    let thunder: LevelThunderbolts = LevelThunderbolts::read(&source);
    let modifiers = LevelEnvironment::read_modifiers(level, search);

    RenderLevelWeather {
      engine: environment.catalog.engine,
      effects: environment
        .catalog
        .effects
        .iter()
        .map(|effect| (effect.name.clone(), effect.list_descriptors(environment.catalog.engine)))
        .collect(),
      modifiers: modifiers.iter().map(Self::to_weather_modifier).collect(),
      sun_table: environment
        .catalog
        .sun_table
        .as_ref()
        .map(|table| table.list_positions()),
      rain: Some(RenderRain {
        streak: rain.streak.reference,
        drop: rain.drop.map(Self::to_weather_model),
      }),
      wet: Some(RenderWetSurfaces {
        splash: wet.splash.reference,
        flow: wet.flow.reference,
      }),
      thunder: Some(Self::to_thunder(thunder)),
      suns: Self::to_suns(&environment.catalog),
      ambients: to_render_ambients(&environment.catalog, &level.source.get_name().unwrap_or_default()),
      ambient_effects: to_render_ambient_effects(&environment.catalog),
    }
  }

  /// A `level.env_mod` volume as the mixer weighs it: one from a file before the flag word adds every value, as the
  /// engine's `use_flags.one()` before the read leaves it.
  fn to_weather_modifier(modifier: &EnvModifier) -> WeatherModifier {
    let to_array = |vector: &xrf_math::Vector3d<f32>| [vector.x, vector.y, vector.z];

    WeatherModifier {
      position: to_array(&modifier.position),
      radius: modifier.radius,
      power: modifier.power,
      far_plane: modifier.far_plane,
      fog_color: to_array(&modifier.fog_color),
      fog_density: modifier.fog_density,
      ambient: to_array(&modifier.ambient),
      sky_color: to_array(&modifier.sky_color),
      hemi_color: to_array(&modifier.hemi_color),
      flags: modifier.use_flags.unwrap_or(WeatherModifier::ALL),
    }
  }

  /// A model the weather draws as the renderer takes it, its texture by reference.
  fn to_weather_model(model: LevelWeatherModel) -> RenderWeatherModel {
    RenderWeatherModel {
      texture: model.texture.reference,
      positions: model.positions,
      uvs: model.uvs,
      indices: model.indices,
    }
  }

  /// Every lens flare of the game as the renderer draws it, by section, each read as its engine reads it.
  fn to_suns(catalog: &EnvironmentCatalog) -> HashMap<String, RenderLensFlare> {
    catalog
      .suns
      .iter()
      .map(|sun| (sun.name.clone(), to_render_lens_flare(sun, catalog.engine)))
      .collect()
  }

  /// What the level's weather strikes with as the renderer takes it, every texture by reference.
  fn to_thunder(thunder: LevelThunderbolts) -> RenderThunder {
    let gradient = |gradient: LevelThunderboltGradient| RenderThunderboltGradient {
      opacity: gradient.opacity,
      radius: gradient.radius,
      texture: gradient.texture.reference,
      draw: gradient.draw,
    };

    RenderThunder {
      settings: thunder.settings.map(|settings| RenderThunderSettings {
        altitude: settings.altitude,
        delta_longitude: settings.delta_longitude,
        min_distance: settings.min_distance,
        tilt: settings.tilt,
        second_probability: settings.second_probability,
        sky_color: settings.sky_color,
        sun_color: settings.sun_color,
        fog_color: settings.fog_color,
      }),
      collections: thunder
        .collections
        .into_iter()
        .map(|collection| (collection.name, collection.thunderbolts))
        .collect(),
      bolts: thunder
        .bolts
        .into_iter()
        .map(|bolt| {
          (
            bolt.name,
            RenderThunderbolt {
              model: bolt.model.map(|index| index as usize),
              color: bolt.color.map(|index| index as usize),
              top: gradient(bolt.top),
              center: gradient(bolt.center),
            },
          )
        })
        .collect(),
      models: thunder
        .models
        .into_iter()
        .map(|model| RenderThunderboltModel {
          mesh: Self::to_weather_model(model.mesh),
          draw: model.draw,
        })
        .collect(),
      animators: thunder.animators,
    }
  }
}

impl RenderAssetSource for LevelRenderSource {
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

  fn is_texture_reduced(&self, reference: &str) -> bool {
    self.reduced.is_reduced(reference)
  }
}

impl RenderLevelSource for LevelRenderSource {
  fn get_sector_count(&self) -> u32 {
    self.level.get_sectors().len() as u32
  }

  /// The level's session: every source of one opening shares a scene, its next snapshots' included, and a level opened
  /// again streams afresh.
  fn get_scene_key(&self) -> Option<String> {
    Some(Self::to_scene_key(self.level.session_id))
  }

  fn pack_sector(&self, sector: u32) -> XrfResult<SectorPackage> {
    let sectors: &[LevelSector] = self.level.get_sectors();
    let root: u32 = sectors
      .get(sector as usize)
      .ok_or_else(|| XrfError::new_not_found_error(format!("The level has no sector {sector}")))?
      .root;
    let composition: LevelSectorComposition = LevelSectorComposition::of(&self.level.visuals, root);
    let geometry: Arc<LevelGeomSource<InMemoryChunkDataSource>> =
      LevelGeometryPart::get(&self.level, &self.search()?).map_err(XrfError::new_read_error)?;
    let started: Instant = Instant::now();
    let package: SectorPackage = SectorPacker::new(&self.level.visuals, self.level.level.shaders.as_ref(), &geometry)
      .pack::<XRayByteOrder>(sector, &composition, DRAWN_ATTRIBUTES);

    report_packed_sector(&package, started);

    Ok(package)
  }

  fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
    &self.surfaces
  }

  fn read_lights(&self) -> XrfResult<LightsDescription> {
    let level: &SessionSnapshot<SelectedLevel> = &self.level;
    let search: XraySearch = self.search()?;
    let packed: PackedLevelLights =
      LevelLightsPart::pack(level, &search, Self::read_sections(level, &search).as_deref());

    self.textures.extend(
      packed
        .projectors
        .into_iter()
        .map(|projector| (projector.reference, projector.logical_path)),
    );

    Ok(packed.lights)
  }

  fn read_weather(&self) -> XrfResult<RenderLevelWeather> {
    let level: &SessionSnapshot<SelectedLevel> = &self.level;
    let search: XraySearch = self.search()?;
    let environment: Arc<LevelEnvironment> =
      LevelEnvironmentPart::get(level, &search).map_err(XrfError::new_asset_error)?;

    Ok(LevelWeatherPart::record(level, &search, |search| {
      Self::to_render_weather(level, &environment, search)
    }))
  }

  fn read_weather_cycle(&self, name: &str) -> XrfResult<Vec<WeatherDescriptor>> {
    let environment: Arc<LevelEnvironment> =
      LevelEnvironmentPart::get(&self.level, &self.search()?).map_err(XrfError::new_asset_error)?;
    let cycle = environment
      .catalog
      .find_cycle(name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("There is no weather cycle '{name}'")))?;

    Ok(cycle.list_descriptors(environment.catalog.engine))
  }

  fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>> {
    let packed: Option<PackedLevelDetails> =
      LevelDetailsPart::pack(&self.level, &self.search()?).map_err(XrfError::new_asset_error)?;
    let Some(packed) = packed else {
      return Ok(None);
    };

    self.textures.extend(
      packed
        .textures
        .iter()
        .map(|texture| (texture.reference.clone(), texture.logical_path.clone())),
    );

    Ok(Some(RenderLevelDetails {
      package: packed.package,
      surfaces: packed.surfaces,
    }))
  }

  fn read_spawn(&self) -> XrfResult<RenderLevelSpawn> {
    let level: &SessionSnapshot<SelectedLevel> = &self.level;
    let spawn = LevelSpawnPart::get(level, &self.search()?).map_err(XrfError::new_asset_error)?;
    let described = describe_spawn_objects(&spawn);

    Ok(RenderLevelSpawn {
      objects: described
        .objects
        .iter()
        .map(|object| RenderSpawnObject {
          index: object.index,
          category: Self::to_render_category(object.category),
          is_released: object.release.is_some(),
          visual: object.visual,
          transform: object.transform.to_matrix(),
        })
        .collect(),
      visuals: described.visuals,
      detail: 0.0,
    })
  }

  fn read_particles(&self) -> XrfResult<Option<RenderLevelParticles>> {
    let level: &SessionSnapshot<SelectedLevel> = &self.level;
    let search: XraySearch = self.search()?;
    let packed: PackedLevelParticles =
      LevelParticlesPart::pack(level, &search, Self::read_sections(level, &search).as_deref())
        .map_err(XrfError::new_asset_error)?;
    let collision = LevelCollisionPart::get(level, &search);
    let hemi = LevelHemiPart::get(level, &search);

    self.textures.extend(
      packed
        .textures
        .iter()
        .map(|texture| (texture.reference.clone(), texture.logical_path.clone())),
    );

    Ok(Some(RenderLevelParticles {
      library: Arc::new(packed.library),
      rules: ParticleEngineRules::new(level.engine.engine, ParticleEngineRules::DEFAULT_UPDATE_COEFFICIENT),
      placements: packed.placements,
      surfaces: packed.surfaces,
      collider: collision
        .inspect_err(|error| report_missing_particle_collision(&level.source, error))
        .ok()
        .map(|tracer| Arc::new(LevelParticleCollider::new(tracer)) as Arc<dyn ParticleCollider>),
      // Its failure is reported where the estimator is built.
      hemi: hemi.ok(),
    }))
  }

  fn read_object_motion(&self, name: &str) -> XrfResult<AnmFile> {
    LevelObjectMotionsPart::read(&self.level, &self.search()?, name).map_err(XrfError::new_asset_error)
  }

  fn read_spawn_models(&self, names: &[String]) -> XrfResult<RenderSpawnModels> {
    let level: &SessionSnapshot<SelectedLevel> = &self.level;
    let search: XraySearch = self.search()?;
    let mut failures: Vec<RenderLoadFailure> = Vec::new();
    // A visual that cannot be read is reported by the reader, and left out.
    let read: Vec<(&str, Arc<LevelSpawnVisual>)> = SpawnVisualReader::with(level, &search, |visuals| {
      names
        .iter()
        .filter_map(|name| match visuals.get(name) {
          Ok(visual) => Some((name.as_str(), visual)),
          Err(error) => {
            failures.push(RenderLoadFailure {
              name: name.clone(),
              reason: error.to_string(),
            });

            None
          }
        })
        .collect()
    });
    let hemi = LevelHemiPart::get(level, &search)
      .map(|estimator| LevelHemiPart::estimate_visuals(level, &search, &estimator, &read))
      .unwrap_or_default();

    self.textures.extend(read.iter().flat_map(|(_, visual)| {
      visual
        .textures
        .iter()
        .map(|texture| (texture.reference.clone(), texture.logical_path.clone()))
    }));

    Ok(RenderSpawnModels {
      models: read
        .iter()
        .map(|(name, visual)| RenderSpawnModel {
          name: (*name).to_owned(),
          package: VisualPoser::pose(&visual.package, visual.rest.as_deref()),
          surfaces: visual.surfaces.clone(),
          skeleton: None,
        })
        .collect(),
      lighting: hemi
        .into_iter()
        .map(|object| RenderSpawnLighting {
          object: object.index,
          cube: object.cube,
          sky: object.sky,
        })
        .collect(),
      failures,
    })
  }
}
