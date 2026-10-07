//! The world a renderer draws: what moves and plays in a level whatever draws it — the cameras, the weather, the
//! level's streaming into its scene and its skinned objects' poses — run inline before each frame and handed to the
//! renderer as one frame's input.

pub(crate) mod camera;
pub(crate) mod contract;
pub(crate) mod level;
pub(crate) mod weather;
pub(crate) mod world;
pub(crate) mod world_event_sink;
pub(crate) mod world_scene;
pub(crate) mod world_viewport;

#[cfg(test)]
mod tests;

pub use crate::contract::world_ambient_effect_report::WorldAmbientEffectReport;
pub use crate::contract::world_ambient_report::WorldAmbientReport;
pub use crate::contract::world_camera::WorldCamera;
pub use crate::contract::world_camera_command::WorldCameraCommand;
pub use crate::contract::world_camera_pose::WorldCameraPose;
pub use crate::contract::world_input_event::WorldInputEvent;
pub use crate::contract::world_input_kind::WorldInputKind;
pub use crate::contract::world_level_problems::WorldLevelProblems;
pub use crate::contract::world_model_pose::WorldModelPose;
pub use crate::contract::world_sector_skip::WorldSectorSkip;
pub use crate::contract::world_surface_geometry::WorldSurfaceGeometry;
pub use crate::contract::world_surface_span::WorldSurfaceSpan;
pub use crate::contract::world_toggles::WorldToggles;
pub use crate::contract::world_viewport_event::WorldViewportEvent;
pub use crate::contract::world_weather_control::WorldWeatherControl;
pub use crate::contract::world_weather_effect_report::WorldWeatherEffectReport;
pub use crate::contract::world_weather_play::WorldWeatherPlay;
pub use crate::contract::world_weather_report::WorldWeatherReport;
pub use crate::contract::world_weather_transition::WorldWeatherTransition;
pub use crate::world::World;
pub use crate::world_event_sink::WorldEventSink;
