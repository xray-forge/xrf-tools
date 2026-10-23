//! Verifies the object motions and post-process effects a project holds under `anims\`, through the animations check.

use std::fs;
use std::path::PathBuf;

use xrf_animation_envelope::{AnimationEnvelope, AnimationKey, AnimationShape};
use xrf_anm::{AnmChannel, AnmFile};
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_ppe::PpeFile;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::{
  GamedataCheckResult, GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions,
  GamedataVerificationStatus,
};

/// A two-second motion along x, as `COMotion::SaveMotion` frames one.
fn new_motion_bytes(value: f32) -> Vec<u8> {
  let line: AnimationEnvelope = AnimationEnvelope {
    keys: vec![
      AnimationKey {
        shape: AnimationShape::Linear,
        ..AnimationKey::default()
      },
      AnimationKey {
        value,
        time: 2.0,
        shape: AnimationShape::Linear,
        ..AnimationKey::default()
      },
    ],
    ..AnimationEnvelope::default()
  };
  let mut channels: [AnimationEnvelope; 6] = Default::default();

  channels[AnmChannel::PositionX.get_index()] = line;

  let motion: AnmFile = AnmFile {
    name: String::new(),
    frame_start: 0,
    frame_end: 60,
    fps: 30.0,
    version: AnmFile::CURRENT_VERSION,
    channels,
  };

  motion.write_to_bytes::<XRayByteOrder>().unwrap()
}

/// A version 1 effect whose blur is keyed once at `value`.
fn new_effect_bytes(value: f32) -> Vec<u8> {
  let mut effect: PpeFile = PpeFile::default();

  effect.blur.keys.push(AnimationKey {
    value,
    ..AnimationKey::default()
  });

  write_effect(&effect)
}

/// A version 1 effect whose noise granularity falls from 1 to -1 over two seconds, passing zero at one.
fn new_falling_grain_effect_bytes() -> Vec<u8> {
  let key = |time: f32, value: f32| AnimationKey {
    value,
    time,
    shape: AnimationShape::Linear,
    ..AnimationKey::default()
  };
  let mut effect: PpeFile = PpeFile::default();

  effect.noise_granularity.keys = vec![key(0.0, 1.0), key(2.0, -1.0)];

  write_effect(&effect)
}

fn write_effect(effect: &PpeFile) -> Vec<u8> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  effect.write::<XRayByteOrder>(&mut writer).unwrap();

  writer.buffer
}

fn open_project(name: &str, files: &[(&str, Vec<u8>)]) -> (PathBuf, GamedataProject) {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("animations/{name}"));

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(root.join("configs")).unwrap();
  fs::create_dir_all(root.join("anims")).unwrap();
  fs::write(root.join("configs").join("system.ltx"), "").unwrap();

  for (path, bytes) in files {
    fs::write(root.join("anims").join(path), bytes).unwrap();
  }

  let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
    root: root.clone(),
    output: xrf_output::OutputOptions::default(),
    ..Default::default()
  })
  .unwrap();

  (root, project)
}

#[test]
fn passes_motions_and_effects_that_read_write_back_and_evaluate() {
  let (root, project) = open_project(
    "valid",
    &[("line.anm", new_motion_bytes(6.0)), ("blur.ppe", new_effect_bytes(0.5))],
  );
  let result = project
    .verify_animations(&GamedataProjectVerifyOptions::default())
    .unwrap();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
  assert!(
    result
      .get_failure_message()
      .ends_with("1/1 object motions valid, 1/1 post-process effects valid"),
    "{}",
    result.get_failure_message()
  );

  fs::remove_dir_all(root).unwrap();
}

#[test]
fn reports_each_way_a_motion_or_effect_is_invalid_under_its_own_rule() {
  let mut extra_chunk: Vec<u8> = new_motion_bytes(6.0);

  // A chunk the engine's `find_chunk` steps over, which the writer therefore does not keep.
  extra_chunk.extend_from_slice(&0x1200_u32.to_le_bytes());
  extra_chunk.extend_from_slice(&2_u32.to_le_bytes());
  extra_chunk.extend_from_slice(&[0xde, 0xad]);

  let truncated: Vec<u8> = new_motion_bytes(6.0)[..20].to_vec();
  let (root, project) = open_project(
    "invalid",
    &[
      ("extra.anm", extra_chunk),
      ("nan.anm", new_motion_bytes(f32::NAN)),
      ("short.anm", truncated),
      ("infinite.ppe", new_effect_bytes(f32::INFINITY)),
      ("grain.ppe", new_falling_grain_effect_bytes()),
    ],
  );
  let result = project
    .verify_animations(&GamedataProjectVerifyOptions::default())
    .unwrap();
  let findings: Vec<(String, String)> = result
    .get_findings()
    .iter()
    .map(|finding| {
      (
        finding.subject().unwrap_or_default().to_owned(),
        finding.rule_id().to_string(),
      )
    })
    .collect();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert!(
    result
      .get_failure_message()
      .ends_with("0/3 object motions valid, 0/2 post-process effects valid"),
    "{}",
    result.get_failure_message()
  );
  assert_eq!(
    findings,
    [
      ("anims/extra.anm", "animations.object-motion-round-trip"),
      ("anims/nan.anm", "animations.object-motion-value"),
      ("anims/short.anm", "animations.object-motion-read"),
      ("anims/grain.ppe", "animations.post-process-noise-granularity"),
      ("anims/infinite.ppe", "animations.post-process-value"),
    ]
    .map(|(path, rule)| (path.to_owned(), rule.to_owned()))
  );
  assert_eq!(
    result.get_findings()[1].message(),
    "Channel 'position x' evaluates to NaN at time 1.000"
  );
  assert_eq!(
    result.get_findings()[3].message(),
    "Noise granularity reaches 0.000 at time 1.000, which the engine refuses at full strength"
  );

  fs::remove_dir_all(root).unwrap();
}
