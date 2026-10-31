use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureDescriptor, UniformBinding};

use crate::camera::camera_view::CameraView;
use crate::contract::render_camera_settings::RenderCameraSettings;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::camera_effects_parameters::CameraEffectsParameters;
use crate::pass::camera_effects_pass::CameraEffectsPass;
use crate::pass::camera_effects_stage::CameraEffectsStage;
use crate::pass::camera_effects_uniform::CameraEffectsUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::pipeline_source::PipelineSource;
use crate::scene::level::feature::view_feature::ViewFeature;

/// The camera the resolved frame is shown through (`RenderCameraSettings`): its depth of field, then its motion blur,
/// then its lens, each over the image before it; the film's grain is the present's. Drawn where the view is lit, shows
/// the finished frame, and turns one of them on.
#[derive(Default)]
pub struct CameraFeature {
  /// What the camera draws this frame: its settings and the view it looks through.
  frame: Option<(RenderCameraSettings, CameraView)>,
  /// The stages' pipelines, with the cache's generation they were asked for in; the last made are kept while a
  /// reloaded shader is refused.
  pipelines: Option<(u64, [wgpu::RenderPipeline; CameraEffectsStage::COUNT])>,
}

impl CameraFeature {
  /// Takes this frame's camera as `options` set it, looking through `view`; the stages' pipelines asked of the cache
  /// when it draws and the cache has changed since they were.
  pub fn prepare(&mut self, passes: LevelPasses<'_>, options: &RenderViewOptions, view: &CameraView) {
    let settings: &RenderCameraSettings = &options.features.camera;
    let is_drawn: bool = options.mode.is_lit
      && !options.mode.is_wireframe
      && options.mode.debug_view == RenderDebugView::Final
      && (settings.is_blurred() || settings.is_vignetted || settings.is_aberrated);

    self.frame = None;

    if is_drawn {
      self.ask_pipelines(passes.camera_effects, passes.pipelines);
      self.frame = self.pipelines.as_ref().map(|_| (*settings, *view));
    }
  }

  /// Asks the cache for the stages' pipelines where it changed since they were last asked for; a refusal keeps the
  /// last made, and is not asked again before the cache changes.
  fn ask_pipelines(&mut self, pass: &CameraEffectsPass, source: PipelineSource<'_>) {
    let generation: u64 = source.get_generation();

    if self.pipelines.as_ref().is_some_and(|(asked, _)| *asked == generation) {
      return;
    }

    match pass.get_pipelines(source) {
      Ok(pipelines) => self.pipelines = Some((generation, pipelines)),
      Err(error) => {
        log::error!("Camera effects rejected, showing with the last ones: {error}");

        if let Some((asked, _)) = &mut self.pipelines {
          *asked = generation;
        }
      }
    }
  }

  /// Adds the camera over `shown`, the image the resolve shows, `size` pixels across, answering the image it leaves,
  /// none where it draws nothing: the distance and motion under each pixel, read from the drawn `depth` and
  /// `handles.motion`; the depth of field; the motion blur; and the lens.
  pub fn add_passes<'a>(
    &'a self,
    GraphDeclaration { graph, runtime, .. }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    handles: ViewTargetHandles,
    (shown, size): (GraphTexture, (u32, u32)),
  ) -> Option<GraphTexture> {
    let (settings, view) = self.frame.as_ref()?;
    let (_, pipelines) = self.pipelines.as_ref()?;
    let uniform: CameraEffectsUniform = CameraEffectsUniform::new(settings, view, (handles.size, size));

    // A switch on that draws nothing, the depth of field through a lens of parallel rays, leaves the image as it is.
    if !uniform.is_focused() && !uniform.is_motion_blurred() && !uniform.is_lensed() {
      return None;
    }

    let pass: &'a CameraEffectsPass = passes.camera_effects;
    let camera_effects: UniformBinding<CameraEffectsUniform> = runtime.push_uniform(&uniform);
    let per = |tile: u32| (size.0.div_ceil(tile), size.1.div_ceil(tile));
    // Adds one stage reading `source`, `beside` and `tiles` and the `inputs` beside them, answering the target of
    // `target` texels across it writes.
    let mut add = |stage: CameraEffectsStage,
                   (source, beside, tiles): (GraphTexture, GraphTexture, GraphTexture),
                   inputs: GraphTexture,
                   (width, height): (u32, u32)| {
      let written: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
        stage.get_pass(),
        width.max(1),
        height.max(1),
        CameraEffectsPass::TARGET,
      ));
      let parameters: CameraEffectsParameters = CameraEffectsParameters {
        camera_effects,
        source,
        beside,
        inputs,
        tiles,
        source_sampler: pass.get_sampler(),
        depth_target: handles.depth,
        motion_target: handles.motion,
      };

      graph
        .add_raster_pass(stage.get_pass())
        .parameters(&parameters)
        .color(GraphColorAttachment::new(
          written,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .record(move |context| pass.record(context, &pipelines[stage.get_index()], &parameters));

      written
    };
    let mut image: GraphTexture = shown;
    // Read only by the blurs; the image stands in for them where neither draws.
    let inputs: GraphTexture = if uniform.is_focused() || uniform.is_motion_blurred() {
      add(CameraEffectsStage::Inputs, (shown, shown, shown), shown, size)
    } else {
      shown
    };

    if uniform.is_focused() {
      let reduced: GraphTexture = add(CameraEffectsStage::FocusReduce, (image, image, image), inputs, per(2));
      let tiles_size: (u32, u32) = per(CameraEffectsUniform::FOCUS_TILE);
      let tiles: GraphTexture = add(
        CameraEffectsStage::FocusTiles,
        (reduced, reduced, reduced),
        inputs,
        tiles_size,
      );
      let dilated: GraphTexture = add(
        CameraEffectsStage::FocusDilate,
        (tiles, tiles, tiles),
        inputs,
        tiles_size,
      );
      let gathered: GraphTexture = add(
        CameraEffectsStage::FocusGather,
        (reduced, reduced, dilated),
        inputs,
        per(2),
      );

      image = add(CameraEffectsStage::FocusCombine, (image, gathered, image), inputs, size);
    }

    if uniform.is_motion_blurred() {
      let blocks: GraphTexture = add(
        CameraEffectsStage::MotionReduce,
        (image, image, image),
        inputs,
        per(CameraEffectsUniform::MOTION_BLOCK),
      );
      let tiles_size: (u32, u32) = per(uniform.get_motion_tile_size());
      let tiles: GraphTexture = add(
        CameraEffectsStage::MotionTiles,
        (blocks, blocks, blocks),
        inputs,
        tiles_size,
      );
      let neighbours: GraphTexture = add(
        CameraEffectsStage::MotionNeighbours,
        (tiles, tiles, tiles),
        inputs,
        tiles_size,
      );

      image = add(CameraEffectsStage::MotionBlur, (image, image, neighbours), inputs, size);
    }

    if uniform.is_lensed() {
      image = add(CameraEffectsStage::Lens, (image, image, image), inputs, size);
    }

    Some(image)
  }
}

impl ViewFeature for CameraFeature {
  /// Its targets are the frame's transients.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {
    *self = Self::default();
  }
}
