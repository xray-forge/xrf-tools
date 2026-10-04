use xrf_engine_target::XrayEngine;

/// How each engine steps particles: OpenXRay by 33 ms, Monolith by 33 ms times `particle_update_mod`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleEngineRules {
  engine: XrayEngine,
  update_coefficient: f32,
}

impl ParticleEngineRules {
  /// `uDT_STEP` and `CPEDef::m_uStep`: the step both engines start from, in milliseconds.
  pub const BASE_STEP_MILLISECONDS: u32 = 33;

  /// `ps_particle_update_coeff`'s default.
  pub const DEFAULT_UPDATE_COEFFICIENT: f32 = 1.0;

  /// `particle_update_mod`'s console bounds.
  pub const UPDATE_COEFFICIENT_RANGE: (f32, f32) = (0.04, 10.0);

  /// The engine's rules; the coefficient is clamped to the console's bounds and read by Monolith alone.
  pub fn new(engine: XrayEngine, update_coefficient: f32) -> Self {
    Self {
      engine,
      update_coefficient: update_coefficient.clamp(Self::UPDATE_COEFFICIENT_RANGE.0, Self::UPDATE_COEFFICIENT_RANGE.1),
    }
  }

  pub fn get_engine(&self) -> XrayEngine {
    self.engine
  }

  /// `ps_particle_update_coeff` on Monolith; one on OpenXRay, which has no such value.
  pub fn get_update_coefficient(&self) -> f32 {
    match self.engine {
      XrayEngine::Vanilla => Self::DEFAULT_UPDATE_COEFFICIENT,
      XrayEngine::Extended => self.update_coefficient,
    }
  }

  /// `uDT_STEP`, `GetUStep`: milliseconds an effect accumulates before it takes a step, truncated as the engine casts.
  pub fn get_step_milliseconds(&self) -> u32 {
    match self.engine {
      XrayEngine::Vanilla => Self::BASE_STEP_MILLISECONDS,
      XrayEngine::Extended => ((Self::BASE_STEP_MILLISECONDS as f32 * self.update_coefficient) as u32).max(1),
    }
  }

  /// `fDT_STEP`, `GetFStep`: the seconds each step simulates.
  pub fn get_step_seconds(&self) -> f32 {
    let base: f32 = Self::BASE_STEP_MILLISECONDS as f32 / 1000.0;

    match self.engine {
      XrayEngine::Vanilla => base,
      XrayEngine::Extended => base * self.update_coefficient,
    }
  }
}

impl Default for ParticleEngineRules {
  fn default() -> Self {
    Self::new(XrayEngine::default(), Self::DEFAULT_UPDATE_COEFFICIENT)
  }
}

#[cfg(test)]
mod tests {
  use xrf_engine_target::XrayEngine;

  use super::ParticleEngineRules;

  #[test]
  fn steps_openxray_by_33_milliseconds_whatever_the_coefficient() {
    let rules: ParticleEngineRules = ParticleEngineRules::new(XrayEngine::Vanilla, 2.0);

    assert_eq!(rules.get_step_milliseconds(), 33);
    assert_eq!(rules.get_step_seconds(), 0.033);
    assert_eq!(rules.get_update_coefficient(), 1.0);
  }

  #[test]
  fn scales_monolith_steps_by_the_coefficient_truncating_milliseconds() {
    let rules: ParticleEngineRules = ParticleEngineRules::new(XrayEngine::Extended, 0.5);

    assert_eq!(rules.get_step_milliseconds(), 16);
    assert_eq!(rules.get_step_seconds(), 0.033 * 0.5);
  }

  #[test]
  fn clamps_the_coefficient_to_the_console_bounds() {
    assert_eq!(
      ParticleEngineRules::new(XrayEngine::Extended, 0.0).get_update_coefficient(),
      0.04
    );
  }
}
