use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use xrf_anm::AnmFile;
use xrf_chunk::XRayByteOrder;
use xrf_environment::{EnvironmentCatalog, WeatherDescriptor, WeatherModifier};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{EnvModifier, LevelSector, LevelSectorComposition};
use xrf_ltx::Ltx;
use xrf_material::{XrayReducedTextures, XraySurfaceDescriptor};
use xrf_particles::{ParticleCollider, ParticleEngineRules};
use xrf_renderer::{
  RenderAssetSource, RenderLensFlare, RenderLevelDetails, RenderLevelParticles, RenderLevelSource, RenderLevelSpawn,
  RenderLevelWeather, RenderLoadFailure, RenderRain, RenderSpawnCategory, RenderSpawnLighting, RenderSpawnModel,
  RenderSpawnModels, RenderSpawnObject, RenderThunder, RenderThunderSettings, RenderThunderbolt,
  RenderThunderboltGradient, RenderThunderboltModel, RenderWeatherModel, RenderWetSurfaces,
};
use xrf_visual::{LightsDescription, SectorPackage, SectorPacker, VisualPoser};

use crate::core::assets::AssetMountState;
use crate::core::session::SessionSnapshot;
use crate::plugins::levels::ambients::{to_render_ambient_effects, to_render_ambients};
use crate::plugins::levels::collision::get_level_collision;
use crate::plugins::levels::configs::get_level_sections;
use crate::plugins::levels::details::{PackedLevelDetails, pack_details};
use crate::plugins::levels::drawn_attributes::DRAWN_ATTRIBUTES;
use crate::plugins::levels::hemi::{estimate_visuals_hemi, get_level_hemi};
use crate::plugins::levels::lens_flares::to_render_lens_flare;
use crate::plugins::levels::lights::{PackedLevelLights, pack_lights};
use crate::plugins::levels::object_motions::read_object_motion;
use crate::plugins::levels::particle_collider::LevelParticleCollider;
use crate::plugins::levels::particles::{PackedLevelParticles, pack_particles};
use crate::plugins::levels::report::{
  report_missing_particle_collision, report_missing_sections, report_packed_sector,
};
use crate::plugins::levels::spawn::get_level_spawn;
use crate::plugins::levels::spawn_objects::describe_spawn_objects;
use crate::plugins::levels::spawn_visuals::SpawnVisualReader;
use crate::plugins::levels::state::selection::level_rain::LevelRain;
use crate::plugins::levels::state::selection::level_thunderbolt_gradient::LevelThunderboltGradient;
use crate::plugins::levels::state::selection::level_thunderbolts::LevelThunderbolts;
use crate::plugins::levels::state::selection::level_weather_model::LevelWeatherModel;
use crate::plugins::levels::state::selection::level_wet_surfaces::LevelWetSurfaces;
use crate::plugins::levels::state::{LevelEnvironment, LevelSpawnCategory, LevelSpawnVisual, SelectedLevel};
use crate::plugins::levels::texture_resolver::TextureResolver;

/// The open level as the native renderer draws it: sectors packed when its loader asks, textures read from the roots
/// the level was opened in.
pub struct LevelRenderSource {
  level: Arc<SessionSnapshot<SelectedLevel>>,
  assets: AssetMountState,
  /// Where each texture reference the level's surfaces bind resolved to at open, each projector its lights name once
  /// they are read, and anything else once asked for, as the weather's skies are.
  textures: TextureResolver,
  /// The textures the game loads a step coarser at the same texture quality, read the first time a texture quality
  /// above full asks.
  reduced: OnceLock<XrayReducedTextures>,
}

impl LevelRenderSource {
  pub fn new(level: Arc<SessionSnapshot<SelectedLevel>>, assets: AssetMountState) -> Self {
    let textures: TextureResolver = TextureResolver::new(
      level.roots.clone(),
      level.source.get_texture_scope(),
      level
        .textures
        .iter()
        .map(|texture| (texture.reference.clone(), texture.logical_path.clone())),
    );

    Self {
      level,
      assets,
      textures,
      reduced: OnceLock::new(),
    }
  }
}

