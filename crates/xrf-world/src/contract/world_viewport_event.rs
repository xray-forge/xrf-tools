use serde::{Deserialize, Serialize};

use crate::contract::camera::world_camera_pose::WorldCameraPose;
use crate::contract::weather::world_weather_report::WorldWeatherReport;

/// What a viewport's world tells its page.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WorldViewportEvent {
  /// Where the camera stands, sent while it moves and once more after it stops.
  Camera { pose: WorldCameraPose },
  /// The fly camera's wheel, turned while the secondary button flies it, asked for it faster or slower: whole steps,
  /// positive faster, which the page's speed setting takes, since the speed the camera flies at is the page's to keep.
  FlySpeed { steps: i32 },
  /// Where its weather stands, sent as it changes, a few times a second at most; none while nothing plays.
  Weather { report: Option<WorldWeatherReport> },
}
