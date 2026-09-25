//! A level's lights: those its spawned objects carry, and its own.

use std::sync::Arc;
use std::time::Instant;

use xrf_light_anim::LightAnimFile;
use xrf_ltx::LtxResolution;
use xrf_spawn::XRayByteOrder;
use xrf_vfs::XrayProbe;
use xrf_visual::{LightsDescription, LightsPacker};

use crate::plugins::levels::configs::get_level_configs;
use crate::plugins::levels::read::read_asset;
use crate::plugins::levels::spawn::get_level_spawn;
use crate::plugins::levels::spawn_visuals::get_spawn_visual;
use crate::plugins::levels::state::{LevelTextureReference, SelectedLevel};
use crate::plugins::levels::textures::resolve_reference;

/// The colour animations lights name.
const ANIMATIONS_FILE: &str = "lanims.xr";

/// A level's lights, and what each projector a spot names resolved to.
pub struct PackedLevelLights {
  pub lights: LightsDescription,
  pub projectors: Vec<LevelTextureReference>,
}

/// Collects the open level's lights. Where its spawn cannot be read the level keeps its own lights alone.
pub fn pack_lights(current: &SelectedLevel, probe: &XrayProbe, directory: Option<&str>) -> PackedLevelLights {
  let started: Instant = Instant::now();
  let animations: Option<LightAnimFile> = read_asset(probe, ANIMATIONS_FILE)
    .ok()
    .and_then(|bytes| LightAnimFile::read_from_bytes::<XRayByteOrder>(bytes).ok());

  if animations.is_none() {
    log::warn!("No readable '{ANIMATIONS_FILE}', so no light is animated");
  }

  let configs: Option<Arc<LtxResolution>> = get_level_configs(current)
    .inspect_err(|error| log::warn!("No zone lights and no lamp sections: {error}"))
    .ok();
  let mut packer: LightsPacker = LightsPacker::new(animations.as_ref());

  if let Some(configs) = configs.as_ref() {
    packer = packer.with_sections(&configs.ltx);
  }

  match get_level_spawn(current, probe) {
    Ok(spawn) => packer.add_objects(&spawn.objects, &mut |visual| {
      get_spawn_visual(current, probe, visual).and_then(|it| it.rest.clone())
    }),
    Err(error) => log::warn!("No spawned lights for {}: {error}", current.source.get_label()),
  }

  if let Some(lights) = current.level.lights.as_ref() {
    packer.add_level_lights(&lights.lights);
  }

  let lights: LightsDescription = packer.pack();

  log::info!(
    "Collected lights of {}: {} lights, {} animators, {} projectors in {:?}",
    current.source.get_label(),
    lights.lights.len(),
    lights.animators.len(),
    lights.projectors.len(),
    started.elapsed()
  );

  PackedLevelLights {
    projectors: lights
      .projectors
      .iter()
      .map(|reference| LevelTextureReference {
        logical_path: resolve_reference(probe, directory, reference),
        reference: reference.clone(),
      })
      .collect(),
    lights,
  }
}
