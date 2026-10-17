use std::sync::Arc;

use xrf_dlss::Dlss;

use crate::camera::camera_view::CameraView;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_view_options::RenderViewOptions;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::level_passes::LevelPasses;
use crate::scene::texture::weather_textures::WeatherTextures;

/// What a view readies its frame from: the GPU and its passes, the camera, the sizes the frame is drawn, resolved and
/// shown at, the options, and the weather's light, textures and clock rate.
pub struct ViewPrepare<'a> {
  pub device: &'a wgpu::Device,
  pub queue: &'a wgpu::Queue,
  pub dlss: Option<&'a Dlss>,
  pub passes: LevelPasses<'a>,
  pub view: &'a CameraView,
  /// Texels the frame is drawn at.
  pub drawn: (u32, u32),
  /// Texels the frame is resolved at, past any upscaler.
  pub resolved: (u32, u32),
  /// Where the viewport shows it.
  pub output: RenderRect,
  pub field_of_view: f32,
  pub options: &'a RenderViewOptions,
  pub lighting: &'a RenderLighting,
  pub weather: Option<&'a Arc<RenderLevelWeather>>,
  pub weather_textures: WeatherTextures<'a>,
  /// How fast the weather's clock runs against real time.
  pub weather_rate: f32,
  /// Whether it is the first of its scene's views this frame, which draws the scene's own work: the lights' faces.
  pub is_scene_first: bool,
}
