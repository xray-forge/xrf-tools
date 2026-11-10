use xrf_renderer_core::{
  GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureDescriptor, PipelineAsk, UniformBinding,
};

use crate::camera::camera_view::CameraView;
use crate::contract::feature::image::render_camera_settings::RenderCameraSettings;
use crate::contract::view::output::render_debug_view::RenderDebugView;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::image::camera::camera_effects_parameters::CameraEffectsParameters;
use crate::feature::image::camera::camera_effects_pass::CameraEffectsPass;
use crate::feature::image::camera::camera_effects_stage::CameraEffectsStage;
use crate::feature::image::camera::camera_effects_uniform::CameraEffectsUniform;
use crate::feature::image::camera::camera_frame::CameraFrame;
use crate::feature::image::image_chain::ImageChain;
use crate::feature::view::view_feature::ViewFeature;
use crate::frame::jitter_guard::JitterGuard;
use crate::frame::targets::view_target_handles::ViewTargetHandles;
use crate::pass::level_passes::LevelPasses;

/// The camera the resolved frame is shown through (`RenderCameraSettings`): its depth of field, then its motion blur,
/// then its lens, each over the image before it; the film's grain is the present's. Drawn where the view is lit, shows
/// the finished frame, and turns one of them on.
#[derive(Default)]
pub(crate) struct CameraFeature {
  /// What the camera draws this frame with, none where it draws nothing.
  frame: Option<CameraFrame>,
  /// The stages' pipelines, asked for once it draws, made ahead as the renderer starts (`ImagePasses::new`).
  pipelines: PipelineAsk<[wgpu::RenderPipeline; CameraEffectsStage::COUNT]>,
}

impl CameraFeature {
  /// Takes this frame's camera as `options` set it, looking through `view`; the stages' pipelines asked of the cache
  /// when it draws and the cache has changed since they were.
  pub(crate) fn prepare(&mut self, passes: LevelPasses<'_>, options: &RenderViewOptions, view: &CameraView) {
    let settings: &RenderCameraSettings = &options.features.camera;
    let is_drawn: bool = options.mode.is_lit
      && !options.mode.is_wireframe
      && options.mode.debug_view == RenderDebugView::Final
      && (settings.is_blurred() || settings.is_vignetted || settings.is_aberrated);

    self.frame = None;

    if is_drawn {
      let pass: &CameraEffectsPass = &passes.image.camera_effects;

      if let Err(error) = self
        .pipelines
        .ask(passes.pipelines.cache, || pass.get_pipelines(passes.pipelines))
      {
        log::error!("Camera effects rejected, showing with the last ones: {error}");
      }

      self.frame = self.pipelines.get().map(|_| CameraFrame {
        settings: *settings,
        view: *view,
      });
    }
  }

  /// Adds the camera over the image the resolve shows (`chain`), answering the image it leaves, none where it draws
  /// nothing: the distance and motion under each pixel, read from the drawn `depth` and `handles.motion`; the depth of
  /// field; the motion blur; and the lens.
  pub(crate) fn add_passes<'a>(
    &'a self,
    GraphDeclaration { graph, runtime, .. }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    handles: ViewTargetHandles,
    ImageChain {
      shown,
      shown_size: size,
      ..
    }: ImageChain,
  ) -> Option<GraphTexture> {
    let CameraFrame { settings, view } = self.frame.as_ref()?;
    let pipelines: &[wgpu::RenderPipeline; CameraEffectsStage::COUNT] = self.pipelines.get()?;
    let uniform: CameraEffectsUniform = CameraEffectsUniform::new(settings, view, (handles.size, size));

    // A switch on that draws nothing, the depth of field through a lens of parallel rays, leaves the image as it is.
    if !uniform.is_focused() && !uniform.is_motion_blurred() && !uniform.is_lensed() {
      return None;
    }

    let pass: &'a CameraEffectsPass = &passes.image.camera_effects;
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
      let parameters: CameraEffectsParameters<'_> = CameraEffectsParameters {
        camera_effects,
        source,
        beside,
        inputs,
        tiles,
        source_sampler: pass.get_sampler(),
        depth_target: handles.depth,
        motion_target: handles.motion,
      };

      // The inputs stage reads the depth, the water's where it draws, and the motion over the drawn texels about each
      // shown pixel, the nearest and the longest among them; the other stages share its layout and read neither.
      graph
        .add_raster_pass(stage.get_pass())
        .tag(JitterGuard::READS_DEPTH)
        .tag(JitterGuard::READS_DEPTH_WITH_WATER)
        .tag(JitterGuard::READS_MOTION)
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
  fn release(&mut self) {
    *self = Self::default();
  }
}
