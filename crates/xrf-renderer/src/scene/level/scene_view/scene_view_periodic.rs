use glam::Vec3;

use crate::feature::effects::rain::cover::rain_cover::RainCover;
use crate::feature::weather::wet::puddles::level_surface::LevelSurface;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::scene::level::feature::scene_features::SceneFeatures;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::scene_view::SceneView;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::static_scene::static_scene::StaticScene;

impl SceneView {
  /// Offers `scheduler` what the view draws once in so many frames and wants this frame: the sun's cascades, the
  /// surroundings' face, the rain cover, the level's surface and the sky's refill. `commit_periodic` then readies what
  /// was granted.
  pub fn offer_periodic(
    &mut self,
    scene: &LevelScene,
    device: &wgpu::Device,
    scheduler: &mut PeriodicScheduler,
    view_layout: &wgpu::BindGroupLayout,
  ) {
    let SceneView { shared, features, .. } = self;
    let SceneFeatures {
      shadows,
      rain_cover,
      level_surface,
      overhead_layout: overhead_view_layout,
      surroundings,
      reflections,
      rain,
      wet,
      sky,
      ..
    } = features;
    let statics: &StaticScene = &scene.statics;
    let frame: ShadowFrame<'_> = ShadowFrame {
      scene: statics,
      camera: &shared.camera,
      settings: &shared.shadow_settings,
      sun_direction: shared.sun_direction,
      is_sunlit: shared.lighting.sun.truncate() != Vec3::ZERO,
      switches: shared.switches,
      sway: Self::to_sway(statics, &features.wind),
      grass_reach: features.grass.get_cascade_reach(),
    };

    shadows.prepare_cascades(device, view_layout, &frame, scheduler);

    // The cube is for the traced reflections alone; the engine's reflect its own.
    if reflections.get_settings().is_drawn() {
      surroundings.offer(device, &frame, reflections.get_settings().probe, scheduler);
    } else {
      surroundings.rest();
    }

    // Out to the wet reach where the wetting reaches past the engine's.
    if rain.is_raining() || wet.is_wet() {
      rain_cover
        .get_or_insert_with(|| RainCover::new(device, overhead_view_layout))
        .offer(&frame, wet.get_reach(), scheduler);
    } else {
      *rain_cover = None;
    }

    // The puddles seat on it.
    if wet.is_enhanced() {
      level_surface
        .get_or_insert_with(|| LevelSurface::new(device, overhead_view_layout))
        .offer(&frame, scene.lights.get_fires(), scheduler);
    } else {
      *level_surface = None;
    }

    sky.offer_refill(scheduler);
  }

  /// Readies what `scheduler` granted of what `offer_periodic` offered, which the frame's passes then draw, and writes
  /// where the overhead maps now stand; `encoder` takes what readying them writes.
  pub fn commit_periodic(
    &mut self,
    scene: &LevelScene,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    scheduler: &PeriodicScheduler,
  ) {
    let SceneView { shared, features, .. } = self;
    let SceneFeatures {
      shadows,
      rain_cover,
      level_surface,
      surroundings,
      reflections,
      sky,
      ..
    } = features;
    let statics: &StaticScene = &scene.statics;
    let frame: ShadowFrame<'_> = ShadowFrame {
      scene: statics,
      camera: &shared.camera,
      settings: &shared.shadow_settings,
      sun_direction: shared.sun_direction,
      is_sunlit: shared.lighting.sun.truncate() != Vec3::ZERO,
      switches: shared.switches,
      sway: Self::to_sway(statics, &features.wind),
      grass_reach: features.grass.get_cascade_reach(),
    };

    shadows.commit_cascades(device, encoder, &frame, scheduler);

    if reflections.get_settings().is_drawn() {
      surroundings.commit(device, encoder, &frame, reflections.get_settings().probe, scheduler);
    }

    if let Some(rain_cover) = rain_cover {
      rain_cover.commit(device, encoder, &frame, scheduler);
    }

    if let Some(level_surface) = level_surface {
      level_surface.commit(device, encoder, &frame, scheduler);
    }

    sky.commit_refill(scheduler);
    self.follow_overhead_maps();
  }

  /// Readies the scene's light shadow faces `scheduler` granted of those every view of it asked for this frame, from
  /// this view's culling; `encoder` takes the copies a list growing makes.
  pub fn commit_scene_shadows(
    &self,
    scene: &mut LevelScene,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    scheduler: &PeriodicScheduler,
  ) {
    let LevelScene { statics, lights, .. } = scene;
    let frame: ShadowFrame<'_> = ShadowFrame {
      scene: statics,
      camera: &self.shared.camera,
      settings: &self.shared.shadow_settings,
      sun_direction: self.shared.sun_direction,
      is_sunlit: self.shared.lighting.sun.truncate() != Vec3::ZERO,
      switches: self.shared.switches,
      sway: Self::to_sway(statics, &self.features.wind),
      grass_reach: self.features.grass.get_cascade_reach(),
    };

    lights.commit_shadows(device, encoder, &frame, scheduler);
  }
}
