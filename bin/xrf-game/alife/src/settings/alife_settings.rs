use std::time::Duration;

use xrf_ltx::{Ltx, Section};

/// What `[alife]` sets of A-Life (`gamedata/configs/alife.ltx`): how far from the actor objects go online and offline,
/// how many offline objects an update takes, and the scheduler bounds its update interval comes from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlifeSettings {
  /// Metres from the actor the switch is centred on (`switch_distance`).
  pub switch_distance: f32,
  /// How far either side of it objects go online and offline, as a share of it (`switch_factor`).
  pub switch_factor: f32,
  /// Offline objects an update takes (`objects_per_update`).
  pub objects_per_update: u32,
  /// The scheduler's bounds for the A-Life update, in milliseconds (`schedule_min`, `schedule_max`).
  pub schedule_min: u32,
  pub schedule_max: u32,
}

impl Default for AlifeSettings {
  /// Call of Pripyat's (`gamedata/configs/alife.ltx`).
  fn default() -> Self {
    Self {
      switch_distance: 150.0,
      switch_factor: 0.1,
      objects_per_update: 10,
      schedule_min: 1,
      schedule_max: 1,
    }
  }
}

impl AlifeSettings {
  /// The section of the configs it is read from.
  pub const SECTION: &'static str = "alife";

  /// How the A-Life update weighs its interval between the scheduler's bounds (`CALifeUpdateManager::shedule_Scale`).
  const SCHEDULE_SCALE: f32 = 0.5;

  /// The settings `ltx`'s `[alife]` holds, each key it lacks or cannot read taken from the defaults and logged.
  pub fn read_ltx(ltx: &Ltx) -> Self {
    let defaults: Self = Self::default();
    let Some(section) = ltx.section(Self::SECTION) else {
      log::warn!(
        "The configs hold no [{}]; A-Life runs on Call of Pripyat's",
        Self::SECTION
      );

      return defaults;
    };

    Self {
      switch_distance: Self::read(section, "switch_distance", defaults.switch_distance),
      switch_factor: Self::read(section, "switch_factor", defaults.switch_factor),
      objects_per_update: Self::read(section, "objects_per_update", defaults.objects_per_update),
      schedule_min: Self::read(section, "schedule_min", defaults.schedule_min),
      schedule_max: Self::read(section, "schedule_max", defaults.schedule_max),
    }
  }

  /// The distance within which an offline object goes online (`CALifeSwitchManager::online_distance`).
  pub fn get_online_distance(&self) -> f32 {
    self.switch_distance * (1.0 - self.switch_factor)
  }

  /// The distance beyond which an online object goes offline (`CALifeSwitchManager::offline_distance`).
  pub fn get_offline_distance(&self) -> f32 {
    self.switch_distance * (1.0 + self.switch_factor)
  }

  /// How often A-Life updates, as X-Ray's scheduler spaces it (`CSheduler::ProcessStep`, `xrEngine/xrSheduler.cpp`):
  /// between the bounds, at least 30 ms, weighed by the update's scale; 265 ms for Call of Pripyat's.
  pub fn get_update_interval(&self) -> Duration {
    let lowest: u32 = self.schedule_min.max(30);
    let highest: u32 = 1000_u32.midpoint(self.schedule_max);
    let weighed: u32 = lowest + ((highest.saturating_sub(lowest)) as f32 * Self::SCHEDULE_SCALE).floor() as u32;

    Duration::from_millis(u64::from(weighed.clamp(lowest.max(20), highest.max(lowest))))
  }

  fn read<T: std::str::FromStr + std::fmt::Display + Copy>(section: &Section, key: &str, default: T) -> T {
    match section.get(key).map(|value| value.trim().parse::<T>()) {
      Some(Ok(value)) => value,
      Some(Err(_)) => {
        log::warn!("[{}] {key} is no number; A-Life takes {default}", Self::SECTION);

        default
      }
      None => default,
    }
  }
}
