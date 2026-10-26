//! The SDK editor's actions: how they are stored, edited and compiled into what the engine plays.
//!
//! Stored layouts are framed by hand from `EParticleAction::Save`, so the reader cannot pass by agreeing with its writer.

use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::{
  ParticleAction, ParticleActionSource, ParticleActionType, ParticleEditorAction, ParticleEditorDomain,
  ParticleEditorSchema, ParticleEditorValue, ParticleEffect, ParticleEffectEditorData,
};

fn floats(values: &[f32]) -> Vec<u8> {
  values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

/// A bounce as the SDK stores it: floats by name, then the domain, then the bool.
fn stored_bounce() -> Vec<u8> {
  [
    &(ParticleActionType::Bounce as u32).to_le_bytes()[..],
    &1u32.to_le_bytes(),
    b"floor\0",
    &ParticleEditorAction::FLAG_ENABLED.to_le_bytes(),
    // `Cutoff`, `Friction`, `Resilience`: sorted by name, whatever order the SDK shows them in.
    &floats(&[2.0, 0.25, 0.5]),
    // `Position`: a plane through the origin facing up.
    &3u32.to_le_bytes(),
    &floats(&[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]),
    // `Allow Rotate`.
    &[1],
  ]
  .concat()
}

fn read_action(bytes: &[u8]) -> XrfResult<ParticleEditorAction> {
  ParticleEditorAction::read::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(bytes)?)
}

#[test]
fn starts_a_new_action_where_the_sdk_does() -> XrfResult {
  let action: ParticleEditorAction = ParticleEditorAction::new(ParticleActionType::KillOld)?;

  assert_eq!(action.name, "KillOld");
  assert_eq!(action.version, ParticleEditorSchema::VERSION);
  assert!(action.is_enabled());
  assert_eq!(action.get_float("Age Limit")?, 5.0);
  assert!(!action.get_bool("Kill Less Than")?);

  Ok(())
}

#[test]
fn reads_properties_in_the_order_the_sdk_stores_them() -> XrfResult {
  let action: ParticleEditorAction = read_action(&stored_bounce())?;

  assert_eq!(action.name, "floor");
  assert_eq!(action.get_float("Friction")?, 0.25);
  assert_eq!(action.get_float("Resilience")?, 0.5);
  assert_eq!(action.get_float("Cutoff")?, 2.0);
  assert_eq!(action.get_domain("Position")?.domain_type, 3);
  assert!(action.get_bool("Allow Rotate")?);

  // Kept in the order the SDK shows them, which is not the stored one.
  assert_eq!(
    action.properties.iter().map(|it| it.name.as_str()).collect::<Vec<_>>(),
    ["Position", "Friction", "Resilience", "Cutoff", "Allow Rotate"]
  );

  Ok(())
}

#[test]
fn writes_an_action_back_byte_for_byte() -> XrfResult {
  let mut writer: ChunkWriter = ChunkWriter::new();

  read_action(&stored_bounce())?.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.flush_raw_into_buffer()?, stored_bounce());

  Ok(())
}

#[test]
fn compiles_a_bounce_as_pbounce_does() -> XrfResult {
  let ParticleAction::Bounce(bounce) = read_action(&stored_bounce())?.compile()? else {
    panic!("expect a bounce to compile to a bounce");
  };

  assert_eq!(bounce.one_minus_friction, 0.75);
  assert_eq!(bounce.cutoff_sqr, 4.0);
  assert_eq!(bounce.resilience, 0.5);
  assert_eq!(bounce.action_flags, ParticleAction::ALLOW_ROTATE);
  assert_eq!(bounce.position.coordinates.1, Vector3d::Y);

  Ok(())
}

#[test]
fn compiles_a_source_with_the_flags_psource_sets() -> XrfResult {
  let mut source: ParticleEditorAction = ParticleEditorAction::new(ParticleActionType::Source)?;

  source.set("Single Size", ParticleEditorValue::Bool(true))?;
  source.set("Rate", ParticleEditorValue::Float(25.0))?;

  let ParticleAction::Source(compiled) = source.compile()? else {
    panic!("expect a source to compile to a source");
  };

  // A new source does not turn with its placement, but always tracks its secondary position.
  assert_eq!(
    compiled.action_flags,
    ParticleActionSource::VERTEX_B_TRACKS | ParticleActionSource::SINGLE_SIZE
  );
  assert_eq!(compiled.particle_rate, 25.0);
  assert_eq!(compiled.color.coordinates.0, Vector3d::new(1.0, 1.0, 1.0));

  Ok(())
}

