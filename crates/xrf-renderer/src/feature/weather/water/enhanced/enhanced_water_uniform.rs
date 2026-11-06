use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

use crate::contract::feature::weather::water::render_enhanced_water_settings::RenderEnhancedWaterSettings;
use crate::feature::weather::water::enhanced::enhanced_water_frame::EnhancedWaterFrame;

/// What the enhanced water's programs read as their `EnhancedWater`: its strengths, the weather's wind and rain, how
/// far its maps have scrolled, whether its reflection drew, and how far its reflection's misses are traced.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "EnhancedWater")]
pub struct EnhancedWaterUniform {
  pub refraction: f32,
  pub turbidity: f32,
  pub soft_border: f32,
  pub reflectivity: f32,
  /// The reflection's blur and the noise mixing the clear reflection in.
  pub reflection_blur: f32,
  pub blur_noise: f32,
  /// One where the reflection drew this frame, and one where the history it accumulates over holds a frame.
  pub reflected: f32,
  pub history: f32,
  /// The weather's wind velocity, which strengthens the waves.
  pub wind_velocity: f32,
  /// The sun's highlight and the caustics.
  pub specular: f32,
  pub caustics: f32,
  /// The waves' height and the rain's ripples, and how hard it rains.
  pub parallax_height: f32,
  pub ripples: f32,
  pub rain: f32,
  /// How far the maps' repeat is broken.
  pub variation: f32,
  /// Metres a reflected ray the screen does not answer is traced against the level, the surfaces' reflections' distance
  /// where they trace what the screen misses; none where they do not.
  pub trace_distance: f32,
  /// Frames a pixel the screen does not answer waits between its rays, the history held between, and the frame's count.
  pub trace_turns: u32,
  pub trace_frame: u32,
  /// How many times its turns a still, dim reflection waits, one for as often as any.
  pub dim_turns: u32,
  /// Steps the reflection marches over the screen.
  pub march_steps: u32,
  /// How much of its colour the reflection keeps.
  pub vibrance: f32,
  /// Metres a side of the tile the enhanced drops' rings wrap over (`RainUniform::get_rings`), which it ripples by in
  /// place of its map's rings, none without them; the metres from the eye they reach, and their strength, of their
  /// designed one.
  pub rings_tile: f32,
  pub rings_reach: f32,
  pub rings_strength: f32,
  /// One where every pixel traces every frame and the screen answers only plants, and metres from the eye within which
  /// it leaves those to the trace's grass cards too (`WaterFeature::follow_grass_cards`).
  pub fully_traced: u32,
  pub plant_reach: f32,
  pub _pad: [u32; 2],
  /// Where each of its maps' reads has scrolled to (`EnhancedWaterScroll`): the heights', the wave layers', the rain
  /// ripples' and the wind layer's, the caustics' and the calm patches', and the ripples' clocks.
  pub heights: Vec4,
  pub wave_layers: [Vec4; 2],
  pub map_offsets: [Vec4; 2],
  pub flow_scrolls: Vec4,
  pub ripple_clocks: Vec4,
}

impl EnhancedWaterUniform {
  /// The settings over one `frame` of the weather, the water's flow and its reflection.
  pub fn new(settings: &RenderEnhancedWaterSettings, frame: EnhancedWaterFrame) -> Self {
    let EnhancedWaterFrame {
      wind,
      rain,
      scroll,
      is_reflected,
      has_history,
      trace,
    } = frame;

    Self {
      refraction: settings.refraction,
      turbidity: settings.turbidity,
      soft_border: settings.soft_border,
      reflectivity: settings.reflectivity,
      reflection_blur: settings.reflection_blur,
      blur_noise: settings.blur_noise,
      reflected: is_reflected as u32 as f32,
      history: has_history as u32 as f32,
      wind_velocity: wind.velocity,
      specular: settings.specular,
      caustics: settings.caustics,
      parallax_height: settings.parallax_height,
      ripples: settings.ripples,
      rain,
      variation: settings.variation,
      trace_distance: trace.distance,
      trace_turns: trace.turns,
      trace_frame: trace.frame,
      dim_turns: trace.dim_turns.max(1),
      march_steps: trace.march_steps.max(1),
      fully_traced: trace.is_full as u32,
      vibrance: settings.vibrance,
      heights: scroll.heights,
      wave_layers: scroll.wave_layers,
      map_offsets: scroll.map_offsets,
      flow_scrolls: scroll.flow_scrolls,
      ripple_clocks: scroll.ripple_clocks,
      ..Default::default()
    }
  }
}
