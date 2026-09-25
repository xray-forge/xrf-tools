//! A level's lights: those its spawned objects carry, and its own.

use std::time::Instant;

use xrf_light_anim::LightAnimFile;
use xrf_ogf::OgfFile;
use xrf_spawn::XRayByteOrder;
use xrf_vfs::XrayProbe;
use xrf_visual::{LightsDescription, LightsPacker};

use crate::plugins::levels::read::read_asset;
use crate::plugins::levels::spawn::get_level_spawn;
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

  let mut packer: LightsPacker = LightsPacker::new(animations.as_ref());

  match get_level_spawn(current, probe) {
    Ok(spawn) => packer.add_objects(&spawn.objects, &mut |visual| read_visual(probe, visual)),
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

/// A visual by the name an object gives it, or `None` where it cannot be read: a light then stands at the object's
/// own place, as the engine stands one without a bone.
fn read_visual(probe: &XrayProbe, visual: &str) -> Option<OgfFile> {
  let path: String = format!("meshes\\{visual}.ogf");

  match read_asset(probe, &path)
    .and_then(|bytes| OgfFile::read_from_bytes::<XRayByteOrder>(bytes).map_err(|error| error.to_string()))
  {
    Ok(file) => Some(file),
    Err(error) => {
      log::warn!("Visual '{path}' placed nothing: {error}");

      None
    }
  }
}
