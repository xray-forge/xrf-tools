use std::collections::{BTreeSet, HashMap};
use std::time::Instant;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelPsStaticFile, LevelPsStaticPlacement};
use xrf_ltx::{Ltx, Section};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};
use xrf_particles::{ParticleLibrary, ParticlesFile};
use xrf_renderer::{RenderParticlePlacement, RenderParticleSource};
use xrf_spawn::{AlifeObject, AlifeObjectCustomZone, AlifeObjectInherited, AlifeObjectMotion, ClsId};
use xrf_vfs::XraySearch;
use xrf_visual::{VisualTransform, ZoneSphere};

use crate::level::{LevelSource, SelectedLevel};
use crate::parts::LevelTextureReference;
use crate::parts::{LevelEnvironmentPart, LevelSpawnPart, PackedLevelParticles};
use crate::render::LevelRenderAmbients;
use crate::report::LevelReport;

/// What a change event calls what a level's particle systems were read from when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_PARTICLES_PART: &str = "particles";

/// What a level's particle systems were read from: the game's library, `level.ps_static`, and the shaders and textures
/// their effects draw with, with the parts they ask for; the renderer holds the systems themselves.
pub(crate) struct LevelParticlesPart;

impl SessionPart for LevelParticlesPart {
  type Value = ();

  const NAME: &'static str = LEVEL_PARTICLES_PART;

  const IS_POOLED: bool = false;
}

impl LevelParticlesPart {
  /// `eGameIDSingle`: the game type a planted system must name to play in a single player game.
  const SINGLE_PLAYER: u16 = 1;

  /// The height `CLevel::Load_GameSpecific_After` lifts every planted system by.
  const PLANTED_LIFT: f32 = 0.01;

  /// Reads the game's particle library and places what the open level plants and what its zones play, each effect any
  /// of them reaches described, what it reads recorded as the level's particles part.
  ///
  /// # Errors
  ///
  /// Returns why the library cannot be read; a level without `level.ps_static` or a spawn plants nothing of it.
  pub fn pack(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    sections: Option<&Ltx>,
  ) -> Result<PackedLevelParticles, String> {
    current
      .parts
      .record_read::<Self, _>(search, |search| Self::pack_recorded(current, search, sections))
  }

  /// One system `level.ps_static` plants, lifted off the ground it stands on.
  pub fn place_planted(placement: &LevelPsStaticPlacement) -> RenderParticlePlacement {
    let mut transform: [f32; 16] = placement.transform.values;

    // The `Fmatrix` rows are the columns of a column-major matrix; the translation is the last.
    transform[13] += Self::PLANTED_LIFT;

    RenderParticlePlacement {
      source: RenderParticleSource::Static {
        name: placement.effect.clone(),
      },
      transform,
      motion: None,
      zone_sphere: None,
    }
  }

  /// The particle systems packed, every read through `search`.
  fn pack_recorded(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    sections: Option<&Ltx>,
  ) -> Result<PackedLevelParticles, String> {
    let started: Instant = Instant::now();
    let library: ParticleLibrary = Self::read_library(search)?;
    let mut placements: Vec<RenderParticlePlacement> = Self::read_planted(&current.source, search)?;

    // todo: Play the systems of space restrictors whose logic starts in `sr_particle` (`name` at `path`'s points, after
    //  each point's delay, `looped` or once), as the b53 steam field's do; condition switches would stay unread.

    // Without the configs no zone names its particles; their reader has said why.
    match (LevelSpawnPart::get(current, search), sections) {
      (Ok(spawn), Some(sections)) => {
        placements.extend(spawn.list_kept().filter_map(|it| Self::place_zone(it, sections)))
      }
      (Err(error), _) => LevelReport::missing_zone_particles(&current.source, &error),
      (Ok(_), None) => {}
    }

    // An ambient effect plays near the camera whatever stands in the level, so its particles are described with the
    // rest; a level whose configs cannot be read has said why with its weather.
    let ambient_particles: Vec<String> = LevelEnvironmentPart::get(current, search)
      .map(|environment| LevelRenderAmbients::list_particles(&environment.catalog))
      .unwrap_or_default();
    let scope: XrayTextureScope = current.source.get_texture_scope();
    let resolver: XraySurfaceResolver = XraySurfaceResolver::open(search, scope.clone());
    let mut surfaces: HashMap<String, XraySurfaceDescriptor> = HashMap::new();
    let mut references: BTreeSet<String> = BTreeSet::new();

    for name in Self::list_reached_effects(&library, &placements, &ambient_particles) {
      // An effect without a sprite draws nothing, so it has no surface to describe.
      let Some(sprite) = library.get_effect(&name).and_then(|effect| effect.sprite.as_ref()) else {
        continue;
      };
      let textures: Vec<String> = sprite.list_textures();

      references.extend(textures.iter().cloned());
      surfaces.insert(name, resolver.describe(&sprite.shader_name, &textures));
    }

    LevelReport::particles(&current.source, &library, placements.len(), surfaces.len(), started);

    Ok(PackedLevelParticles {
      textures: references
        .into_iter()
        .map(|reference| LevelTextureReference::locate(&scope, search, reference))
        .collect(),
      library,
      placements,
      surfaces,
    })
  }

  fn read_library(search: &XraySearch) -> Result<ParticleLibrary, String> {
    search
      .read(ParticlesFile::FILE_NAME)
      .and_then(ParticlesFile::read_from_bytes::<XRayByteOrder>)
      .map(ParticleLibrary::from)
      .map_err(|error| format!("Failed to read '{}': {error}", ParticlesFile::FILE_NAME))
  }

  /// `CLevel::Load_GameSpecific_After`: each system the level plants for a single player game, lifted a centimetre.
  fn read_planted(source: &LevelSource, search: &XraySearch) -> Result<Vec<RenderParticlePlacement>, String> {
    let Some(bytes) = source.read_optional_file(search, LevelPsStaticFile::FILE_NAME)? else {
      return Ok(Vec::new());
    };
    let file: LevelPsStaticFile = LevelPsStaticFile::read_from_bytes::<XRayByteOrder>(bytes)
      .map_err(|error| source.describe_read_failure(LevelPsStaticFile::FILE_NAME, &error))?;

    Ok(
      file
        .placements
        .iter()
        .filter(|placement| {
          placement
            .game_types
            .is_none_or(|types| types & Self::SINGLE_PLAYER != 0)
        })
        .map(Self::place_planted)
        .collect(),
    )
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

  /// Every effect a placement or an ambient can play: the effects they name, each effect of the groups they name, and
  /// the children those effects start.
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
      LevelReport::unknown_particles(&unknown.into_iter().collect::<Vec<&str>>());
    }

    reached
  }
}
