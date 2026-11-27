use xrf_game_movement::{MovementFooting, WalkPace};
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

fn footing(x: f32, ground: Option<u16>, pace: WalkPace) -> MovementFooting {
  MovementFooting {
    feet: Vec3::new(x, 0.0, 0.0),
    ground_material: ground,
    pace,
    speed: 2.77,
  }
}

/// Walks the feet from 0 to `distance` metres in steps of a tenth, gathering what they play.
fn walk(footsteps: &mut GameFootsteps, distance: f32, ground: Option<u16>, pace: WalkPace) -> Vec<WorldSoundCue> {
  (0..=(distance * 10.0).round() as u32)
    .filter_map(|tenth| footsteps.hear(Some(&footing(tenth as f32 / 10.0, ground, pace))))
    .collect()
}

#[test]
fn plays_a_step_each_stride_from_the_listener_never_the_same_sound_twice_running() {
  let mut footsteps: GameFootsteps = GameFootsteps::of_materials(&create_materials(), 0.21, 7);
  let steps: Vec<WorldSoundCue> = walk(&mut footsteps, 4.5, Some(DIRT), WalkPace::Run);

  assert_eq!(steps.len(), 4, "a running stride is a metre");
  assert!(
    steps
      .iter()
      .all(|step| step.is_2d && step.kind == WorldSoundKind::Footstep)
  );
  assert!((steps[0].volume - 0.9 * 0.21).abs() < 1e-6);
  assert!(steps.windows(2).all(|pair| pair[0].reference != pair[1].reference));
  assert!(
    steps
      .iter()
      .all(|step| step.reference.starts_with("material\\dirt\\step\\"))
  );
}

#[test]
fn walking_steps_shorter_and_softer_than_sprinting() {
  let materials: GameMtlFile = create_materials();
  let walked: Vec<WorldSoundCue> = walk(
    &mut GameFootsteps::of_materials(&materials, 1.0, 7),
    6.8,
    Some(DIRT),
    WalkPace::Walk,
  );
  let sprinted: Vec<WorldSoundCue> = walk(
    &mut GameFootsteps::of_materials(&materials, 1.0, 7),
    6.8,
    Some(DIRT),
    WalkPace::Sprint,
  );

  assert_eq!(walked.len(), 10);
  assert_eq!(sprinted.len(), 4);
  assert!(walked[0].volume < sprinted[0].volume);
}

#[test]
fn plays_nothing_in_the_air_standing_flying_or_on_a_ground_with_no_step_sounds() {
  let mut footsteps: GameFootsteps = GameFootsteps::of_materials(&create_materials(), 1.0, 7);

  assert!(walk(&mut footsteps, 4.0, None, WalkPace::Run).is_empty());
  assert!(walk(&mut footsteps, 4.0, Some(GLASS), WalkPace::Run).is_empty());
  assert!(footsteps.hear(None).is_none());

  let standing: MovementFooting = MovementFooting {
    speed: 0.0,
    ..footing(0.0, Some(DIRT), WalkPace::Run)
  };

  assert!((0..100).all(|_| footsteps.hear(Some(&standing)).is_none()));
}