impl LevelRenderSource {
  /// The sections the level's spawned objects name, none where its spawn or the configs cannot be read. Read between
  /// two probes, as the configs mount a tree of their own.
  fn read_sections(&self) -> XrfResult<Option<Arc<Ltx>>> {
    let level: &SelectedLevel = &self.level;

    Ok(
      self
        .assets
        .with_probe(&level.roots, |probe| get_level_spawn(level, probe))
        .map_err(XrfError::new_asset_error)?
        .and_then(|spawn| get_level_sections(level, &spawn))
        .inspect_err(report_missing_sections)
        .ok(),
    )
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
    self
      .reduced
      .get_or_init(|| {
        self
          .read_sections()
          .ok()
          .flatten()
          .and_then(|sections| {
            sections
              .section(XrayReducedTextures::SECTION)
              .map(|section| XrayReducedTextures::new(section.iter().map(|(key, _)| key)))
          })
          .unwrap_or_default()
      })
      .is_reduced(reference)
  }
}

impl RenderLevelSource for LevelRenderSource {
  fn get_sector_count(&self) -> u32 {
    self.level.get_sectors().len() as u32
  }

  /// The level as it was opened: every source of one opening shares a scene, a level opened again streams afresh.
  fn get_scene_key(&self) -> Option<String> {
    Some(format!("{:p}", Arc::as_ptr(&self.level)))
  }

  fn pack_sector(&self, sector: u32) -> XrfResult<SectorPackage> {
    let sectors: &[LevelSector] = self.level.get_sectors();
    let root: u32 = sectors
      .get(sector as usize)
      .ok_or_else(|| XrfError::new_not_found_error(format!("The level has no sector {sector}")))?
      .root;
    let composition: LevelSectorComposition = LevelSectorComposition::of(&self.level.visuals, root);
    let started: Instant = Instant::now();
    let package: SectorPackage = SectorPacker::new(
      &self.level.visuals,
      self.level.level.shaders.as_ref(),
      &self.level.geometry,
    )
    .pack::<XRayByteOrder>(sector, &composition, DRAWN_ATTRIBUTES);

    report_packed_sector(&package, started);

    Ok(package)
  }

  fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
    &self.level.surfaces
  }

  fn read_lights(&self) -> XrfResult<LightsDescription> {
    let level: &SelectedLevel = &self.level;
    let sections: Option<Arc<Ltx>> = self.read_sections()?;
    let packed: PackedLevelLights = self
      .assets
      .with_probe(&level.roots, |probe| pack_lights(level, probe, sections.as_deref()))
      .map_err(XrfError::new_asset_error)?;

    self.textures.extend(
      packed
        .projectors
        .into_iter()
        .map(|projector| (projector.reference, projector.logical_path)),
    );

    Ok(packed.lights)
  }

  fn read_weather(&self) -> XrfResult<RenderLevelWeather> {
    let level: &SelectedLevel = &self.level;
    let environment: Arc<LevelEnvironment> = LevelEnvironment::of(level).map_err(XrfError::new_asset_error)?;
    let (modifiers, rain, wet, thunder) = self
      .assets
      .with_probe(&level.roots, |probe| {
        let source = environment.get_source(level, probe);

        (
          LevelEnvironment::read_modifiers(level, probe),
          LevelRain::read(&source),
          LevelWetSurfaces::read(&source),
          LevelThunderbolts::read(&source),
        )
      })
      .map_err(XrfError::new_asset_error)?;

    Ok(RenderLevelWeather {
      engine: environment.catalog.engine,
      effects: environment
        .catalog
        .effects
        .iter()
        .map(|effect| (effect.name.clone(), effect.list_descriptors(environment.catalog.engine)))
        .collect(),
      modifiers: modifiers.iter().map(to_weather_modifier).collect(),
      sun_table: environment
        .catalog
        .sun_table
        .as_ref()
        .map(|table| table.list_positions()),
      rain: Some(RenderRain {
        streak: rain.streak.reference,
        drop: rain.drop.map(to_weather_model),
      }),
      wet: Some(RenderWetSurfaces {
        splash: wet.splash.reference,
        flow: wet.flow.reference,
      }),
      thunder: Some(to_thunder(thunder)),
      suns: to_suns(&environment.catalog),
      ambients: to_render_ambients(&environment.catalog, &level.source.get_name().unwrap_or_default()),
      ambient_effects: to_render_ambient_effects(&environment.catalog),
    })
  }

  fn read_weather_cycle(&self, name: &str) -> XrfResult<Vec<WeatherDescriptor>> {
    let environment: Arc<LevelEnvironment> = LevelEnvironment::of(&self.level).map_err(XrfError::new_asset_error)?;
    let cycle = environment
      .catalog
      .find_cycle(name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("There is no weather cycle '{name}'")))?;

    Ok(cycle.list_descriptors(environment.catalog.engine))
  }

  fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>> {
    let level: &SelectedLevel = &self.level;
    let packed: Option<PackedLevelDetails> = self
      .assets
      .with_probe(&level.roots, |probe| pack_details(&level.source, probe))
      .map_err(XrfError::new_asset_error)?
      .map_err(XrfError::new_asset_error)?;
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
    let level: &SelectedLevel = &self.level;
    let spawn = self
      .assets
      .with_probe(&level.roots, |probe| get_level_spawn(level, probe))
      .map_err(XrfError::new_asset_error)?
      .map_err(XrfError::new_asset_error)?;
    let described = describe_spawn_objects(&spawn);

    Ok(RenderLevelSpawn {
      objects: described
        .objects
        .iter()
        .map(|object| RenderSpawnObject {
          index: object.index,
          category: to_render_category(object.category),
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
    let level: &SelectedLevel = &self.level;
    let sections: Option<Arc<Ltx>> = self.read_sections()?;
    let (packed, collision, hemi) = self
      .assets
      .with_probe(&level.roots, |probe| {
        (
          pack_particles(level, probe, sections.as_deref()),
          get_level_collision(level, probe),
          get_level_hemi(level, probe),
        )
      })
      .map_err(XrfError::new_asset_error)?;
    let packed: PackedLevelParticles = packed.map_err(XrfError::new_asset_error)?;

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
    let level: &SelectedLevel = &self.level;

    self
      .assets
      .with_probe(&level.roots, |probe| read_object_motion(&level.source, probe, name))
      .map_err(XrfError::new_asset_error)?
      .map_err(XrfError::new_asset_error)
  }

  fn read_spawn_models(&self, names: &[String]) -> XrfResult<RenderSpawnModels> {
    let level: &SelectedLevel = &self.level;
    let (read, failures, hemi) = self
      .assets
      .with_probe(&level.roots, |probe| {
        let visuals: SpawnVisualReader = SpawnVisualReader::new(level, probe);
        let mut failures: Vec<RenderLoadFailure> = Vec::new();
        // A visual that cannot be read is reported by the reader, and left out.
        let read: Vec<(&str, Arc<LevelSpawnVisual>)> = names
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
          .collect();
        let hemi = get_level_hemi(level, probe)
          .map(|estimator| estimate_visuals_hemi(level, probe, &estimator, &read))
          .unwrap_or_default();

        (read, failures, hemi)
      })
      .map_err(XrfError::new_asset_error)?;

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

/// The group a spawn category's objects are shown in.
fn to_render_category(category: LevelSpawnCategory) -> RenderSpawnCategory {
  match category {
    LevelSpawnCategory::Props => RenderSpawnCategory::Props,
    LevelSpawnCategory::Items => RenderSpawnCategory::Items,
    LevelSpawnCategory::Weapons => RenderSpawnCategory::Weapons,
    LevelSpawnCategory::Lamps => RenderSpawnCategory::Lamps,
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
        mesh: to_weather_model(model.mesh),
        draw: model.draw,
      })
      .collect(),
    animators: thunder.animators,
  }
}
