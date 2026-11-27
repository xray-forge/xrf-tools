use std::convert::Infallible;
use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_game_loop::{LoopSystem, LoopTime};

use crate::alife_context::AlifeContext;
use crate::alife_data::AlifeData;
use crate::schedule::alife_schedule::AlifeSchedule;
use crate::schedule::alife_schedule_report::AlifeScheduleReport;
use crate::settings::alife_settings::AlifeSettings;
use crate::switch::alife_switch::AlifeSwitch;
use crate::switch::alife_switch_report::AlifeSwitchReport;

/// A-Life, a system of the game loop: every update interval (265 ms for Call of Pripyat, as X-Ray's scheduler spaces
/// it), the switch sweeps the current level, then the schedule updates the next offline objects of every level, in
/// that order as `CALifeUpdateManager::update` runs them. It lives as long as the game: the level the actor is on comes
/// with each step, so a level played after another sweeps there.
#[derive(Clone, Debug, PartialEq)]
pub struct Alife {
  settings: AlifeSettings,
  data: AlifeData,
  /// Simulated time left before the next update; the first comes at the first step.
  until_update: Duration,
  schedule: AlifeSchedule,
  /// What the last update's sweep and schedule did.
  last_switch: AlifeSwitchReport,
  last_schedule: AlifeScheduleReport,
}

impl Alife {
  /// A-Life by `settings` over `data`.
  pub fn new(settings: AlifeSettings, data: AlifeData) -> Self {
    Self {
      settings,
      data,
      until_update: Duration::ZERO,
      schedule: AlifeSchedule::default(),
      last_switch: AlifeSwitchReport::default(),
      last_schedule: AlifeScheduleReport::default(),
    }
  }

  pub fn get_settings(&self) -> &AlifeSettings {
    &self.settings
  }

  pub fn get_data(&self) -> &AlifeData {
    &self.data
  }

  pub fn get_last_switch(&self) -> &AlifeSwitchReport {
    &self.last_switch
  }

  pub fn get_last_schedule(&self) -> &AlifeScheduleReport {
    &self.last_schedule
  }
}

impl LoopSystem for Alife {
  /// Nothing from outside steers it yet.
  type Command = Infallible;
  type Context<'a> = AlifeContext<'a>;

  fn apply(&mut self, command: Infallible) {
    match command {}
  }

  fn step(&mut self, time: &LoopTime, context: AlifeContext<'_>) -> XrfResult<()> {
    self.until_update = self.until_update.saturating_sub(time.step);

    if !self.until_update.is_zero() {
      return Ok(());
    }

    self.until_update = self.settings.get_update_interval();

    // The wall clock is read for the log alone, and only where a line is written: nothing of the state depends on it.
    let started: Option<Instant> = (time.tick == 0 || log::log_enabled!(log::Level::Debug)).then(Instant::now);

    // While the actor is on no level the graph holds, the switch rests and offline life goes on.
    self.last_switch = match context.level {
      Some(level) => AlifeSwitch::sweep(context.objects, level, context.actor, &self.settings, time.tick),
      None => AlifeSwitchReport::default(),
    };
    self.last_schedule = self.schedule.update(
      context.objects,
      usize::try_from(self.settings.objects_per_update).unwrap_or(usize::MAX),
      time.tick,
    );

    let switch: AlifeSwitchReport = self.last_switch;

    let elapsed: Duration = started.map(|started| started.elapsed()).unwrap_or_default();

    if time.tick == 0 {
      // Counted for the log once: how many share the schedule sets how long each waits for its turn.
      let waiting: usize = context
        .objects
        .get_schedules()
        .iter()
        .filter(|(id, _)| context.objects.needs_update(*id))
        .count();

      log::info!(
        "A-Life's first update weighed {} objects, {} online, and updated {} of {waiting} offline objects needing \
         updates, in {elapsed:.2?}",
        switch.swept,
        context.objects.get_online().len(),
        self.last_schedule.updated
      );
    } else if switch.went_online + switch.went_offline > 0 {
      log::debug!(
        "A-Life switched {} online and {} offline of {} at tick {} in {elapsed:.2?}",
        switch.went_online,
        switch.went_offline,
        switch.swept,
        time.tick
      );
    }

    Ok(())
  }
}
