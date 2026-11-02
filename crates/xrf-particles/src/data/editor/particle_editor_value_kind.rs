use serde::{Deserialize, Serialize};

/// The kinds of value an editor action holds, in the order `EParticleAction::Save` writes them: each kind's values
/// together, before the next kind's.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ParticleEditorValueKind {
  Float,
  Vector,
  Domain,
  Bool,
  Int,
}

impl ParticleEditorValueKind {
  /// Every kind, in the order a stored action lists them.
  pub const STORED_ORDER: [Self; 5] = [Self::Float, Self::Vector, Self::Domain, Self::Bool, Self::Int];
}
