use std::f32::consts::TAU;

use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

use crate::feature::scene::wind::foliage_wind_values::FoliageWindValues;
use crate::lighting::wind::render_tree_wind::RenderTreeWind;
use crate::scene::level::level_time::LevelTime;

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
  /// The enhanced foliage motion's wind, grass and trees strengths, its flow fields' drift this frame and the last, and
  /// the last frame's wind (`FoliageWindValues`).
  pub foliage_wind: Vec4,
  pub foliage_grass: Vec4,
  pub foliage_trees: Vec4,
  pub foliage_anim: Vec4,
  pub foliage_previous_anim: Vec4,
  pub foliage_flora: Vec4,
  pub foliage_previous_wind: Vec4,
}

impl WindUniform {
  /// The sway at a time, still for none or for no amplitude: the wind's turn taken within its rotation, and the wave's
  /// phase within the turn `shaders/static/sway.wgsl`'s `cyclic` repeats over.
  pub fn new(trees: Option<&RenderTreeWind>, time: LevelTime) -> Self {
    let Some(trees) = trees.filter(|trees| trees.amplitude > 0.0) else {
      return Self::default();
    };
    let rotation: f32 = if trees.rotation > 0.0 {
      TAU * time.get_phase(f64::from(trees.rotation)) / trees.rotation
    } else {
      0.0
    };

    let wind: Vec4 = Vec4::new(rotation.sin(), 0.0, -rotation.cos(), 0.0) * trees.amplitude;
    let wave: Vec4 = (Vec4::new(trees.wave.x, trees.wave.y, -trees.wave.z, 0.0) / TAU)
      .with_w(time.get_turns(f64::from(trees.speed) / std::f64::consts::TAU));

    Self {
      wind,
      wave,
      previous_wind: wind,
      previous_wave: wave,
      ..Self::default()
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

  /// This sway with the enhanced foliage motion's values.
  pub fn with_foliage(mut self, foliage: &FoliageWindValues) -> Self {
    self.foliage_wind = foliage.wind;
    self.foliage_grass = foliage.grass;
    self.foliage_trees = foliage.trees;
    self.foliage_anim = foliage.anim;
    self.foliage_previous_anim = foliage.previous_anim;
    self.foliage_flora = foliage.flora;
    self.foliage_previous_wind = foliage.previous_wind;
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