#[test]
fn compiles_a_shadow_of_chernobyl_target_colour_over_its_whole_life() -> XrfResult {
  // Version 0 has no `TimeFrom` or `TimeTo`, which the later SDK starts at 0 and 1.
  let stored: Vec<u8> = [
    &(ParticleActionType::TargetColor as u32).to_le_bytes()[..],
    &ParticleEditorSchema::SHADOW_OF_CHERNOBYL_VERSION.to_le_bytes(),
    b"tint\0",
    &ParticleEditorAction::FLAG_ENABLED.to_le_bytes(),
    // `Alpha`, `Scale`, then the `Color` vector.
    &floats(&[0.5, 2.0, 1.0, 0.0, 0.0]),
  ]
  .concat();

  let ParticleAction::TargetColor(compiled) = read_action(&stored)?.compile()? else {
    panic!("expect a target colour to compile to a target colour");
  };

  assert_eq!((compiled.time_from, compiled.time_to), (0.0, 1.0));
  assert_eq!(compiled.color, Vector3d::X);
  assert_eq!((compiled.alpha, compiled.scale), (0.5, 2.0));

  Ok(())
}

#[test]
fn refuses_an_action_of_a_version_no_sdk_wrote() {
  let mut stored: Vec<u8> = stored_bounce();

  stored[4..8].copy_from_slice(&2u32.to_le_bytes());

  assert!(read_action(&stored).is_err());
}

#[test]
fn refuses_a_property_of_another_kind_or_name() -> XrfResult {
  let mut action: ParticleEditorAction = ParticleEditorAction::new(ParticleActionType::Gravity)?;

  assert!(action.set("Direction", ParticleEditorValue::Float(1.0)).is_err());
  assert!(action.set("Strength", ParticleEditorValue::Float(1.0)).is_err());
  assert!(action.get_float("Direction").is_err());

  action.set("Direction", ParticleEditorValue::Vector(Vector3d::new(0.0, -1.0, 0.0)))?;

  assert_eq!(action.get_vector("Direction")?, &Vector3d::new(0.0, -1.0, 0.0));

  Ok(())
}

#[test]
fn saves_an_effects_editor_actions_and_compiles_the_enabled_ones() -> XrfResult {
  let mut effect: ParticleEffect = ParticleEffect {
    version: ParticleEffect::VERSION,
    name: String::from("fx"),
    max_particles: 8,
    actions: Vec::new(),
    flags: 0,
    frame: None,
    sprite: None,
    time_limit: None,
    collision: None,
    velocity_scale: None,
    description: None,
    rotation: None,
    editor_data: None,
  };
  let mut disabled: ParticleEditorAction = ParticleEditorAction::new(ParticleActionType::Move)?;

  disabled.flags &= !ParticleEditorAction::FLAG_ENABLED;

  let actions: Vec<ParticleEditorAction> = vec![
    ParticleEditorAction::new(ParticleActionType::KillOld)?,
    disabled,
    ParticleEditorAction::new(ParticleActionType::Move)?,
  ];

  effect.set_editor_actions::<XRayByteOrder>(&actions)?;

  assert_eq!(
    effect
      .actions
      .iter()
      .map(ParticleActionType::get_action_type)
      .collect::<Vec<_>>(),
    [ParticleActionType::KillOld, ParticleActionType::Move]
  );
  assert_eq!(effect.read_editor_actions::<XRayByteOrder>()?, Some(actions));

  Ok(())
}

#[test]
fn keeps_editor_data_that_does_not_decode_as_it_is() -> XrfResult {
  // The engine never reads editor data, so a library carrying a broken copy still plays; it decodes only on request.
  let editor_data: ParticleEffectEditorData = ParticleEffectEditorData {
    value: vec![1, 0, 0, 0, 99],
  };

  assert!(editor_data.read_actions::<XRayByteOrder>().is_err());

  let mut writer: ChunkWriter = ChunkWriter::new();

  editor_data.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.flush_raw_into_buffer()?, editor_data.value);

  Ok(())
}

#[test]
fn compiles_a_disc_domain_as_pdomain_does() {
  let disc: ParticleEditorDomain = ParticleEditorDomain {
    domain_type: 9,
    points: [
      Vector3d::new(0.0, 1.0, 0.0),
      Vector3d::new(0.0, 0.0, 2.0),
      Vector3d::new(0.5, 1.5, 0.0),
    ],
  };

  let compiled = disc.compile();

  assert_eq!(compiled.coordinates.1, Vector3d::Z);
  assert_eq!((compiled.radius1, compiled.radius2), (1.5, 0.5));
  assert_eq!(compiled.basis.0, Vector3d::X);
  assert_eq!(compiled.basis.1, Vector3d::Y);
}
