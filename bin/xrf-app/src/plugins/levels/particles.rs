//! A level's particle systems: the game's library, what the level plants and its zones play, and how each effect draws.

use std::collections::{BTreeSet, HashMap};
use std::time::Instant;

use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelPsStaticFile, PsStaticPlacement};
use xrf_ltx::{Ltx, Section};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};
use xrf_particles::{ParticleEffect, ParticleLibrary, ParticlesFile};
use xrf_renderer::{RenderParticlePlacement, RenderParticleSource};
use xrf_spawn::{AlifeObject, AlifeObjectInherited};
use xrf_vfs::XrayProbe;
use xrf_visual::VisualTransform;

use crate::core::assets::read_located_asset;
use crate::plugins::levels::read::read_optional_file;
use crate::plugins::levels::report::{report_missing_zone_particles, report_particles, report_unknown_particles};
use crate::plugins::levels::spawn::get_level_spawn;
use crate::plugins::levels::state::{LevelSource, LevelTextureReference, PS_STATIC_FILE, SelectedLevel};
use crate::plugins::levels::textures::resolve_reference;

/// The game's particle library, at the root of its data (`$game_data$`).
const PARTICLES_FILE: &str = "particles.xr";

/// `eGameIDSingle`: the game type a planted system must name to play in a single player game.
const SINGLE_PLAYER: u16 = 1;

/// The height `CLevel::Load_GameSpecific_After` lifts every planted system by.
const PLANTED_LIFT: f32 = 0.01;

/// `CLSID_Z_CAMPFIRE`'s class name, which a campfire's section names.
const CAMPFIRE_CLASS: &str = "Z_CFIRE";

/// A level's particle systems as the renderer takes them.
pub struct PackedLevelParticles {
  pub library: ParticleLibrary,
  pub placements: Vec<RenderParticlePlacement>,
  /// How each effect a placement can play draws, by its name.
  pub surfaces: HashMap<String, XraySurfaceDescriptor>,
  pub textures: Vec<LevelTextureReference>,
}

/// Reads the game's particle library and places what the open level plants and what its zones play, each effect any
/// of them reaches described.
///
/// # Errors
///
/// Returns why the library cannot be read; a level without `level.ps_static` or a spawn plants nothing of it.
pub fn pack_particles(
  current: &SelectedLevel,
  probe: &XrayProbe,
  sections: Option<&Ltx>,
) -> Result<PackedLevelParticles, String> {
  let started: Instant = Instant::now();
  let library: ParticleLibrary = read_library(probe)?;
  let mut placements: Vec<RenderParticlePlacement> = read_planted(&current.source, probe)?;

  // Without the configs no zone names its particles; their reader has said why.
  match (get_level_spawn(current, probe), sections) {
    (Ok(spawn), Some(sections)) => placements.extend(spawn.list_kept().filter_map(|it| place_zone(it, sections))),
    (Err(error), _) => report_missing_zone_particles(&current.source, &error),
    (Ok(_), None) => {}
  }

  let scope: XrayTextureScope = current.source.get_texture_scope();
  let resolver: XraySurfaceResolver = XraySurfaceResolver::open(probe, scope.clone());
  let mut surfaces: HashMap<String, XraySurfaceDescriptor> = HashMap::new();
  let mut references: BTreeSet<String> = BTreeSet::new();

  for name in list_reached_effects(&library, &placements) {
    let Some(effect) = library.get_effect(&name) else {
      continue;
    };
    let textures: Vec<String> = split_textures(effect);

    references.extend(textures.iter().cloned());
    surfaces.insert(name, resolver.describe(&effect.sprite.shader_name, &textures));
  }

  report_particles(&current.source, &library, placements.len(), surfaces.len(), started);

  Ok(PackedLevelParticles {
    textures: references
      .into_iter()
      .map(|reference| LevelTextureReference {
        logical_path: resolve_reference(probe, &scope, &reference),
        reference,
      })
      .collect(),
    library,
    placements,
    surfaces,
  })
}

fn read_library(probe: &XrayProbe) -> Result<ParticleLibrary, String> {
  read_located_asset(probe, PARTICLES_FILE)
    .and_then(ParticlesFile::read_from_bytes::<XRayByteOrder>)
    .map(ParticleLibrary::from)
    .map_err(|error| format!("Failed to read '{PARTICLES_FILE}': {error}"))
}

/// `CLevel::Load_GameSpecific_After`: each system the level plants for a single player game, lifted a centimetre.
fn read_planted(source: &LevelSource, probe: &XrayProbe) -> Result<Vec<RenderParticlePlacement>, String> {
  let Some(bytes) = read_optional_file(source, probe, PS_STATIC_FILE)? else {
    return Ok(Vec::new());
  };
  let file: LevelPsStaticFile = LevelPsStaticFile::read_from_bytes::<XRayByteOrder>(bytes).map_err(|error| {
    format!(
      "Failed to read '{PS_STATIC_FILE}' of level '{}': {error}",
      source.get_label()
    )
  })?;

  Ok(
    file
      .placements
      .iter()
      .filter(|placement| placement.game_types.is_none_or(|types| types & SINGLE_PLAYER != 0))
      .map(place_planted)
      .collect(),
  )
}

