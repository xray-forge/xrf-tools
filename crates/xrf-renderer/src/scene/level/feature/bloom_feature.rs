use xrf_engine_target::XrayEngine;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureDescriptor, UniformBinding,
};

use crate::contract::render_bloom_settings::RenderBloomSettings;
use crate::contract::render_enhanced_bloom_settings::RenderEnhancedBloomSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::bloom_parameters::BloomParameters;
use crate::pass::bloom_stage::BloomStage;
use crate::pass::bloom_uniform::BloomUniform;
use crate::pass::enhanced_bloom_parameters::EnhancedBloomParameters;
use crate::pass::enhanced_bloom_pass::EnhancedBloomPass;
use crate::pass::enhanced_bloom_stage::EnhancedBloomStage;
use crate::pass::enhanced_bloom_uniform::EnhancedBloomUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;

/// The bloom, the engine's or the enhanced one in its place: the engine's build and blurs at its fixed size, read by
/// the present; or the enhanced chain from the scene as it stands, its finish handed to the present.
#[derive(Default)]
pub struct BloomFeature {
  /// What the engine's build and its two blurs read this frame, where the engine's bloom draws.
  engine: Option<[BloomUniform; 3]>,
  /// What the enhanced bloom draws with, where it draws in the engine's place.
  enhanced: Option<RenderEnhancedBloomSettings>,
  /// The enhanced bloom's pipelines, one a stage, with the cache's generation they were asked for in; the last made
  /// are kept while a reloaded shader is refused.
  pipelines: Option<(u64, [wgpu::RenderPipeline; 5])>,
}

impl BloomFeature {
  /// Takes this frame's bloom for a frame `size` texels across, drawn where the view is lit and not wireframe; the
  /// enhanced bloom's pipelines asked of the cache when it draws and the cache has changed since they were.
  pub fn prepare(
    &mut self,
    passes: LevelPasses<'_>,
    options: &RenderViewOptions,
    engine: XrayEngine,
    size: Option<(u32, u32)>,
  ) {
    let bloom: &RenderBloomSettings = &options.features.bloom;
    let size: Option<(u32, u32)> = size.filter(|_| options.mode.is_lit && !options.mode.is_wireframe);

    self.enhanced = EnhancedBloomUniform::for_view(options).filter(|_| size.is_some());

    if self.enhanced.is_some() {
      self.ask_pipelines(passes.enhanced_bloom, passes.pipelines);
      self.enhanced = self.enhanced.filter(|_| self.pipelines.is_some());
    }

    self.engine = size
      .filter(|_| bloom.is_enabled && self.enhanced.is_none())
      .map(|size| {
        let aspect: f32 = size.1 as f32 / size.0 as f32;
        let is_one_sided: bool = engine == XrayEngine::Extended;

        [
          BloomUniform::build(size, bloom.threshold),
          BloomUniform::filter(true, (bloom.radius, bloom.strength), aspect, is_one_sided),
          BloomUniform::filter(false, (bloom.radius, bloom.strength), aspect, is_one_sided),
        ]
      });
  }

  /// Asks the cache for the enhanced bloom's pipelines where it changed since they were last asked for; a refusal
  /// keeps the last made, and is not asked again before the cache changes.
  fn ask_pipelines(&mut self, pass: &EnhancedBloomPass, source: PipelineSource<'_>) {
    let generation: u64 = source.get_generation();

    if self.pipelines.as_ref().is_some_and(|(asked, _)| *asked == generation) {
      return;
    }

    match pass.get_pipelines(source) {
      Ok(pipelines) => self.pipelines = Some((generation, pipelines)),
      Err(error) => {
        log::error!("Enhanced bloom rejected, blooming with the last one: {error}");

        if let Some((asked, _)) = &mut self.pipelines {
          *asked = generation;
        }
      }
    }
  }

  /// Whether the engine's bloom draws this frame, which the present adds.
  pub fn is_engine_drawn(&self) -> bool {
    self.engine.is_some()
  }

  /// Whether the enhanced bloom draws this frame in the engine's place.
  pub fn is_enhanced_drawn(&self) -> bool {
    self.enhanced.is_some()
  }

