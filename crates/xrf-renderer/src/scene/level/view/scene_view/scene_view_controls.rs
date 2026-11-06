use std::time::Duration;

use glam::{Mat4, Vec2};
use xrf_dlss::Dlss;

use crate::camera::camera_view::CameraView;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::pass::view::view_motion::ViewMotion;
use crate::scene::level::view::scene_view::SceneView;

impl SceneView {
  /// Holds its exposure where it stands, metered no more, or lets it adapt again.
  pub fn freeze_exposure(&mut self, is_frozen: bool) {
    self.features.exposure.set_frozen(is_frozen);
  }

  /// Draws its frames `step` apart, however long each takes, or, none, at the time each is drawn: what adapts or fades
  /// over its frames moves by it.
  pub fn step_clock(&mut self, step: Option<Duration>) {
    self.clock.set_step(step);
  }

  /// Starts what it carries from frame to frame and moves by the time again from nothing at its next frame: its last
  /// frame's time, which every unit steps on from, the wind's drift and sway, the water's scroll, the falling rain's
  /// clock and easing, the exposure's adaptation, and the temporal resolve's history and jitter. What it draws from then
  /// on follows from its frames since alone, as two bursts from one time must.
  pub fn restart_motion(&mut self) {
    self.last_time = None;
    self.features.wind.restart();
    self.features.rain.restart();
    self.features.grass.restart_wind();
    self.features.water.restart();
    self.features.exposure.restart();
    self.features.resolve.restart();
  }

  /// Draws its next frames as a reference's `sample` of `samples`: unresolved, at that place of the jitter's
  /// sequence; none resolves them again.
  pub fn sample_reference(&mut self, reference: Option<(u32, u32)>) {
    self.features.resolve.set_reference(reference);
  }

  /// What this frame, seen through `view` and drawn by the `jittered` view projection, measures its motion and
  /// reprojects by; none moved across a cut.
  pub fn next_motion(&mut self, view: &CameraView, jittered: Mat4) -> ViewMotion {
    self.features.resolve.next_motion(view, jittered)
  }

  /// Where the next frame's samples sit within their pixels, in drawn pixels, `y` down, on a GPU that has DLSS where
  /// `dlss` gives it: jittered while it resolves frames temporally and shows the finished frame, still otherwise.
  pub fn next_jitter(&mut self, options: &RenderViewOptions, ratio: f32, dlss: Option<&Dlss>) -> Vec2 {
    self.features.resolve.next_jitter(options, ratio, dlss)
  }

  /// The mip bias the surfaces' textures are read at this frame, as `next_jitter` decided it.
  pub fn get_texture_bias(&self) -> f32 {
    self.features.resolve.get_texture_bias()
  }

  /// Drops what a frame that was never submitted copied out for reading back, its counts and its pick, and forgets the
  /// periodic work it drew, which draws again.
  pub fn abandon_frame(&mut self) {
    self.features.statics.abandon_stats();
    self.features.statics.abandon_frame();
    self.features.resolve.abandon_frame();
    self.features.grass.abandon_stats();
    self.features.lights.abandon_report();
    self.features.reflections.abandon_demand();
    self.stages.effects.abandon_frame();
    self.features.exposure.abandon_readback();
    self.features.pick.abandon();
    // What the frame drew of the periodic work was never drawn.
    self.features.shadows.abandon_frame();
    self.features.surroundings.abandon_frame();
    self.features.sky.abandon_frame();

    if let Some(rain_cover) = &mut self.features.rain_cover {
      rain_cover.abandon_frame();
    }

    if let Some(level_surface) = &mut self.features.level_surface {
      level_surface.abandon_frame();
    }
  }
}
