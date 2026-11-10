use xrf_engine_target::XrayEngine;
use xrf_renderer_core::{
  GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureDescriptor, PipelineAsk, UniformBinding,
};

use crate::contract::feature::image::bloom::render_bloom_mode::RenderBloomMode;
use crate::contract::feature::image::bloom::render_bloom_settings::RenderBloomSettings;
use crate::contract::feature::image::bloom::render_enhanced_bloom_settings::RenderEnhancedBloomSettings;
use crate::contract::report::applied::render_applied_report::RenderAppliedReport;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::image::bloom::bloom_filter::BloomFilter;
use crate::feature::image::bloom::bloom_parameters::BloomParameters;
use crate::feature::image::bloom::bloom_stage::BloomStage;
use crate::feature::image::bloom::bloom_uniform::BloomUniform;
use crate::feature::image::bloom::enhanced::enhanced_bloom_parameters::EnhancedBloomParameters;
use crate::feature::image::bloom::enhanced::enhanced_bloom_pass::EnhancedBloomPass;
use crate::feature::image::bloom::enhanced::enhanced_bloom_stage::EnhancedBloomStage;
use crate::feature::image::bloom::enhanced::enhanced_bloom_uniform::EnhancedBloomUniform;
use crate::feature::view::view_feature::ViewFeature;
use crate::frame::jitter_guard::JitterGuard;
use crate::frame::targets::view_target_handles::ViewTargetHandles;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;
use crate::pass::level_passes::LevelPasses;
use crate::scene::level::view::record::view_record::ViewRecord;

/// The bloom, the engine's or the enhanced one in its place: the engine's build and blurs at its fixed size, read by
/// the present; or the enhanced chain from the resolved image, its finish handed to the present.
#[derive(Default)]
pub(crate) struct BloomFeature {
  /// What the engine's build and its two blurs read this frame, where the engine's bloom is built: where it is shown,
  /// or under the enhanced bloom where the frame's distortion blurs towards it.
  engine: Option<[BloomUniform; 3]>,
  /// Whether the engine's bloom is shown this frame, which the present adds.
  is_engine_shown: bool,
  /// What the enhanced bloom draws with, where it draws in the engine's place.
  enhanced: Option<RenderEnhancedBloomSettings>,
  /// The enhanced bloom's pipelines, one a stage, asked for once it draws.
  pipelines: PipelineAsk<[wgpu::RenderPipeline; 5]>,
}

impl BloomFeature {
  /// Takes this frame's bloom for a frame `size` texels across, drawn where the view is lit and not wireframe; the
  /// enhanced bloom's pipelines asked of the cache when it draws and the cache has changed since they were.
  pub(crate) fn prepare(
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
      let pass: &EnhancedBloomPass = &passes.image.enhanced_bloom;

      if let Err(error) = self
        .pipelines
        .ask(passes.pipelines.cache, || pass.get_pipelines(passes.pipelines))
      {
        log::error!("Enhanced bloom rejected, blooming with the last one: {error}");
      }

      self.enhanced = self.enhanced.filter(|_| self.pipelines.get().is_some());
    }

