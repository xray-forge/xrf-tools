use std::fmt::{Display, Formatter, Result};

use crate::data::group::particle_group_child::ParticleGroupChild;

/// A fault the SDK's particle editor refuses to save a library over (`CParticleTool::Validate`), as
/// [`crate::ParticlesFile::validate`] finds it.
#[derive(Clone, Debug, PartialEq)]
pub enum ParticlesFileIssue {
  /// `CPEDef::Validate`: the effect has no `KillOld` action, so its particles live as long as it plays.
  MissingKillOld { effect: String },
  /// The effect's editor data does not decode, so the SDK could not open the effect to validate or edit it.
  UnreadableEditorData { effect: String, message: String },
  /// `CPGDef::Validate`: a group plays an effect the library does not define.
  UndefinedEffect { group: String, effect: String },
  /// `CPGDef::Validate`: a group effect spawns a child on an event but names none.
  UnnamedChild {
    group: String,
    effect: String,
    child: ParticleGroupChild,
  },
}

impl Display for ParticlesFileIssue {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
    match self {
      Self::MissingKillOld { effect } => write!(formatter, "Particle effect '{effect}' has no KillOld action"),
      Self::UnreadableEditorData { effect, message } => {
        write!(
          formatter,
          "Particle effect '{effect}' has editor data that does not read: {message}"
        )
      }
      Self::UndefinedEffect { group, effect } => {
        write!(formatter, "Particle group '{group}' plays undefined effect '{effect}'")
      }
      Self::UnnamedChild { group, effect, child } => write!(
        formatter,
        "Particle group '{group}' effect '{effect}' spawns a {child:?} child but names none"
      ),
    }
  }
}