  /// Adds the enhanced bloom from the scene as it stands, answering what the present screens over the frame: the
  /// half-size blur across and down, the build, six halvings, five doublings back to half size, and the finish; none
  /// where the enhanced bloom does not draw.
  pub fn add_enhanced_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (targets, frame, lighting): (ViewTargetHandles, (u32, u32), UniformBinding<LightingUniform>),
  ) -> Option<GraphTexture> {
    let settings: &RenderEnhancedBloomSettings = self.enhanced.as_ref()?;
    let (_, pipelines) = self.pipelines.as_ref()?;
    let pass: &'a EnhancedBloomPass = passes.enhanced_bloom;
    let empty: GraphTexture =
      bindings.import_view(&mut *graph, "enhanced bloom empty", passes.fog_scattering.get_empty());
    let half_size: (u32, u32) = EnhancedBloomUniform::get_level_size(frame, 1);
    let stage = |size: (u32, u32)| EnhancedBloomUniform::new(size, frame, settings);
    let mut create = |label: &'static str, (width, height): (u32, u32), format: wgpu::TextureFormat| {
      graph.create_texture(GraphTextureDescriptor::new_2d(label, width, height, format))
    };
    let across: GraphTexture = create("enhanced bloom across", half_size, EnhancedBloomPass::BLURRED);
    let blurred: GraphTexture = create("enhanced bloom blurred", half_size, EnhancedBloomPass::BLURRED);
    let built: GraphTexture = create("enhanced bloom built", half_size, EnhancedBloomPass::BLOOM);
    // Each halving's target, the first at half the frame's size, then each doubling's, the last back at half size.
    let halved: Vec<(GraphTexture, (u32, u32))> = (1..=EnhancedBloomUniform::LEVELS as u32)
      .map(|level| {
        let size: (u32, u32) = EnhancedBloomUniform::get_level_size(frame, level);

        (create("enhanced bloom halved", size, EnhancedBloomPass::BLOOM), size)
      })
      .collect();
    let doubled: Vec<(GraphTexture, (u32, u32))> = (1..EnhancedBloomUniform::LEVELS as u32)
      .rev()
      .map(|level| {
        let size: (u32, u32) = EnhancedBloomUniform::get_level_size(frame, level);

        (create("enhanced bloom doubled", size, EnhancedBloomPass::BLOOM), size)
      })
      .collect();
    let finished: GraphTexture = create("enhanced bloom", half_size, EnhancedBloomPass::BLOOM);
    // Adds one stage: its pass, its pipeline, what it reads and what it adds, what it writes, and its uniform.
    let mut add = |name: &'static str,
                   stage: EnhancedBloomStage,
                   (source, coarser): (GraphTexture, GraphTexture),
                   written: GraphTexture,
                   uniform: EnhancedBloomUniform| {
      let parameters: EnhancedBloomParameters = EnhancedBloomParameters {
        source,
        coarser,
        source_sampler: pass.get_sampler(),
        depth_target: targets.depth,
        material_target: targets.material,
        bloom: runtime.push_uniform(&uniform),
        lighting,
      };

      graph
        .add_raster_pass(name)
        .parameters(&parameters)
        .color(GraphColorAttachment::new(
          written,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .record(move |context| pass.record(context, view, &pipelines[stage.get_index()], &parameters));
    };

    add(
      EnhancedBloomPass::BLUR_PASSES[0],
      EnhancedBloomStage::Blur,
      (targets.scene, empty),
      across,
      stage(half_size).along(true).reading_scene(),
    );
    add(
      EnhancedBloomPass::BLUR_PASSES[1],
      EnhancedBloomStage::Blur,
      (across, empty),
      blurred,
      stage(half_size).along(false),
    );
    add(
      EnhancedBloomPass::BUILD_PASS,
      EnhancedBloomStage::Build,
      (blurred, empty),
      built,
      stage(half_size),
    );

    let mut source: GraphTexture = built;

    for (index, (target, size)) in halved.iter().enumerate() {
      add(
        EnhancedBloomPass::HALVE_PASSES[index],
        EnhancedBloomStage::Halve,
        (source, empty),
        *target,
        stage(*size),
      );
      source = *target;
    }

    // The first doubling reads the smallest halving twice over; each after reads the doubling before and adds the
    // halving of that one's size.
    for (index, (target, size)) in doubled.iter().enumerate() {
      let coarser: GraphTexture = halved[halved.len() - 1 - index].0;

      add(
        EnhancedBloomPass::DOUBLE_PASSES[index],
        EnhancedBloomStage::Double,
        (source, coarser),
        *target,
        stage(*size),
      );
      source = *target;
    }

    add(
      EnhancedBloomPass::FINISH_PASS,
      EnhancedBloomStage::Finish,
      (source, empty),
      finished,
      stage(half_size),
    );

    Some(finished)
  }

  /// Adds the engine's bloom: built from the high target into the first, blurred across into the second, then down
  /// into the first; nothing where it does not draw.
  pub fn add_engine_passes<'a>(
    &self,
    (graph, runtime): (&mut FrameGraph<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    handles: ViewTargetHandles,
  ) {
    let Some(stages) = self.engine else {
      return;
    };

    for (stage, (read, written)) in BloomStage::ALL.into_iter().zip([
      (handles.high, handles.bloom[0]),
      (handles.bloom[0], handles.bloom[1]),
      (handles.bloom[1], handles.bloom[0]),
    ]) {
      let parameters: BloomParameters = BloomParameters {
        source: read,
        source_sampler: passes.bloom.get_sampler(),
        bloom: runtime.push_uniform(&stages[stage.get_index()]),
      };

      graph
        .add_raster_pass(stage.get_name())
        .parameters(&parameters)
        .color(GraphColorAttachment::new(
          written,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .record(move |context| passes.bloom.record(context, stage, &parameters));
    }
  }
}

impl ViewFeature for BloomFeature {
  /// Its targets are the view's (the high target and the two bloom targets) or the frame's own.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {
    *self = Self::default();
  }
}
