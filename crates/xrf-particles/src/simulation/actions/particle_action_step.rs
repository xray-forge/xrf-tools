use crate::simulation::particle_engine_rules::ParticleEngineRules;

/// What `ParticleManager::Update` hands each action in turn: the step and the last `KillOld` age.
pub(crate) struct ParticleActionStep<'a> {
  pub dt: f32,
  pub kill_old_time: f32,
  pub rules: &'a ParticleEngineRules,
}

impl<'a> ParticleActionStep<'a> {
  /// `P_MAXFLOAT`: a radius at or past this is unlimited, which actions test to skip their distance check.
  pub const MAX_FLOAT: f32 = 1.0e16;

  /// `kill_old_time`'s value before any `KillOld` runs.
  pub const DEFAULT_KILL_OLD_TIME: f32 = 1.0;

  pub fn new(dt: f32, rules: &'a ParticleEngineRules) -> Self {
    Self {
      dt,
      kill_old_time: Self::DEFAULT_KILL_OLD_TIME,
      rules,
    }
  }
}
