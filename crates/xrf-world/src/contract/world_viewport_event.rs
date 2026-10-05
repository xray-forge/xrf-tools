use serde::{Deserialize, Serialize};

use crate::contract::world_camera_pose::WorldCameraPose;
use crate::contract::world_weather_report::WorldWeatherReport;

/// What a viewport's world tells its page.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WorldViewportEvent {
  /// Where the camera stands, sent while it moves and once more after it stops.
  Camera { pose: WorldCameraPose },
  /// Where its weather stands, sent as it changes, a few times a second at most; none while nothing plays.
  Weather { report: Option<WorldWeatherReport> },
}
