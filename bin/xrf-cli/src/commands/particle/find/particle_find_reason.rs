use serde::Serialize;
use xrf_particles::{ParticleActionType, ParticleGroupChild};

/// One way an effect or group matched a query, with the query's value as the library spells it.
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum ParticleFindReason {
  /// An effect's sprite samples the texture.
  Texture(String),
  /// An effect's sprite draws with the shader.
  Shader(String),
  /// An effect runs an action of the type.
  Action(ParticleActionType),
  /// A group plays the effect.
  Effect(String),
  /// A group spawns the effect as an effect of it starts playing.
  PlayChild(String),
  /// A group spawns the effect at each birth of one of its effects' particles.
  BirthChild(String),
  /// A group spawns the effect at each death of one of its effects' particles.
  DeathChild(String),
}

impl ParticleFindReason {
  /// A group spawning the effect as a child, by when it spawns it.
  pub fn of_child(child: ParticleGroupChild, effect: &str) -> Self {
    match child {
      ParticleGroupChild::Play => Self::PlayChild(effect.to_owned()),
      ParticleGroupChild::Birth => Self::BirthChild(effect.to_owned()),
      ParticleGroupChild::Death => Self::DeathChild(effect.to_owned()),
    }
  }

  /// The reason as text: its kind and value.
  pub fn describe(&self) -> String {
    match self {
      Self::Texture(texture) => format!("texture {texture}"),
      Self::Shader(shader) => format!("shader {shader}"),
      Self::Action(action) => format!("action {action}"),
      Self::Effect(effect) => format!("effect {effect}"),
      Self::PlayChild(effect) => format!("play child {effect}"),
      Self::BirthChild(effect) => format!("birth child {effect}"),
      Self::DeathChild(effect) => format!("death child {effect}"),
    }
  }
}
