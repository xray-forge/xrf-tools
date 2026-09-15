use std::collections::HashMap;

use serde::Serialize;
use xrf_db::{ParticleActionType, ParticleEffect};
use xrf_vfs::XrayAssetType;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;

/// One emitter of the library: what it draws with, how much of it, and what moves it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveParticlesEffect {
  pub name: String,
  /// Particles the emitter may hold at once, `max_particles`.
  pub max_particles: u32,
  /// Seconds the emitter runs for, absent where it declares no limit and runs until something stops it.
  pub time_limit: Option<f32>,
  /// The blender the sprite is drawn with, which names a definition of `shaders.xr` rather than a file.
  pub shader: String,
  /// The texture the sprite is drawn from, resolved against the subject being browsed.
  pub texture: ArchiveReference,
  /// Every action kind the effect is built from, in the order it first uses each.
  pub actions: Vec<String>,
  /// Actions in total, which is more than the kinds above when one kind is used twice.
  pub actions_count: usize,
  pub flags: u32,
}

impl ArchiveParticlesEffect {
  /// Every effect of a library, each with its texture resolved.
  pub fn of_all(source: &ArchiveDescribeSource, effects: &[ParticleEffect]) -> Vec<Self> {
    let mut resolved: HashMap<&str, ArchiveReference> = HashMap::new();

    effects
      .iter()
      .map(|effect| Self {
        name: effect.name.clone(),
        max_particles: effect.max_particles,
        time_limit: effect.time_limit,
        shader: effect.sprite.shader_name.clone(),
        texture: resolved
          .entry(effect.sprite.texture_name.as_str())
          .or_insert_with(|| ArchiveReference::resolve(source, XrayAssetType::Dds, &effect.sprite.texture_name))
          .clone(),
        actions: Self::to_action_kinds(effect),
        actions_count: effect.actions.len(),
        flags: effect.flags,
      })
      .collect()
  }

  /// The action kinds an effect holds, each named once, in first use order.
  fn to_action_kinds(effect: &ParticleEffect) -> Vec<String> {
    let mut kinds: Vec<String> = Vec::new();

    for action in &effect.actions {
      let kind: String = ParticleActionType::get_action_type(action).to_string();

      if !kinds.contains(&kind) {
        kinds.push(kind);
      }
    }

    kinds
  }
}
