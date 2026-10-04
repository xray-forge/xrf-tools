use serde::Serialize;
use xrf_particles::{ParticleActionType, ParticleEffect, ParticleGroup};

/// What `particle list` answers: the effects and groups of a library a filter keeps, effects first, each in the
/// library's order.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleListReport {
  pub entries: Vec<ParticleListEntry>,
}

/// One effect or group of a library, with what tells it apart at a glance.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ParticleListEntry {
  /// An effect: the sprite it draws, how many particles it holds, what acts on them and how long it plays.
  Effect {
    name: String,
    /// The sprite's shader, as `shaders.xr` or a script names it.
    shader: String,
    /// The sprite's textures, the comma-separated list split as the engine splits it: the base first.
    textures: Vec<String>,
    max_particles: u32,
    /// Its actions' types, in the order they run.
    actions: Vec<ParticleActionType>,
    /// Seconds it plays for; none, or a value not above zero, for one that loops.
    time_limit: Option<f32>,
  },
  /// A group: the effects it plays, the children they spawn, and how long it plays.
  Group {
    name: String,
    effects: Vec<String>,
    /// The effects its effects spawn on playing, on a particle's birth or on its death, each once.
    children: Vec<String>,
    /// Seconds it plays for; not above zero for one that loops.
    time_limit: f32,
  },
}

impl ParticleListEntry {
  pub fn of_effect(effect: &ParticleEffect) -> Self {
    Self::Effect {
      name: effect.name.clone(),
      shader: effect.sprite.shader_name.clone(),
      textures: Self::split_textures(&effect.sprite.texture_name),
      max_particles: effect.max_particles,
      actions: effect.actions.iter().map(ParticleActionType::get_action_type).collect(),
      time_limit: effect.time_limit,
    }
  }

  pub fn of_group(group: &ParticleGroup) -> Self {
    let mut children: Vec<String> = Vec::new();

    for (_, child) in group.effects.iter().flat_map(|effect| effect.list_children()) {
      if !children.iter().any(|it| it == child) {
        children.push(child.to_owned());
      }
    }

    Self::Group {
      name: group.name.clone(),
      effects: group.effects.iter().map(|effect| effect.name.clone()).collect(),
      children,
      time_limit: group.time_limit,
    }
  }

  pub fn get_name(&self) -> &str {
    match self {
      Self::Effect { name, .. } | Self::Group { name, .. } => name,
    }
  }

  /// The entry as a line: its kind, name, and what tells it apart.
  pub fn describe(&self) -> String {
    match self {
      Self::Effect {
        name,
        shader,
        textures,
        max_particles,
        actions,
        ..
      } => format!(
        "effect {name} ({shader}, {}, {max_particles} particles, {} actions)",
        if textures.is_empty() {
          String::from("no texture")
        } else {
          textures.join(", ")
        },
        actions.len()
      ),
      Self::Group {
        name,
        effects,
        children,
        ..
      } => format!("group {name} ({} effects, {} children)", effects.len(), children.len()),
    }
  }

  /// `_ParseList`: a sprite's comma-separated texture list, each trimmed, empty ones dropped.
  fn split_textures(list: &str) -> Vec<String> {
    list
      .split(',')
      .map(str::trim)
      .filter(|it| !it.is_empty())
      .map(str::to_owned)
      .collect()
  }
}
