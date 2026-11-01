use crate::contract::feature::image::bloom::render_bloom_mode::RenderBloomMode;
use crate::contract::feature::image::resolve::render_antialiasing::RenderAntialiasing;
use crate::contract::feature::tracing::render_trace_size::RenderTraceSize;
use crate::contract::report::applied::render_applied_reflections::RenderAppliedReflections;
use crate::contract::report::applied::render_applied_report::RenderAppliedReport;
use crate::contract::report::applied::render_applied_shadows::RenderAppliedShadows;
use crate::contract::report::frame::render_pool_use::RenderPoolUse;
use crate::contract::report::scene::render_static_report::RenderStaticReport;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::image::present::present_uniform::PresentUniform;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::view::scene_view::SceneView;
use crate::scene::level::view::view_reports::ViewReports;
use crate::scene::static_scene::draw::static_batch::StaticBatch;

impl SceneView {
  /// Asks for the counts recorded with the frame just submitted, which `take_reports` reads once they are back.
  pub fn request_stats(&self) {
    self.features.statics.request_stats();
    self.features.grass.request_stats();
    self.features.lights.request_report();
    self.features.reflections.request_demand();
    self.stages.effects.request_readbacks();
    self.features.exposure.request_readback();
    self.features.sky.request_readback();
  }

  /// Takes what it reports of its recent frames: the counts read back since the last report, its scene's particles'
  /// simulation, and what it draws with as resolved from `options`.
  pub fn take_reports(&mut self, scene: &mut LevelScene, options: &RenderViewOptions) -> ViewReports {
    let antialiasing: RenderAntialiasing = self
      .features
      .resolve
      .get_temporal()
      .or(self.stages.image.get_smoothing())
      .unwrap_or(RenderAntialiasing::None);

    ViewReports {
      static_draws: self.take_static_report(scene),
      lights: self.features.lights.take_report(),
      particles: scene.particles.take_report(self.features.particles.get_drawn()),
      grass: self.features.grass.take_report(),
      meter: self.stages.effects.take_meter_reading(),
      render_size: self.get_render_size(),
      output_size: self.targets.as_ref().map(|_| self.features.resolve.get_resolved()),
      antialiasing,
      is_ray_reconstructed: self.shared.resolve.is_ray_reconstructed,
      applied: self.describe_applied(options, antialiasing),
    }
  }

  /// The size its scene is rendered at, once its targets are made.
  pub fn get_render_size(&self) -> Option<(u32, u32)> {
    self.targets.as_ref().map(|it| (it.width, it.height))
  }

  /// What its window's present pass reads this frame.
  pub fn get_present_uniform(&self) -> &PresentUniform {
    self.features.present.get_uniform()
  }

  /// What its frames are drawn with, smoothed by `antialiasing`, as resolved from what `options` asked; each stage
  /// writes its own part.
  fn describe_applied(&self, options: &RenderViewOptions, antialiasing: RenderAntialiasing) -> RenderAppliedReport {
    let cascades: usize = self.shared.shadow_settings.get_cascade_count();
    let mut applied: RenderAppliedReport = RenderAppliedReport {
      antialiasing,
      is_ray_reconstructed: self.shared.resolve.is_ray_reconstructed,
      render_scale: self.features.resolve.get_upscaling().scale,
      shadows: (cascades > 0).then(|| RenderAppliedShadows {
        cascades: self.shared.shadow_settings.cascades[..cascades].to_vec(),
        resolution: self.features.shadows.get_maps().map_or(0, |maps| maps.resolution),
        filter: self.shared.shadow_settings.filter,
      }),
      reflections: self
        .features
        .reflections
        .get_trace()
        .map(|trace| RenderAppliedReflections {
          intensity: trace.intensity,
          trace_size: if trace.is_raw {
            RenderTraceSize::Full
          } else {
            trace.trace_size
          },
          steps: self.features.reflections.get_settings().get_steps(),
        }),
      debanding: self.features.present.get_debanding(),
      bloom: if self.features.bloom.is_enhanced_drawn() {
        Some(RenderBloomMode::Enhanced)
      } else {
        self.features.bloom.is_engine_drawn().then_some(RenderBloomMode::Engine)
      },
      lights: Some(self.features.lights.get_settings()).filter(|settings| settings.is_enabled),
      grass: self.features.grass.get_applied(&options.features.grass),
      is_water: options.features.water.is_enabled,
      environment: None,
      sun: self
        .features
        .sky
        .get_sun_sprite()
        .and_then(|_| self.stages.effects.get_shown_flare())
        .map(str::to_owned),
      ..RenderAppliedReport::default()
    };

    self.stages.lighting.describe(&mut applied);
    self.stages.effects.describe(&mut applied);

    applied
  }

  /// The static draws' pools and what the latest counted frame's cull kept and hid.
  fn take_static_report(&mut self, scene: &LevelScene) -> RenderStaticReport {
    let [kept_clusters, kept_triangles, occluded_clusters, occluded_triangles] = self.features.statics.take_stats();
    let pools: RenderStaticReport = scene.statics.get_pools();
    let phases: u32 = if self.features.statics.get_cull().is_occluding != 0 {
      2
    } else {
      1
    };

    RenderStaticReport {
      surface_list: RenderPoolUse {
        used: kept_clusters,
        ..pools.surface_list
      },
      commands: StaticBatch::list_deferred().count() as u32 * phases
        + u32::from(self.features.statics.get_cull().is_impostors != 0),
      kept_clusters,
      kept_triangles,
      occluded_clusters,
      occluded_triangles,
      ..pools
    }
  }
}
