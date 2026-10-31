use std::collections::HashMap;
use std::sync::Arc;

use xrf_app_core::session::SessionSnapshot;
use xrf_environment::{EnvironmentCatalog, WeatherModifier};
use xrf_level::LevelEnvModifier;
use xrf_math::Vector3d;
use xrf_renderer::{RenderLensFlare, RenderLevelWeather, RenderRain, RenderWetSurfaces};
use xrf_vfs::XraySearch;
use xrf_world::WorldLevelWeather;

use crate::level::SelectedLevel;
use crate::parts::LevelEnvironment;
use crate::render::{LevelLensFlares, LevelWorldAmbients};
use crate::weather::render::{LevelRain, LevelThunderbolts, LevelWetSurfaces};

/// What a level's weathers play with as the world takes them, what the renderer draws them with among it.
pub struct LevelWorldWeather;

impl LevelWorldWeather {
  /// The level's weather as the world takes it, the environment's catalog read already and the rest read through
  /// `search`.
  pub fn read(
    level: &SessionSnapshot<SelectedLevel>,
    environment: &LevelEnvironment,
    search: &XraySearch,
  ) -> WorldLevelWeather {
    let source = environment.get_source(level, search);
    let rain: LevelRain = LevelRain::read(&source);
    let wet: LevelWetSurfaces = LevelWetSurfaces::read(&source);
    let thunder: LevelThunderbolts = LevelThunderbolts::read(&source);
    let modifiers: Vec<LevelEnvModifier> = LevelEnvironment::read_modifiers(level, search);

    WorldLevelWeather {
      render: Arc::new(RenderLevelWeather {
        rain: Some(RenderRain {
          streak: rain.streak.reference,
          drop: rain.drop.map(Into::into),
        }),
        wet: Some(RenderWetSurfaces {
          splash: wet.splash.reference,
          flow: wet.flow.reference,
        }),
        thunder: Some(thunder.into()),
        suns: Self::to_suns(&environment.catalog),
      }),
      engine: environment.catalog.engine,
      effects: environment
        .catalog
        .effects
        .iter()
        .map(|effect| (effect.name.clone(), effect.list_descriptors(environment.catalog.engine)))
        .collect(),
      modifiers: modifiers.iter().map(Self::to_modifier).collect(),
      sun_table: environment
        .catalog
        .sun_table
        .as_ref()
        .map(|table| table.list_positions()),
      ambients: LevelWorldAmbients::to_world(&environment.catalog, &level.source.get_name().unwrap_or_default()),
      ambient_effects: LevelWorldAmbients::to_world_effects(&environment.catalog),
    }
  }

  /// A `level.env_mod` volume as the mixer weighs it: one from a file before the flag word adds every value, as the
  /// engine's `use_flags.one()` before the read leaves it.
  fn to_modifier(modifier: &LevelEnvModifier) -> WeatherModifier {
    let to_array = |vector: &Vector3d<f32>| [vector.x, vector.y, vector.z];

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

  /// Every lens flare of the game as the renderer draws it, by section, each read as its engine reads it.
  fn to_suns(catalog: &EnvironmentCatalog) -> HashMap<String, RenderLensFlare> {
    catalog
      .suns
      .iter()
      .map(|sun| (sun.name.clone(), LevelLensFlares::to_render(sun, catalog.engine)))
      .collect()
  }
}
