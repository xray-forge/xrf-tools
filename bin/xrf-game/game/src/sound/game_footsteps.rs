use std::collections::HashMap;

use xrf_game_movement::{MovementStep, WalkPace};
use xrf_gamemtl::{GameMtlFile, GameMtlPair};
use xrf_random::{RandomAddress, RandomStream};
use xrf_world::{WorldSoundCue, WorldSoundKind};

use crate::sound::game_last_step::GameLastStep;

/// The player's footsteps as presentation plays them, one for each step event the game loop tells (movement counts
/// the strides): its sound drawn from the step sounds of the pair the actor's material makes with the ground's
/// (`gamemtl.xr`), never the same one twice running, and played from the listener, as X-Ray plays its first-person
/// actor's steps (`CStepManager`, `step_manager.cpp`).
pub struct GameFootsteps {
  /// The step sounds on each ground material, by its id; a material the actor makes no pair with has none.
  sounds: HashMap<u16, Vec<String>>,
  /// How loud a first-person step plays against its power (`[hud_sound] hud_step_sound_vol_k`).
  volume: f32,
  random: RandomStream,
  last: Option<GameLastStep>,
}

impl GameFootsteps {
  /// The material X-Ray's actor is made of, which its steps pair with the ground's.
  const ACTOR_MATERIAL: &'static str = "creatures\\actor";

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
    }
  }

  /// The sound of `step`, none on a ground with no step sounds.
  pub fn hear(&mut self, step: &MovementStep) -> Option<WorldSoundCue> {
    let reference: String = self.pick(step.ground_material)?;

    Some(WorldSoundCue {
      reference,
      kind: WorldSoundKind::Footstep,
      // A 2D sound's place is from the listener, not in the level: at the listener itself the step plays centred.
      position: [0.0; 3],
      is_2d: true,
      volume: Self::get_power(step.pace) * self.volume,
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
