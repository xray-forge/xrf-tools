//! Editing a library as the SDK's particle editor does: adding, renaming and removing effects and groups, and the
//! faults it refuses to save over.

use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;

use crate::{
  ParticleActionType, ParticleEditorAction, ParticleEffect, ParticleGroup, ParticleGroupChild, ParticleGroupEffect,
  ParticleGroupEffectFlags, ParticlesEffectsChunk, ParticlesFile, ParticlesFileIssue, ParticlesGroupsChunk,
  ParticlesHeaderChunk,
};

fn effect(name: &str) -> XrfResult<ParticleEffect> {
  let mut effect: ParticleEffect = ParticleEffect {
    version: ParticleEffect::VERSION,
    name: String::from(name),
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

  effect.set_editor_actions::<XRayByteOrder>(&[ParticleEditorAction::new(ParticleActionType::KillOld)?])?;

  Ok(effect)
}

fn group_effect(name: &str, flags: u32) -> ParticleGroupEffect {
  ParticleGroupEffect {
    name: String::from(name),
    on_play_child_name: String::new(),
    on_birth_child_name: String::new(),
    on_dead_child_name: String::new(),
    time_0: 0.0,
    time_1: 1.0,
    flags,
  }
}

fn group(name: &str, effects: Vec<ParticleGroupEffect>) -> ParticleGroup {
  ParticleGroup {
    version: ParticleGroup::VERSION,
    name: String::from(name),
    flags: 0,
    time_limit: 0.0,
    effects,
    description: None,
    effects_old: None,
  }
}

fn library() -> ParticlesFile {
  ParticlesFile {
    header: ParticlesHeaderChunk { version: 1 },
    effects: ParticlesEffectsChunk::default(),
    groups: ParticlesGroupsChunk::default(),
  }
}

#[test]
fn adds_effects_where_their_names_sort() -> XrfResult {
  let mut file: ParticlesFile = library();

  file.insert_effect(effect("fx\\smoke")?)?;
  file.insert_effect(effect("fx\\flame")?)?;
  file.insert_effect(effect("fx\\sparks")?)?;

  assert_eq!(
    file
      .effects
      .effects
      .iter()
      .map(|it| it.name.as_str())
      .collect::<Vec<_>>(),
    ["fx\\flame", "fx\\smoke", "fx\\sparks"]
  );
  assert_eq!(file.effects.get("fx\\smoke").map(|it| it.max_particles), Some(8));

  Ok(())
}

#[test]
fn keeps_names_unique_across_effects_and_groups() -> XrfResult {
  let mut file: ParticlesFile = library();

  file.insert_effect(effect("fx\\smoke")?)?;

  assert!(file.insert_group(group("fx\\smoke", Vec::new())).is_err());
  assert!(file.insert_effect(effect("fx\\smoke")?).is_err());

  Ok(())
}

#[test]
fn renames_an_effect_and_every_group_reference_to_it() -> XrfResult {
  let mut file: ParticlesFile = library();
  let mut spawning: ParticleGroupEffect = group_effect("fx\\flame", ParticleGroupEffectFlags::ENABLED);

  spawning.on_dead_child_name = String::from("fx\\smoke");

  file.insert_effect(effect("fx\\smoke")?)?;
  file.insert_effect(effect("fx\\flame")?)?;
  file.insert_group(group(
    "fire",
    vec![group_effect("fx\\smoke", ParticleGroupEffectFlags::ENABLED), spawning],
  ))?;

  file.rename_effect("fx\\smoke", "fx\\ash")?;

  assert!(file.effects.get("fx\\smoke").is_none());
  assert_eq!(file.effects.effects[0].name, "fx\\ash");

  let fire: &ParticleGroup = file.groups.get("fire").expect("the group is kept");

  assert_eq!(fire.effects[0].name, "fx\\ash");
  assert_eq!(fire.effects[1].on_dead_child_name, "fx\\ash");

  Ok(())
}

#[test]
fn refuses_to_rename_onto_a_name_in_use() -> XrfResult {
  let mut file: ParticlesFile = library();

  file.insert_effect(effect("fx\\smoke")?)?;
  file.insert_group(group("fire", Vec::new()))?;

  assert!(file.rename_effect("fx\\smoke", "fire").is_err());
  assert!(file.rename_group("fire", "fx\\smoke").is_err());
  assert!(file.rename_group("water", "rain").is_err());

  file.rename_group("fire", "blaze")?;

  assert!(file.groups.get("blaze").is_some());

  Ok(())
}

#[test]
fn removes_by_exact_name() -> XrfResult {
  let mut file: ParticlesFile = library();

  file.insert_effect(effect("fx\\smoke")?)?;

  assert!(file.effects.remove("FX\\SMOKE").is_none());
  assert!(file.effects.remove("fx\\smoke").is_some());
  assert!(file.effects.effects.is_empty());

  Ok(())
}

#[test]
fn finds_the_faults_the_sdk_refuses_to_save() -> XrfResult {
  let mut file: ParticlesFile = library();
  let mut endless: ParticleEffect = effect("fx\\endless")?;

  endless.set_editor_actions::<XRayByteOrder>(&[ParticleEditorAction::new(ParticleActionType::Move)?])?;

  file.insert_effect(effect("fx\\smoke")?)?;
  file.insert_effect(endless)?;
  file.insert_group(group(
    "fire",
    vec![
      group_effect("fx\\gone", ParticleGroupEffectFlags::ENABLED),
      group_effect(
        "fx\\smoke",
        ParticleGroupEffectFlags::ENABLED | ParticleGroupEffectFlags::ON_BIRTH_CHILD,
      ),
    ],
  ))?;

  assert_eq!(
    file.validate::<XRayByteOrder>(),
    [
      ParticlesFileIssue::MissingKillOld {
        effect: String::from("fx\\endless")
      },
      ParticlesFileIssue::UndefinedEffect {
        group: String::from("fire"),
        effect: String::from("fx\\gone")
      },
      ParticlesFileIssue::UnnamedChild {
        group: String::from("fire"),
        effect: String::from("fx\\smoke"),
        child: ParticleGroupChild::Birth
      },
    ]
  );

  Ok(())
}

#[test]
fn plays_a_group_to_its_latest_effect_where_it_stores_no_limit() {
  let mut effects: Vec<ParticleGroupEffect> = vec![group_effect("a", 0), group_effect("b", 0)];

  effects[1].time_1 = 3.5;

  let mut fire: ParticleGroup = group("fire", effects);

  assert_eq!(fire.get_time_limit(), 3.5);

  fire.time_limit = 2.0;

  assert_eq!(fire.get_time_limit(), 2.0);
}
