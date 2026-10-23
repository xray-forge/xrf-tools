//! A level's particle systems: the game's library, what the level plants and its zones play, and how each effect draws.

use std::collections::{BTreeSet, HashMap};
use std::time::Instant;

use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelPsStaticFile, PsStaticPlacement};
use xrf_ltx::{Ltx, Section};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};
use xrf_particles::{ParticleLibrary, ParticlesFile};
use xrf_renderer::{RenderParticlePlacement, RenderParticleSource};
use xrf_spawn::{AlifeObject, AlifeObjectCustomZone, AlifeObjectInherited, AlifeObjectMotion, ClsId};
use xrf_vfs::XraySearch;
use xrf_visual::{VisualTransform, ZoneSphere};

use crate::core::assets::read_located_asset;
use crate::core::session::{SessionPart, SessionSnapshot};
use crate::plugins::levels::ambients::list_ambient_particles;
use crate::plugins::levels::read::read_optional_file;
use crate::plugins::levels::report::{report_missing_zone_particles, report_particles, report_unknown_particles};
use crate::plugins::levels::spawn::get_level_spawn;
use crate::plugins::levels::state::{
  LevelEnvironment, LevelSource, LevelTextureReference, PS_STATIC_FILE, SelectedLevel,
};
use crate::plugins::levels::textures::resolve_reference;

/// The game's particle library, at the root of its data (`$game_data$`).
const PARTICLES_FILE: &str = "particles.xr";

/// `eGameIDSingle`: the game type a planted system must name to play in a single player game.
const SINGLE_PLAYER: u16 = 1;

/// The height `CLevel::Load_GameSpecific_After` lifts every planted system by.
const PLANTED_LIFT: f32 = 0.01;

/// What a change event calls what a level's particle systems were read from when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_PARTICLES_PART: &str = "particles";

/// What a level's particle systems were read from: the game's library, `level.ps_static`, and the shaders and textures
/// their effects draw with, with the parts they ask for; the renderer holds the systems themselves.
struct LevelParticlesPart;

impl SessionPart for LevelParticlesPart {
  type Value = ();

  const NAME: &'static str = LEVEL_PARTICLES_PART;

  const IS_POOLED: bool = false;
}

/// A level's particle systems as the renderer takes them.
pub struct PackedLevelParticles {
  pub library: ParticleLibrary,
  pub placements: Vec<RenderParticlePlacement>,
  /// How each effect a placement can play draws, by its name.
  pub surfaces: HashMap<String, XraySurfaceDescriptor>,
  pub textures: Vec<LevelTextureReference>,
}

/// Reads the game's particle library and places what the open level plants and what its zones play, each effect any
/// of them reaches described, what it reads recorded as the level's particles part.
///
/// # Errors
///
/// Returns why the library cannot be read; a level without `level.ps_static` or a spawn plants nothing of it.
pub fn pack_particles(
  current: &SessionSnapshot<SelectedLevel>,
  search: &XraySearch,
  sections: Option<&Ltx>,
) -> Result<PackedLevelParticles, String> {
  current
    .parts
    .record_read::<LevelParticlesPart, _>(search, |search| pack_recorded(current, search, sections))
}

/// The particle systems packed, every read through `search`.
fn pack_recorded(
  current: &SessionSnapshot<SelectedLevel>,
  search: &XraySearch,
  sections: Option<&Ltx>,
) -> Result<PackedLevelParticles, String> {
  let started: Instant = Instant::now();
  let library: ParticleLibrary = read_library(search)?;
  let mut placements: Vec<RenderParticlePlacement> = read_planted(&current.source, search)?;

  // todo: Play the systems of space restrictors whose logic starts in `sr_particle` (`name` at `path`'s points, after
  //  each point's delay, `looped` or once), as the b53 steam field's do; condition switches would stay unread.

  // Without the configs no zone names its particles; their reader has said why.
  match (get_level_spawn(current, search), sections) {
    (Ok(spawn), Some(sections)) => placements.extend(spawn.list_kept().filter_map(|it| place_zone(it, sections))),
    (Err(error), _) => report_missing_zone_particles(&current.source, &error),
    (Ok(_), None) => {}
  }

  // An ambient effect plays near the camera whatever stands in the level, so its particles are described with the rest;
  // a level whose configs cannot be read has said why with its weather.
  let ambient_particles: Vec<String> = LevelEnvironment::of(current, search)
    .map(|environment| list_ambient_particles(&environment.catalog))
    .unwrap_or_default();
  let scope: XrayTextureScope = current.source.get_texture_scope();
  let resolver: XraySurfaceResolver = XraySurfaceResolver::open(search, scope.clone());
  let mut surfaces: HashMap<String, XraySurfaceDescriptor> = HashMap::new();
  let mut references: BTreeSet<String> = BTreeSet::new();

  for name in list_reached_effects(&library, &placements, &ambient_particles) {
    let Some(effect) = library.get_effect(&name) else {
      continue;
    };
    let textures: Vec<String> = effect.sprite.list_textures();

    references.extend(textures.iter().cloned());
    surfaces.insert(name, resolver.describe(&effect.sprite.shader_name, &textures));
  }

  report_particles(&current.source, &library, placements.len(), surfaces.len(), started);

  Ok(PackedLevelParticles {
    textures: references
      .into_iter()
      .map(|reference| LevelTextureReference {
        logical_path: resolve_reference(search, &scope, &reference),
        reference,
      })
      .collect(),
    library,
    placements,
    surfaces,
  })
}

