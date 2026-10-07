use std::f32::consts::TAU;

use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

use crate::lighting::render_tree_wind::RenderTreeWind;

/// The sway of the trees as `shaders/static/pulling.wgsl` reads it, built each frame as `FTreeVisual_setup::calculate`
/// builds it: a wind turning once every `rotation` seconds at the amplitude's length, and a wave travelling through the
/// level at the speed. In renderer space, where the engine's `z` is negated.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Wind")]
pub struct WindUniform {
  /// The engine's `wind`: which way the trees lean, and how far, across the ground.
  pub wind: Vec4,
  /// The engine's `wave`: its direction through the level, and its phase in `w`, both over a turn.
  pub wave: Vec4,
  /// The same two the frame before, which a swaying vertex's motion is measured from.
  pub previous_wind: Vec4,
  pub previous_wave: Vec4,
}

impl WindUniform {
  /// The sway at a time, still for none or for no amplitude.
  pub fn new(trees: Option<&RenderTreeWind>, time: f32) -> Self {
    let Some(trees) = trees.filter(|trees| trees.amplitude > 0.0) else {
      return Self::default();
    };
    let rotation: f32 = if trees.rotation > 0.0 {
      TAU * time / trees.rotation
    } else {
      0.0
    };

    let wind: Vec4 = Vec4::new(rotation.sin(), 0.0, -rotation.cos(), 0.0) * trees.amplitude;
    let wave: Vec4 = Vec4::new(trees.wave.x, trees.wave.y, -trees.wave.z, time * trees.speed) / TAU;

    Self {
      wind,
      wave,
      previous_wind: wind,
      previous_wave: wave,
    }
  }

  /// This sway after the one before it, or after itself for a first frame.
  pub fn following(mut self, before: Option<&Self>) -> Self {
    if let Some(before) = before {
      self.previous_wind = before.wind;
      self.previous_wave = before.wave;
    }

    self
  }

  /// Whether the trees move at all.
  pub fn is_swaying(&self) -> bool {
    self.wind != Vec4::ZERO
  }

  /// How far the trees lean for each metre of a vertex's rigid reach from its foot.
  pub fn get_amplitude(&self) -> f32 {
    self.wind.length()
  }
}