fn place_planted(placement: &PsStaticPlacement) -> RenderParticlePlacement {
  let mut transform: [f32; 16] = placement.transform.values;

  // The `Fmatrix` rows are the columns of a column-major matrix; the translation is the last.
  transform[13] += PLANTED_LIFT;

  RenderParticlePlacement {
    source: RenderParticleSource::Static {
      name: placement.effect.clone(),
    },
    transform,
  }
}

/// `CCustomZone::PlayIdleParticles`: a zone whose section names idle particles plays them at its `XFORM`, and a
/// campfire switches between them and its own.
fn place_zone(object: &AlifeObject, sections: &Ltx) -> Option<RenderParticlePlacement> {
  if !matches!(
    object.inherited,
    AlifeObjectInherited::CseAlifeAnomalousZone(_)
      | AlifeObjectInherited::CseAlifeZoneVisual(_)
      | AlifeObjectInherited::CseAlifeTorridZone(_)
  ) {
    return None;
  }

  let section: &Section = sections.section(&object.section)?;
  let read = |key: &str| {
    section
      .get(key)
      .map(str::trim)
      .filter(|value| !value.is_empty())
      .map(str::to_owned)
  };
  let idle: String = read("idle_particles")?;
  let source: RenderParticleSource = match (
    read("class").as_deref() == Some(CAMPFIRE_CLASS),
    read("disabled_particles"),
    read("enabling_particles"),
  ) {
    (true, Some(disabled), Some(enabling)) => RenderParticleSource::Campfire {
      idle,
      disabled,
      enabling,
    },
    _ => RenderParticleSource::Zone { idle },
  };

  Some(RenderParticlePlacement {
    source,
    transform: VisualTransform::of_spawn(&object.position, &object.direction)
      .mirrored()
      .to_matrix(),
  })
}

/// Every effect a placement can play: the effects it names, each effect of the groups it names, and the children those
/// effects start.
fn list_reached_effects(library: &ParticleLibrary, placements: &[RenderParticlePlacement]) -> BTreeSet<String> {
  let mut reached: BTreeSet<String> = BTreeSet::new();
  let names = placements.iter().flat_map(|placement| match &placement.source {
    RenderParticleSource::Static { name } => vec![name.as_str()],
    RenderParticleSource::Zone { idle } => vec![idle.as_str()],
    RenderParticleSource::Campfire {
      idle,
      disabled,
      enabling,
    } => vec![idle.as_str(), disabled.as_str(), enabling.as_str()],
  });

  let mut unknown: BTreeSet<&str> = BTreeSet::new();

  for name in names {
    if library.get_effect(name).is_some() {
      reached.insert(name.to_owned());
    } else if let Some(group) = library.get_group(name) {
      for effect in &group.effects {
        for child in [
          &effect.name,
          &effect.on_play_child_name,
          &effect.on_birth_child_name,
          &effect.on_dead_child_name,
        ] {
          if !child.is_empty() {
            reached.insert(child.clone());
          }
        }
      }
    } else {
      unknown.insert(name);
    }
  }

  if !unknown.is_empty() {
    report_unknown_particles(&unknown.into_iter().collect::<Vec<&str>>());
  }

  reached
}

/// `_ParseList`: the comma-separated textures an effect's sprite names, the base first.
fn split_textures(effect: &ParticleEffect) -> Vec<String> {
  effect
    .sprite
    .texture_name
    .split(',')
    .map(str::trim)
    .filter(|texture| !texture.is_empty())
    .map(str::to_owned)
    .collect()
}

#[cfg(test)]
mod tests {
  use xrf_level::PsStaticPlacement;
  use xrf_math::Matrix4x4;
  use xrf_renderer::RenderParticleSource;

  use super::place_planted;

  #[test]
  fn lifts_a_planted_system_a_centimetre() {
    let mut values: [f32; 16] = [0.0; 16];

    values[0] = 1.0;
    values[5] = 1.0;
    values[10] = 1.0;
    values[12] = 4.0;
    values[13] = 2.0;
    values[15] = 1.0;

    let placement = place_planted(&PsStaticPlacement {
      game_types: Some(1),
      effect: String::from("zones\\zone_thermal_idle"),
      transform: Matrix4x4 { values },
    });

    assert_eq!(placement.transform[12], 4.0);
    assert_eq!(placement.transform[13], 2.0 + 0.01);
    assert_eq!(
      placement.source,
      RenderParticleSource::Static {
        name: String::from("zones\\zone_thermal_idle")
      }
    );
  }
}
