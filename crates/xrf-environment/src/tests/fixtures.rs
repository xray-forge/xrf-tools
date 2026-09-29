use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use xrf_dltx::DltxDialect;
use xrf_engine_target::XrayEngine;
use xrf_ltx::{LtxProject, LtxProjectOptions};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::{EnvironmentCatalog, EnvironmentFinding, EnvironmentReadOptions, EnvironmentReader, EnvironmentRule};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

/// A config tree written for one test, rooted where the configs are, as `configs` is in a game.
pub struct EnvironmentFixture {
  root: PathBuf,
  is_dltx: bool,
}

impl EnvironmentFixture {
  /// A tree holding the definitions a clean keyframe names, and nothing else.
  pub fn new() -> Self {
    let unique: u64 = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("environment/fixture-{unique}"));

    // A panicking test never reaches a cleanup of its own, so a fixture must not open on what one left.
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    Self { is_dltx: false, root }
      .with("environment\\suns.ltx", SUN)
      .with("environment\\thunderbolt_collections.ltx", "[bolts]\nbolt =\n")
      .with("environment\\thunderbolts.ltx", THUNDERBOLT)
      .with("environment\\environment.ltx", THUNDERBOLT_SETTINGS)
      .with(
        "environment\\ambients.ltx",
        "[day]\nsound_channels = wind\neffects = gust\nmin_effect_period = 30\nmax_effect_period = 60\n",
      )
      .with("environment\\sound_channels.ltx", SOUND_CHANNEL)
      .with("environment\\effects.ltx", EFFECT)
  }

  /// Writes one config, named relative to the configs root.
  pub fn with(self, relative: &str, contents: &str) -> Self {
    let path: PathBuf = self.root.join(relative.replace('\\', "/"));

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();

    self
  }

  /// Takes one config away again.
  pub fn without(self, relative: &str) -> Self {
    let _ = fs::remove_file(self.root.join(relative.replace('\\', "/")));

    self
  }

  /// Resolves the tree with DLTX, as Anomaly's is.
  pub fn with_dltx(mut self) -> Self {
    self.is_dltx = true;

    self
  }

  pub fn open(&self) -> LtxProject {
    let options: LtxProjectOptions = if self.is_dltx {
      LtxProjectOptions::new().with_dialect(Arc::new(DltxDialect))
    } else {
      LtxProjectOptions::new()
    };

    LtxProject::open_at_path_opt(&self.root, options).unwrap()
  }

  pub fn read(&self, engine: XrayEngine) -> EnvironmentCatalog {
    EnvironmentReader::read(&self.open(), engine).unwrap()
  }

  pub fn read_explained(&self, engine: XrayEngine) -> EnvironmentCatalog {
    EnvironmentReader::read_opt(
      &self.open(),
      engine,
      &EnvironmentReadOptions::default().with_explained(true),
    )
    .unwrap()
  }
}

impl Drop for EnvironmentFixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

/// The messages of every finding of one rule, which is what most assertions read.
pub fn messages(catalog: &EnvironmentCatalog, rule: EnvironmentRule) -> Vec<&str> {
  catalog
    .findings
    .iter()
    .filter(|finding| finding.rule == rule)
    .map(|finding: &EnvironmentFinding| finding.message.as_str())
    .collect()
}

/// A vanilla keyframe writing every key OpenXRay requires, naming the fixture's definitions.
pub fn vanilla_keyframe(time: &str) -> String {
  format!(
    "[{time}]
ambient = day
ambient_color = 0.1, 0.1, 0.1
clouds_color = 0.2, 0.4, 0.6, 0.8, 1.0
clouds_texture = sky\\clouds
far_plane = 500
fog_color = 0.5, 0.5, 0.5
fog_density = 0.25
fog_distance = 450
hemisphere_color = 0.3, 0.3, 0.3, 1
rain_color = 0.7, 0.7, 0.7
rain_density = 0
sky_color = 1, 1, 1
sky_rotation = 90
sky_texture = sky\\sky_cube
sun = sun
sun_altitude = -45
sun_color = 1, 0.9, 0.8
sun_longitude = -30
thunderbolt_collection = bolts
thunderbolt_duration = 0.5
thunderbolt_period = 10
wind_direction = 180
wind_velocity = 5

"
  )
}

/// An extended keyframe writing every key Monolith requires, and none of the sun's angles, which it does not read.
pub fn extended_keyframe(time: &str) -> String {
  format!(
    "[{time}]
ambient = day
ambient_color = 0.1, 0.1, 0.1
clouds_color = 0, 0, 0, 0
clouds_texture =
far_plane = 650
fog_color = 0.5, 0.5, 0.5
fog_density = 1
fog_distance = 450
hemisphere_color = 0.3, 0.3, 0.3, 1
rain_color = 0.2, 0.2, 0.2
rain_density = 0
sky_color = 4, 4, 4
sky_rotation = 5
sky_texture = sky\\sky_cube
sun = sun
sun_color = 0.05, 0.05, 0.05
thunderbolt_collection =
thunderbolt_duration = 0
thunderbolt_period = 0
wind_direction = 0
wind_velocity = 150
bloom_threshold = 2.5

"
  )
}

/// Monolith's sun table, every hour written.
pub fn sun_table() -> String {
  (0..24)
    .map(|hour| {
      format!(
        "[{hour:02}:00:00]\nsun_longitude = -26\nsun_altitude = {}\n\n",
        hour * 15
      )
    })
    .collect()
}

const SUN: &str = "[sun]
blend_down_time = 60
blend_rise_time = 60
flare_opacity = 0.3, 0.2
flare_position = 1.3, 1.0
flare_radius = 0.08, 0.12
flare_shader = effects\\flare
flare_textures = fx\\fx_flare1.tga, fx\\fx_flare2.tga
flares = on
gradient = on
gradient_opacity = 0.7
gradient_radius = 0.9
gradient_shader = effects\\flare
gradient_texture = fx\\fx_gradient.tga
sun = on
sun_ignore_color = false
sun_radius = 0.15
sun_shader = effects\\sun
sun_texture = fx\\fx_sun.tga
";

const THUNDERBOLT: &str = "[bolt]
color_anim = weathers\\thunderbolt_00
gradient_center_opacity = 0.6
gradient_center_radius = 2, 1
gradient_center_shader = effects\\sun
gradient_center_texture = fx\\fx_thunderbolts_gradient
gradient_top_opacity = 0.6
gradient_top_radius = 0.5, 0.25
gradient_top_shader = effects\\sun
gradient_top_texture = fx\\fx_thunderbolts_gradient
lightning_model = dm\\dm_lightning-01.dm
sound = nature\\thunder-0
";

const THUNDERBOLT_SETTINGS: &str = "[environment]
altitude = 20
delta_longitude = 30
fog_color = 0.1
min_dist_factor = 0.94
second_propability = 0.5
sky_color = 0.1
sun_color = 0.9
tilt = 17
";

const SOUND_CHANNEL: &str = "[wind]
max_distance = 40
min_distance = 20
period0 = 25000
period1 = 30000
period2 = 5000
period3 = 20000
sounds = ambient\\wind_1, ambient\\wind_2
";

const EFFECT: &str = "[gust]
life_time = 10
offset = 0, 0, 0
particles = nature\\fog_stormy
sound = ambient\\rnd_wind_3
wind_gust_factor = 0.015
";
