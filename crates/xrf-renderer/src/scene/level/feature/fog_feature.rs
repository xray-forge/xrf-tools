use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureDescriptor};

use crate::contract::render_applied_height_fog::RenderAppliedHeightFog;
use crate::contract::render_fog_settings::RenderFogSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::texture_copy::add_texture_copy;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::fog_scattering_parameters::FogScatteringParameters;
use crate::pass::fog_scattering_pass::FogScatteringPass;
use crate::pass::fog_scattering_uniform::FogScatteringUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;

/// The camera hemi under which the enhanced fog's height fog is gone and over which it is whole, about the engine's
/// indoors at `0.05` (`CGamePersistent::WeathersUpdate`).
const INDOOR_HEMI_FADE: (f32, f32) = (0.03, 0.1);

/// The enhanced fog: its height fog as applied this frame, and the scattering's three stages over the scene.
#[derive(Default)]
pub struct FogFeature {
  height: Option<RenderAppliedHeightFog>,
  stages: Option<[FogScatteringUniform; 3]>,
}

impl FogFeature {
  /// The share of the enhanced fog's height fog a camera standing at a hemi (`camera_hemi`, none where the level gives
  /// none) sees: none indoors, rising smoothly to whole outdoors, by its hemi as the engine tells indoors.
  pub fn get_indoor_share(camera_hemi: Option<f32>) -> f32 {
    camera_hemi.map_or(1.0, |hemi| {
      let share: f32 = ((hemi - INDOOR_HEMI_FADE.0) / (INDOOR_HEMI_FADE.1 - INDOOR_HEMI_FADE.0)).clamp(0.0, 1.0);

      share * share * (3.0 - 2.0 * share)
    })
  }

  /// Takes this frame's fog: the enhanced height fog where the view draws the weather's fog, and the scattering's
  /// stages for a frame `size` texels across.
  pub fn prepare(
    &mut self,
    options: &RenderViewOptions,
    lighting: &RenderLighting,
    size: Option<(u32, u32)>,
    time: f32,
  ) {
    let fog: &RenderFogSettings = &options.features.fog;
    let is_fogged: bool = options.mode.is_lit && options.show.is_fogged && lighting.fog.is_some();

    self.stages = size.and_then(|size| FogScatteringUniform::for_view(options, lighting.fog.is_some(), size, time));
    self.height = (is_fogged && fog.is_enhanced()).then(|| RenderAppliedHeightFog {
      height: fog.height,
      density: fog.density,
      sun_color: fog.sun_color,
      scattering: self.stages.map_or(0.0, |stages| stages[0].intensity),
    });
  }

  /// The height fog as applied this frame, where the enhanced fog draws.
  pub fn get_applied(&self) -> Option<RenderAppliedHeightFog> {
    self.height
  }

  /// Adds the scattering over the scene: the blurs to a quarter and to half its size, the scattering at its size, and
  /// the scattered frame copied back over the scene; nothing where it does not scatter.
  pub fn add_passes<'a>(
    &self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (targets, lit): (ViewTargetHandles, LightingHandles<'a>),
  ) {
    let Some(stages) = self.stages else {
      return;
    };
    let empty: GraphTexture =
      bindings.import_view(&mut *graph, "fog scattering empty", passes.fog_scattering.get_empty());
    let mut create = |label: &'static str, stage: &FogScatteringUniform, format: wgpu::TextureFormat| {
      graph.create_texture(GraphTextureDescriptor::new_2d(
        label,
        stage.size.x as u32,
        stage.size.y as u32,
        format,
      ))
    };
    let quarter: GraphTexture = create("fog scattered quarter", &stages[0], FogScatteringPass::BLURRED);
    let half: GraphTexture = create("fog scattered half", &stages[1], FogScatteringPass::BLURRED);
    let scattered: GraphTexture = create("fog scattered", &stages[2], ViewTargets::SCENE);

    for (stage, (source, blurred, written)) in [
      (targets.scene, empty, quarter),
      (quarter, empty, half),
      (targets.scene, half, scattered),
    ]
    .into_iter()
    .enumerate()
    {
      let parameters: FogScatteringParameters = FogScatteringParameters {
        source,
        blurred,
        source_sampler: passes.fog_scattering.get_sampler(),
        depth_target: targets.depth,
        lighting: lit.lighting,
        scattering: runtime.push_uniform(&stages[stage]),
      };

      graph
        .add_raster_pass(FogScatteringPass::STAGES[stage])
        .parameters(&parameters)
        .color(GraphColorAttachment::new(
          written,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .record(move |context| passes.fog_scattering.record(context, view, &parameters, stage == 2));
    }

    add_texture_copy(&mut *graph, FogScatteringPass::STAGES[3], scattered, targets.scene);
  }
}

impl ViewFeature for FogFeature {
  /// Its targets are the frame's own, made and let go within it.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {
    *self = Self::default();
  }
}
