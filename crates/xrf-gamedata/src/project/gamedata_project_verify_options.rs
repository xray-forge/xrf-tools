use std::collections::HashSet;

use crate::project::events::{GamedataEvent, GamedataListener};
use crate::project::gamedata_verification_type::GamedataVerificationType;

#[derive(Clone, Default)]
pub struct GamedataProjectVerifyOptions {
  /// What hears the run's events: its start, and each check as it finishes. Nothing hears them by default.
  pub listener: Option<GamedataListener>,
  /// Whether somebody reads the run as it goes, so the checks run in waves and the listener hears each wave as it
  /// finishes. Unfollowed, the default, every check runs at once and the listener hears them all at the end: a silent or
  /// scripted run still wants the findings, only not sooner at the cost of a slower sweep.
  pub is_followed: bool,
  /// Where progress goes and where cancellation comes from.
  pub job: xrf_job::JobHandle,
  pub is_strict: bool,
  pub checks: Vec<GamedataVerificationType>,
}

impl GamedataProjectVerifyOptions {
  /// The same options, handing the run's events to `listener`, or to nothing.
  pub fn with_listener(self, listener: Option<GamedataListener>) -> Self {
    Self { listener, ..self }
  }

  pub fn selected_checks(&self) -> Vec<GamedataVerificationType> {
    let mut seen: HashSet<GamedataVerificationType> = HashSet::with_capacity(self.checks.len());

    self
      .checks
      .iter()
      .copied()
      // Coverage and collisions always run first, so a caller naming either programmatically must not run it again.
      .filter(|check| {
        !matches!(
          check,
          GamedataVerificationType::Coverage | GamedataVerificationType::Collisions
        )
      })
      .filter(|check| seen.insert(*check))
      .collect()
  }

  /// Hands `event` to the listener, if any.
  pub(crate) fn emit(&self, event: GamedataEvent<'_>) {
    if let Some(listener) = &self.listener {
      listener(&event);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::GamedataProjectVerifyOptions;
  use crate::project::gamedata_verification_type::GamedataVerificationType;

  #[test]
  fn selected_checks_preserves_first_requested_order_and_removes_duplicates() {
    let options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions {
      checks: vec![
        GamedataVerificationType::Textures,
        GamedataVerificationType::Scripts,
        GamedataVerificationType::Textures,
      ],
      ..Default::default()
    };

    assert_eq!(
      options.selected_checks(),
      vec![GamedataVerificationType::Textures, GamedataVerificationType::Scripts,]
    );
  }

  /// The two always-run checks already ran before the selection, so naming them must not run them a second time.
  #[test]
  fn selected_checks_leaves_out_the_always_run_checks() {
    let options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions {
      checks: vec![
        GamedataVerificationType::Coverage,
        GamedataVerificationType::Ltx,
        GamedataVerificationType::Collisions,
      ],
      ..Default::default()
    };

    assert_eq!(options.selected_checks(), vec![GamedataVerificationType::Ltx]);
  }
}
