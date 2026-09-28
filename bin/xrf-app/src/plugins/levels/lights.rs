//! A level's lights: those its spawned objects carry, and its own.

use std::time::Instant;

use xrf_chunk::XRayByteOrder;
use xrf_light_anim::LightAnimFile;
use xrf_ltx::Ltx;
use xrf_material::XrayTextureScope;
use xrf_vfs::XrayProbe;
use xrf_visual::{LightsDescription, LightsPacker};

use crate::core::assets::read_located_asset;
use crate::plugins::levels::report::{report_lights, report_missing_spawned_lights, report_unreadable_animations};
use crate::plugins::levels::spawn::get_level_spawn;
use crate::plugins::levels::spawn_visuals::SpawnVisualReader;
use crate::plugins::levels::state::{LevelTextureReference, SelectedLevel};
use crate::plugins::levels::textures::resolve_reference;

/// The colour animations lights name.
const ANIMATIONS_FILE: &str = "lanims.xr";

/// A level's lights, and what each projector a spot names resolved to.
pub struct PackedLevelLights {
  pub lights: LightsDescription,
  pub projectors: Vec<LevelTextureReference>,
}

/// Collects the open level's lights. Where its spawn cannot be read the level keeps its own lights alone, and without
/// configs its zones light nothing and its lamps keep their spawned flags.
pub fn pack_lights(current: &SelectedLevel, probe: &XrayProbe, sections: Option<&Ltx>) -> PackedLevelLights {
  let started: Instant = Instant::now();
  let animations: Option<LightAnimFile> = read_located_asset(probe, ANIMATIONS_FILE)
    .and_then(LightAnimFile::read_from_bytes::<XRayByteOrder>)
    .inspect_err(|error| report_unreadable_animations(ANIMATIONS_FILE, error))
    .ok();
  let mut packer: LightsPacker = LightsPacker::new(animations.as_ref());

  if let Some(sections) = sections {
    packer = packer.with_sections(sections);
  }

  match get_level_spawn(current, probe) {
    Ok(spawn) => {
      let visuals: SpawnVisualReader = SpawnVisualReader::new(current, probe);

      packer.add_objects(&spawn.objects, &mut |name| {
        visuals.get(name).and_then(|it| it.rest.clone())
      });
    }
    Err(error) => report_missing_spawned_lights(&current.source, &error),
  }

  if let Some(lights) = current.level.lights.as_ref() {
    packer.add_level_lights(&lights.lights);
  }

  let lights: LightsDescription = packer.pack();
  let scope: XrayTextureScope = current.source.get_texture_scope();

  report_lights(&current.source, &lights, started);

  PackedLevelLights {
    projectors: lights
      .projectors
      .iter()
      .map(|reference| LevelTextureReference {
        logical_path: resolve_reference(probe, &scope, reference),
        reference: reference.clone(),
      })
      .collect(),
    lights,
  }
}
