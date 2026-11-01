use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_applied_reflections::RenderAppliedReflections;
use crate::contract::render_applied_report::RenderAppliedReport;
use crate::contract::render_applied_shadows::RenderAppliedShadows;
use crate::contract::render_bloom_mode::RenderBloomMode;
use crate::contract::render_grass_report::RenderGrassReport;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::contract::render_meter_reading::RenderMeterReading;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_static_report::RenderStaticReport;
use crate::contract::render_trace_size::RenderTraceSize;
use crate::contract::render_view_options::RenderViewOptions;
use crate::pass::present_uniform::PresentUniform;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::scene_view::SceneView;
use crate::scene::static_scene::static_batch::StaticBatch;

impl SceneView {
  /// What the grass came to on a recent frame.
  pub fn take_grass_report(&mut self) -> RenderGrassReport {
    self.features.grass.take_report()
  }

  /// What the spot meter last read; none while the view does not meter.
  pub fn take_meter_reading(&mut self) -> Option<RenderMeterReading> {
    self.stages.effects.take_meter_reading()
  }

  /// The particles' report for this view: the scene's simulation since the last report and the quads this view drew.
  pub fn take_particles_report(&self, scene: &mut LevelScene) -> RenderParticlesReport {
    scene.particles.take_report(self.features.particles.get_drawn())
  }

  /// The size its scene is rendered at, once its targets are made.
  pub fn get_render_size(&self) -> Option<(u32, u32)> {
    self.targets.as_ref().map(|it| (it.width, it.height))
  }

  /// What smooths its frame as drawn: the temporal resolve, else the smoothing, none where neither runs.
  pub fn get_antialiasing(&self) -> RenderAntialiasing {
    self
      .features
      .resolve
      .get_temporal()
      .or(self.stages.image.get_smoothing())
      .unwrap_or(RenderAntialiasing::None)
  }

  /// Whether DLSS reconstructs rays as it resolves its frame.
  pub fn is_ray_reconstructed(&self) -> bool {
    self.shared.resolve.is_ray_reconstructed
  }

  /// What its window's present pass reads this frame.
  pub fn get_present_uniform(&self) -> &PresentUniform {
    self.features.present.get_uniform()
  }

  /// The size its frame is resolved at, none before its targets are made.
  pub fn get_output_size(&self) -> Option<(u32, u32)> {
    self.targets.as_ref().map(|_| self.features.resolve.get_resolved())
  }

  /// Asks for the counts recorded with the frame just submitted.
  pub fn request_stats(&self) {
    self.features.statics.request_stats();
    self.features.grass.request_stats();
    self.features.lights.request_report();
    self.features.reflections.request_demand();
    self.stages.effects.request_readbacks();
    self.features.exposure.request_readback();
    self.features.sky.request_readback();
  }

  /// What its frames are drawn with, as resolved from what `options` asked; the weather's light is the viewport's to
  /// add.
  pub fn describe_applied(&self, options: &RenderViewOptions) -> RenderAppliedReport {
    let antialiasing: RenderAntialiasing = self.get_antialiasing();
    let cascades: usize = self.shared.shadow_settings.get_cascade_count();
    let mut applied: RenderAppliedReport = RenderAppliedReport {
      antialiasing,
      is_ray_reconstructed: self.is_ray_reconstructed(),
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

  /// The static draws' pools and what the latest counted frame's cull kept and hid, and its lights.
  pub fn take_stats(&mut self, scene: &LevelScene) -> (RenderStaticReport, RenderLightsReport) {
    let [kept_clusters, kept_triangles, occluded_clusters, occluded_triangles] = self.features.statics.take_stats();
    let pools: RenderStaticReport = scene.statics.get_pools();
    let phases: u32 = if self.features.statics.get_cull().is_occluding != 0 {
      2
    } else {
      1
    };

    (
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
      },
      self.features.lights.take_report(),
    )
  }
}
