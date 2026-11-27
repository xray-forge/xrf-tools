use std::convert::Infallible;
use std::time::{Duration, Instant};

use xrf_game_loop::{LoopSystem, LoopTime};

use crate::alife_context::AlifeContext;
use crate::settings::alife_settings::AlifeSettings;
use crate::switch::alife_switch::AlifeSwitch;
use crate::switch::alife_switch_report::AlifeSwitchReport;

/// A-Life, a system of the game loop: every update interval (265 ms for Call of Pripyat, as X-Ray's scheduler spaces
/// it), the switch sweeps the current level. Offline objects' own updates come as modules of their own.
#[derive(Clone, Debug, PartialEq)]
pub struct Alife {
  settings: AlifeSettings,
  /// The graph level the actor is on, none while it is on none the graph holds.
  level: Option<u8>,
  /// Simulated time left before the next update; the first comes at the first step.
  until_update: Duration,
  /// What the last sweep did.
  last_switch: AlifeSwitchReport,
}

impl Alife {
  /// A-Life by `settings`, the actor on graph level `level`.
  pub fn new(settings: AlifeSettings, level: Option<u8>) -> Self {
    Self {
      settings,
      level,
      until_update: Duration::ZERO,
      last_switch: AlifeSwitchReport::default(),
    }
  }

  pub fn get_settings(&self) -> &AlifeSettings {
    &self.settings
  }

  pub fn get_last_switch(&self) -> &AlifeSwitchReport {
    &self.last_switch
  }
}

impl LoopSystem for Alife {
  /// Nothing from outside steers it yet.
  type Command = Infallible;
  type Context<'a> = AlifeContext<'a>;

  fn apply(&mut self, command: Infallible) {
    match command {}
  }

  fn step(&mut self, time: &LoopTime, context: AlifeContext<'_>) {
    let Some(level) = self.level else {
      return;
    };

    self.until_update = self.until_update.saturating_sub(time.step);

    if !self.until_update.is_zero() {
      return;
    }

    self.until_update = self.settings.get_update_interval();

    // The wall clock is read for the log alone: nothing of the state depends on it.
    let started: Instant = Instant::now();
    let is_first: bool = time.tick == 0;

    self.last_switch = AlifeSwitch::sweep(context.objects, level, context.actor, &self.settings, time.tick);

    let report: AlifeSwitchReport = self.last_switch;

    if is_first {
      log::info!(
        "A-Life's first switch weighed {} objects, {} online, in {:.2?}",
        report.swept,
        context.objects.get_online().len(),
        started.elapsed()
      );
    } else if report.went_online + report.went_offline > 0 {
      log::debug!(
        "A-Life switched {} online and {} offline of {} at tick {} in {:.2?}",
        report.went_online,
        report.went_offline,
        report.swept,
        time.tick,
        started.elapsed()
      );
    }
  }
}
