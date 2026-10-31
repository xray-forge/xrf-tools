use xrf_renderer_core::ShaderStruct;

use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::pass::noise_turn::NoiseTurn;
use crate::pass::vbao_frame::VbaoFrame;
use crate::pass::vbao_light::VbaoLight;

/// The exponent a strength of one raises the filtered visibility to, chosen by looking: creases read without
/// blackening.
const BASE_POWER: f32 = 1.5;

/// The share of the search target's height a slice is searched across at most, however near the point.
const MAX_REACH: f32 = 0.2;

/// Frames the noise cycles through before it repeats.
const NOISE_FRAMES: u32 = 256;

/// What VBAO's search, accumulation and filter read, as `shaders/frame/vbao.wgsl` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Vbao")]
pub struct VbaoUniform {
  pub radius: f32,
  pub thickness: f32,
  /// The exponent the filtered visibility is raised to.
  pub power: f32,
  /// Metres one pixel of the search target spans at a metre from the camera.
  pub spread: f32,
  /// Pixels of the search target a slice is searched across at most.
  pub reach: f32,
  /// Frames each pixel is averaged over at most; one for none.
  pub frames: f32,
  /// One where the last frame's accumulation is there to carry.
  pub has_history: f32,
  /// What this frame turns the slices' noise by, from zero to one.
  pub slice_noise: f32,
  /// What this frame moves the steps' noise by, from zero to one.
  pub step_noise: f32,
  /// How much of the gathered indirect light the filter passes on; zero where none is gathered.
  pub intensity: f32,
  /// One where the last frame's indirect light is there to carry.
  pub has_light_history: f32,
  /// Metres around a point the indirect light is gathered from, never under the occlusion's radius.
  pub light_radius: f32,
  /// The slices turned about a pixel and the steps along each; the steps a lit search takes past the occlusion's
  /// radius; and the rays RTAO casts a point in its place.
  pub slices: u32,
  pub steps: u32,
  pub light_steps: u32,
  pub rays: u32,
}

impl VbaoUniform {
  /// The settings over one `frame` of the search, gathering the indirect `light` beside the occlusion.
  pub fn new(settings: &RenderAmbientOcclusionSettings, light: VbaoLight, frame: VbaoFrame) -> Self {
    let frames: u32 = settings.vbao.get_accumulation();
    let (width, height): (u32, u32) = frame.size;
    let has_history: bool = frame.has_history && frames > 1;
    // A noise held still where nothing gathers it, so it never crawls.
    let [slice_noise, step_noise] = NoiseTurn::get_r2(if frames > 1 { frame.frame } else { 0 }, NOISE_FRAMES);

    Self {
      radius: settings.radius,
      thickness: settings.vbao.thickness.max(0.0),
      power: BASE_POWER * settings.strength.max(0.0),
      // Clip x over view x at a metre is the projection's first element.
      spread: 2.0 / (frame.projection.x_axis.x * width.max(1) as f32),
      reach: (height as f32 * MAX_REACH).max(1.0),
      frames: frames as f32,
      has_history: f32::from(u8::from(has_history)),
      slice_noise,
      step_noise,
      intensity: light.intensity.max(0.0),
      has_light_history: f32::from(u8::from(frame.has_light_history && has_history)),
      light_radius: light.radius.max(settings.radius),
      slices: settings.vbao.get_slices(),
      steps: settings.vbao.get_steps(),
      light_steps: light.steps,
      rays: settings.rtao.get_rays(),
    }
  }
}
