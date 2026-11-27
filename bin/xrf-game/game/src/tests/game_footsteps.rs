use xrf_game_movement::{MovementStep, WalkPace};
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial, GameMtlPair};
use xrf_math::glam::Vec3;
use xrf_world::{WorldSoundCue, WorldSoundKind};

use crate::sound::game_footsteps::GameFootsteps;

const ACTOR: u16 = 1;
const DIRT: u16 = 2;
const GLASS: u16 = 3;

/// A library where the actor steps on dirt with two sounds and makes no pair with glass.
fn create_materials() -> GameMtlFile {
  let mut dirt: GameMtlPair = GameMtlPair::new(0, u32::from(ACTOR), u32::from(DIRT));

  dirt.step_sounds = String::from("material\\dirt\\step\\dirt1,material\\dirt\\step\\dirt2");

  GameMtlFile::new(
    vec![
      GameMtlMaterial::new(u32::from(ACTOR), "creatures\\actor"),
      GameMtlMaterial::new(u32::from(DIRT), "materials\\dirt"),
      GameMtlMaterial::new(u32::from(GLASS), "materials\\glass"),
    ],
    vec![dirt],
  )
}

fn step(x: f32, ground: u16, pace: WalkPace) -> MovementStep {
  MovementStep {
    feet: Vec3::new(x, 0.0, 0.0),
    ground_material: ground,
    pace,
  }
}

#[test]
fn plays_each_step_from_the_listener_never_the_same_sound_twice_running() {
  let mut footsteps: GameFootsteps = GameFootsteps::of_materials(&create_materials(), 0.21, 7);
  let steps: Vec<WorldSoundCue> = (0..6)
    .filter_map(|index| footsteps.hear(&step(index as f32, DIRT, WalkPace::Run)))
    .collect();

  assert_eq!(steps.len(), 6);
  assert!(
    steps
      .iter()
      .all(|step| step.is_2d && step.kind == WorldSoundKind::Footstep)
  );
  assert!((steps[0].volume - 0.9 * 0.21).abs() < 1e-6);
  assert_eq!(steps[3].position, [3.0, 0.0, 0.0]);
  assert!(steps.windows(2).all(|pair| pair[0].reference != pair[1].reference));
  assert!(
    steps
      .iter()
      .all(|step| step.reference.starts_with("material\\dirt\\step\\"))
  );
}

#[test]
fn walking_steps_softer_than_sprinting() {
  let mut footsteps: GameFootsteps = GameFootsteps::of_materials(&create_materials(), 1.0, 7);
  let walked: WorldSoundCue = footsteps.hear(&step(0.0, DIRT, WalkPace::Walk)).expect("a step");
  let sprinted: WorldSoundCue = footsteps.hear(&step(0.0, DIRT, WalkPace::Sprint)).expect("a step");

  assert!(walked.volume < sprinted.volume);
}

#[test]
fn plays_nothing_on_a_ground_with_no_step_sounds() {
  let mut footsteps: GameFootsteps = GameFootsteps::of_materials(&create_materials(), 1.0, 7);

  assert!(footsteps.hear(&step(0.0, GLASS, WalkPace::Run)).is_none());
}
