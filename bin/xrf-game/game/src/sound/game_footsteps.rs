use std::collections::HashMap;

use xrf_game_movement::{MovementFooting, WalkPace};
use xrf_gamemtl::{GameMtlFile, GameMtlPair};
use xrf_math::glam::Vec3;
use xrf_random::{RandomAddress, RandomStream};
use xrf_world::{WorldSoundCue, WorldSoundKind};

use crate::sound::game_last_step::GameLastStep;

/// The player's footsteps, heard from the walking actor's footing as the game loop publishes it: a step each stride
/// along the ground, its sound drawn from the step sounds of the pair the actor's material makes with the ground's
/// (`gamemtl.xr`), never the same one twice running, and played from the listener, as X-Ray plays its first-person
/// actor's steps (`CStepManager`, `step_manager.cpp`).
///
/// X-Ray times a step by the leg animation's cycle; with no actor animations yet, the stride of each pace stands in.
pub struct GameFootsteps {
  /// The step sounds on each ground material, by its id; a material the actor makes no pair with has none.
  sounds: HashMap<u16, Vec<String>>,
  /// How loud a first-person step plays against its power (`[hud_sound] hud_step_sound_vol_k`).
  volume: f32,
  random: RandomStream,
  last: Option<GameLastStep>,
  /// Where the feet stood when last heard on the ground; none in the air or flying.
  feet: Option<Vec3>,
  /// How far the feet went along the ground since the last step.
  walked: f32,
}

impl GameFootsteps {
  /// The material X-Ray's actor is made of, which its steps pair with the ground's.
  const ACTOR_MATERIAL: &'static str = "creatures\\actor";

  /// The speed below which the feet count as standing, so the physics' settling plays no step.
  const STANDING_SPEED: f32 = 0.2;

  const RANDOM_DOMAIN: u64 = RandomAddress::domain("game.footsteps");

  /// The footsteps of the actor's material in `materials`, played at `volume` of their power; none where the library
  /// has no actor material.
  pub fn of_materials(materials: &GameMtlFile, volume: f32, seed: u64) -> Self {
    let mut sounds: HashMap<u16, Vec<String>> = HashMap::new();

    if let Some(actor) = materials.find_material_by_name(Self::ACTOR_MATERIAL) {
      for material in &materials.materials {
        let Some(pair) = materials.find_pair(actor.id, material.id) else {
          continue;
        };
        let steps: Vec<String> = GameMtlPair::split_list(&pair.step_sounds)
          .into_iter()
          .map(String::from)
          .collect();

        if let Ok(id) = u16::try_from(material.id)
          && !steps.is_empty()
        {
          sounds.insert(id, steps);
        }
      }
    } else {
      log::warn!(
        "The game materials hold no '{}'; the player's steps play no sound",
        Self::ACTOR_MATERIAL
      );
    }

    Self {
      sounds,
      volume,
      random: RandomStream::new(
        seed,
        RandomAddress {
          domain: Self::RANDOM_DOMAIN,
          ..RandomAddress::default()
        },
      ),
      last: None,
      feet: None,
      walked: 0.0,
    }
  }

  /// The step `footing` takes, if it finishes one: none while flying, in the air or standing, or on a ground with no
  /// step sounds.
  pub fn hear(&mut self, footing: Option<&MovementFooting>) -> Option<WorldSoundCue> {
    let Some(footing) = footing else {
      self.feet = None;
      self.walked = 0.0;

      return None;
    };
    let Some(ground) = footing.ground_material else {
      self.feet = None;

      return None;
    };
    let previous: Option<Vec3> = self.feet.replace(footing.feet);

    if footing.speed < Self::STANDING_SPEED {
      return None;
    }

    if let Some(previous) = previous {
      self.walked += (footing.feet - previous).with_y(0.0).length();
    }

    let stride: f32 = Self::get_stride(footing.pace);

    if self.walked < stride {
      return None;
    }

    // A frame late by more than a stride plays one step, not a burst.
    self.walked = (self.walked - stride).min(stride);

    let reference: String = self.pick(ground)?;

    Some(WorldSoundCue {
      reference,
      kind: WorldSoundKind::Footstep,
      position: footing.feet.to_array(),
      is_2d: true,
      volume: Self::get_power(footing.pace) * self.volume,
    })
  }

  /// A step sound on `ground`: any at first or on new ground, then any but the last, as `material_sound::play_next`
  /// draws.
  fn pick(&mut self, ground: u16) -> Option<String> {
    let sounds: &Vec<String> = self.sounds.get(&ground)?;
    let count: u64 = sounds.len() as u64;
    let sound: usize = match self.last {
      Some(last) if last.ground == ground => ((last.sound as u64 + 1 + self.random.below(count - 1)) % count) as usize,
      _ => self.random.below(count) as usize,
    };

    self.last = Some(GameLastStep { ground, sound });

    Some(sounds[sound].clone())
  }

  /// How far the feet go between two steps at `pace`.
  const fn get_stride(pace: WalkPace) -> f32 {
    match pace {
      WalkPace::Walk => 0.65,
      WalkPace::Run => 1.0,
      WalkPace::Sprint => 1.6,
      WalkPace::Crouch => 0.5,
    }
  }

  /// How hard a step at `pace` lands, the power X-Ray's actor animations give their steps.
  const fn get_power(pace: WalkPace) -> f32 {
    match pace {
      WalkPace::Walk => 0.25,
      WalkPace::Run => 0.9,
      WalkPace::Sprint => 1.3,
      WalkPace::Crouch => 0.1,
    }
  }
}
