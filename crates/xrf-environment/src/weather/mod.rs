//! Weather cycles and effects: their keyframes as authored, and each as the engine loads it.

pub(crate) mod weather_cycle;
pub(crate) mod weather_cycle_id;
pub(crate) mod weather_cycle_kind;
pub(crate) mod weather_descriptor;
pub(crate) mod weather_key;
pub(crate) mod weather_keyframe;
pub(crate) mod weather_time;

pub use crate::weather::weather_cycle::WeatherCycle;
pub use crate::weather::weather_cycle_id::WeatherCycleId;
pub use crate::weather::weather_cycle_kind::WeatherCycleKind;
pub use crate::weather::weather_descriptor::WeatherDescriptor;
pub use crate::weather::weather_key::WeatherKey;
pub use crate::weather::weather_keyframe::WeatherKeyframe;
pub use crate::weather::weather_time::WeatherTime;