    self.is_engine_shown = size.is_some() && bloom.is_enabled && self.enhanced.is_none();
    self.engine = size
      .filter(|_| self.is_engine_shown || self.enhanced.is_some())
      .map(|size| {
        let across: BloomFilter = BloomFilter {
          is_across: true,
          radius: bloom.radius,
          strength: bloom.strength,
          aspect: size.1 as f32 / size.0 as f32,
          is_one_sided: engine == XrayEngine::Extended,
        };

        [
          BloomUniform::build(bloom.threshold),
          BloomUniform::filter(across),
          BloomUniform::filter(BloomFilter {
            is_across: false,
            ..across
          }),
        ]
      });
  }

  /// Builds the engine's bloom under the enhanced one only where the frame's distortion may blur it (`is_blurring`),
  /// which blurs towards the engine's bloom as `combine_2`'s does; once every unit knows what it draws.
  pub(crate) fn keep_for_distortion(&mut self, is_blurring: bool) {
    if !self.is_engine_shown && !is_blurring {
      self.engine = None;
    }
  }

  /// Whether the engine's bloom is shown this frame, which the present adds.
  pub(crate) fn is_engine_drawn(&self) -> bool {
    self.is_engine_shown && self.engine.is_some()
  }

  /// Whether the engine's bloom is built this frame, shown or under the enhanced one for the distortion's blur.
  pub(crate) fn is_engine_built(&self) -> bool {
    self.engine.is_some()
  }

  /// Whether the enhanced bloom draws this frame in the engine's place.
  pub(crate) fn is_enhanced_drawn(&self) -> bool {
    self.enhanced.is_some()
  }

  /// Writes into `applied` which bloom it shows this frame, none where neither is.
  pub(crate) fn describe(&self, applied: &mut RenderAppliedReport) {
    applied.bloom = if self.is_enhanced_drawn() {
      Some(RenderBloomMode::Enhanced)
    } else {
      self.is_engine_drawn().then_some(RenderBloomMode::Engine)
    };
  }

  /// Adds the enhanced bloom from `image`, the resolved image the present shows, answering what the present screens
  /// over it: the half-size blur across and down, the build, six halvings, five doublings back to half size, and the
  /// finish; none where the enhanced bloom does not draw. Built after the resolve, as the major engines build theirs: from
  /// the frame as drawn, a glint that comes and goes with the jitter bloomed into a halo nothing smoothed. Its sizes
  /// and the depth and marks it reads are the frame's as drawn.
  pub(crate) fn add_enhanced_passes<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    ViewRecord {
      passes,
      view,
      handles: targets,
      lit,
      ..
    }: ViewRecord<'a>,
    image: GraphTexture,
  ) -> Option<GraphTexture> {
    let frame: (u32, u32) = targets.size;
    let lighting: UniformBinding<LightingUniform> = lit.lighting;
    let settings: &RenderEnhancedBloomSettings = self.enhanced.as_ref()?;
    let pipelines: &[wgpu::RenderPipeline; 5] = self.pipelines.get()?;
    let pass: &'a EnhancedBloomPass = &passes.image.enhanced_bloom;
    let empty: GraphTexture = bindings.import_view(
      &mut *graph,
      "enhanced bloom empty",
      passes.weather.fog_scattering.get_empty(),
    );
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
      let parameters: EnhancedBloomParameters<'_> = EnhancedBloomParameters {
        source,
        coarser,
        source_sampler: pass.get_sampler(),
        depth_target: targets.depth,
        material_target: targets.material,
        bloom: runtime.push_uniform(&uniform),
        lighting,
      };

      // The build reads the fog's depth and the self-lit marks over the four drawn texels under each of its half-size
      // texels; the other stages share its layout and read neither.
      graph
        .add_raster_pass(name)
        .tag(JitterGuard::READS_DEPTH)
        .tag(JitterGuard::READS_MATERIAL)
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
      (image, empty),
      across,
      stage(half_size).across().reading_scene(),
    );
    add(
      EnhancedBloomPass::BLUR_PASSES[1],
      EnhancedBloomStage::Blur,
      (across, empty),
      blurred,
      stage(half_size).down(),
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
  /// into the first; nothing where it does not draw. The high part is the engine's own (the opaque surfaces, the sky
  /// and the sun shafts), drawn before the resolve and never resolved, so the build averages the whole footprint of each
  /// texel, the same light however the jitter falls.
  pub(crate) fn add_engine_passes<'a>(
    &self,
    GraphDeclaration { graph, runtime, .. }: GraphDeclaration<'_, 'a>,
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
      let parameters: BloomParameters<'_> = BloomParameters {
        source: read,
        source_sampler: passes.image.bloom.get_sampler(),
        bloom: runtime.push_uniform(&stages[stage.get_index()]),
      };

      graph
        .add_raster_pass(stage.get_name())
        .parameters(&parameters)
        .color(GraphColorAttachment::new(
          written,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .record(move |context| passes.image.bloom.record(context, stage, &parameters));
    }
  }
}

impl ViewFeature for BloomFeature {
  fn release(&mut self) {
    *self = Self::default();
  }
}
