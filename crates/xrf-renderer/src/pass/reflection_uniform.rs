use xrf_renderer_core::ShaderStruct;

use crate::contract::render_reflection_settings::RenderReflectionSettings;
use crate::pass::reflection_trace::ReflectionTrace;

/// What the reflections' trace and filters read, as `shaders/frame/reflections.wgsl` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Reflections")]
pub struct ReflectionUniform {
  /// Cells of the depth pyramid a ray crosses at most.
  pub crossings: u32,
  /// The frame's count, which turns each pixel's noise from one frame to the next.
  pub frame: u32,
  /// What scales how much of a surface's reflection a traced one replaces, and how much a puddle's coat reflects.
  pub intensity: f32,
  /// Metres a ray is traced at most.
  pub distance: f32,
  /// One where the last frame's reflections are there to carry.
  pub has_history: f32,
  /// The frame's pixels a traced pixel stands for each way.
  pub ratio: f32,
  /// One where the last frame's scene and depth are kept for a hit to read its colour from.
  pub has_source: f32,
  /// One where a ray the screen does not answer is traced against the level's geometry: asked for, on a device that
  /// can.
  pub is_traced: f32,
  /// Frames such a pixel waits between its rays, the last answer reprojected in between.
  pub trace_turns: u32,
  /// The share of what it reflects a pixel's specular shows at most, below which it is not traced.
  pub trace_floor: f32,
}

impl ReflectionUniform {
  /// The share under which a pixel is not traced: a dielectric's 4% traced, or, on a dry level not asked to trace
  /// dry surfaces, only what reflects more, metal and glass-like surfaces, the rest left to the prefiltered sky.
  pub const TRACE_FLOOR: f32 = 0.02;
  pub const DRY_TRACE_FLOOR: f32 = 0.08;

  /// The settings and how they are traced, at a frame's count, with or without the last frame's reflections to carry
  /// and its scene to read, on a device that traces rays or not, over a level wet or dry. Raw, each pixel traces each
  /// frame.
  pub fn new(
    settings: &RenderReflectionSettings,
    trace: ReflectionTrace,
    frame: u32,
    (has_history, has_source): (bool, bool),
    (can_trace, is_wet): (bool, bool),
  ) -> Self {
    Self {
      crossings: settings.get_steps(),
      frame,
      intensity: settings.intensity.max(0.0),
      distance: settings.distance.max(0.0),
      has_history: f32::from(u8::from(has_history)),
      ratio: trace.get_ratio() as f32,
      has_source: f32::from(u8::from(has_source)),
      is_traced: f32::from(u8::from(settings.is_traced && can_trace)),
      trace_turns: if trace.is_raw { 1 } else { settings.get_trace_interval() },
      trace_floor: if settings.is_dry_traced || is_wet {
        Self::TRACE_FLOOR
      } else {
        Self::DRY_TRACE_FLOOR
      },
    }
  }
}