fn read_library(search: &XraySearch) -> Result<ParticleLibrary, String> {
  read_located_asset(search, PARTICLES_FILE)
    .and_then(ParticlesFile::read_from_bytes::<XRayByteOrder>)
    .map(ParticleLibrary::from)
    .map_err(|error| format!("Failed to read '{PARTICLES_FILE}': {error}"))
}

/// `CLevel::Load_GameSpecific_After`: each system the level plants for a single player game, lifted a centimetre.
fn read_planted(source: &LevelSource, search: &XraySearch) -> Result<Vec<RenderParticlePlacement>, String> {
  let Some(bytes) = read_optional_file(source, search, PS_STATIC_FILE)? else {
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

/// One system `level.ps_static` plants, lifted off the ground it stands on.
pub fn place_planted(placement: &PsStaticPlacement) -> RenderParticlePlacement {
  let mut transform: [f32; 16] = placement.transform.values;

  // The `Fmatrix` rows are the columns of a column-major matrix; the translation is the last.
  transform[13] += PLANTED_LIFT;

  RenderParticlePlacement {
    source: RenderParticleSource::Static {
      name: placement.effect.clone(),
    },
    transform,
    motion: None,
    zone_sphere: None,
  }
}

/// `CCustomZone::PlayIdleParticles`: a zone whose section names idle particles plays them at its `XFORM`, a torrid
/// zone moving it along its motion, and a campfire switches between them and its own.
fn place_zone(object: &AlifeObject, sections: &Ltx) -> Option<RenderParticlePlacement> {
  let (motion, zone): (Option<&AlifeObjectMotion>, &AlifeObjectCustomZone) = match &object.inherited {
    AlifeObjectInherited::CseAlifeAnomalousZone(zone) => (None, &zone.base.base),
    AlifeObjectInherited::CseAlifeZoneVisual(zone) => (None, &zone.base.base),
    AlifeObjectInherited::CseAlifeTorridZone(zone) => (Some(&zone.motion), &zone.base),
    _ => return None,
  };

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
    read("class").as_deref().and_then(ClsId::from_class_name) == Some(ClsId::ZCFire),
    read("disabled_particles"),
    read("enabling_particles"),
  ) {
    (true, Some(disabled), Some(enabling)) => RenderParticleSource::Campfire {
      id: object.id,
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
    motion: motion.and_then(AlifeObjectMotion::get_name).map(str::to_owned),
    zone_sphere: ZoneSphere::of_custom_zone(zone, (&object.position, &object.direction)),
  })
}

/// Every effect a placement or an ambient can play: the effects they name, each effect of the groups they name, and the
/// children those effects start.
fn list_reached_effects(
  library: &ParticleLibrary,
  placements: &[RenderParticlePlacement],
  ambient_particles: &[String],
) -> BTreeSet<String> {
  let mut reached: BTreeSet<String> = BTreeSet::new();
  let names = placements
    .iter()
    .flat_map(|placement| match &placement.source {
      RenderParticleSource::Static { name } => vec![name.as_str()],
      RenderParticleSource::Zone { idle } => vec![idle.as_str()],
      RenderParticleSource::Campfire {
        idle,
        disabled,
        enabling,
        ..
      } => vec![idle.as_str(), disabled.as_str(), enabling.as_str()],
    })
    .chain(ambient_particles.iter().map(String::as_str));

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
