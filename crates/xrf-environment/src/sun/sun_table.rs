use serde::Serialize;
use xrf_engine_target::XrayEngine;
use xrf_ltx::Ltx;

use crate::finding::EnvironmentRule;
use crate::section::{EnvironmentSection, EnvironmentSectionReader};
use crate::sun::sun_position::SunPosition;
use crate::sun::sun_position_key::SunPositionKey;

/// Monolith's `environment\sun_positions.ltx`: where the sun stands at each whole hour, lerped by the minute
/// (`calculate_config_sun_dir`) in place of any keyframe's.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SunTable {
  /// The config, as a logical path.
  pub file: String,
  /// Twenty-four hours, midnight first; an hour the config does not write is left empty, at the engine's zero.
  pub hours: Vec<EnvironmentSection<SunPositionKey>>,
}

impl SunTable {
  /// Hours in the table, each of which the engine reads.
  pub const HOURS: usize = 24;

  /// Reads the twenty-four `[HH:00:00]` sections.
  pub(crate) fn read(reader: &mut EnvironmentSectionReader, ltx: &Ltx) -> Self {
    let hours: Vec<EnvironmentSection<SunPositionKey>> = (0..Self::HOURS)
      .map(|hour| {
        let name: String = format!("{hour:02}:00:00");

        match ltx.section(&name) {
          Some(section) => reader.read::<SunPositionKey>(&name, section),
          None => {
            let message: String = format!("The sun table has no [{name}], which the engine reads");

            reader.report_file(EnvironmentRule::Engine, message);

            EnvironmentSection::new(name, reader.get_file())
          }
        }
      })
      .collect();

    Self {
      file: reader.get_file().to_owned(),
      hours,
    }
  }

  /// Where the sun stands at a whole hour.
  pub fn get_hour(&self, hour: usize) -> SunPosition {
    self
      .hours
      .get(hour % Self::HOURS)
      .map_or_else(SunPosition::default, |position| SunPosition {
        altitude: position.get_number(SunPositionKey::SunAltitude, XrayEngine::Extended),
        longitude: position.get_number(SunPositionKey::SunLongitude, XrayEngine::Extended),
      })
  }

  /// Every hour's position, midnight first.
  pub fn list_positions(&self) -> Vec<SunPosition> {
    (0..Self::HOURS).map(|hour| self.get_hour(hour)).collect()
  }
}
