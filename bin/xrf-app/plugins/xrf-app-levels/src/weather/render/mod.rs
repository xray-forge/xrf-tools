//! What a level's weather plays with that only the renderer reads: rain, wet surfaces and thunderbolts as the engine
//! loads them, the models they draw, and the level's weather converted to what the renderer takes.

pub(crate) mod level_bolt_reader;
mod level_rain;
mod level_render_weather;
mod level_thunderbolt;
mod level_thunderbolt_gradient;
mod level_thunderbolt_model;
mod level_thunderbolt_settings;
mod level_thunderbolts;
mod level_weather_model;
mod level_weather_source;
mod level_wet_surfaces;

pub use level_rain::LevelRain;
pub use level_render_weather::LevelRenderWeather;
pub use level_thunderbolt::LevelThunderbolt;
pub use level_thunderbolt_gradient::LevelThunderboltGradient;
pub use level_thunderbolt_model::LevelThunderboltModel;
pub use level_thunderbolt_settings::LevelThunderboltSettings;
pub use level_thunderbolts::LevelThunderbolts;
pub use level_weather_model::LevelWeatherModel;
pub use level_weather_source::LevelWeatherSource;
pub use level_wet_surfaces::LevelWetSurfaces;
