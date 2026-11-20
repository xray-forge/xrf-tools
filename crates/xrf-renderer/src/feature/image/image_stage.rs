use xrf_renderer_core::{GraphDeclaration, GraphTexture};

use crate::contract::feature::image::resolve::render_antialiasing::RenderAntialiasing;
use crate::feature::image::camera::camera_feature::CameraFeature;
use crate::feature::image::image_chain::ImageChain;
use crate::feature::image::smoothing::smoothing_feature::SmoothingFeature;
use crate::feature::view::view_units::view_units;
use crate::frame::targets::view_target_handles::ViewTargetHandles;
use crate::pass::level_passes::LevelPasses;
use crate::scene::level::view::view_prepare::ViewPrepare;

/// The units turning a view's frame into the image it shows, each alone in that part of it: the smoothing before the
/// resolve, and the layers laid over the resolved image (the camera's). The present's distortion, the resolve, the
/// enhanced bloom, the selection, the streaks and the exposure record between them.
#[derive(Default)]
pub(crate) struct ImageStage {
  smoothing: SmoothingFeature,
  camera: CameraFeature,
}

impl ImageStage {
  /// Takes this frame's smoothing and camera as the view's options ask.
  pub(crate) fn prepare(&mut self, frame: &ViewPrepare<'_>) {
    self.smoothing.prepare(frame);
    self.camera.prepare(frame.passes, frame.options, frame.view);
  }

  /// The pass smoothing the frame, where one does.
  pub(crate) fn get_smoothing(&self) -> Option<RenderAntialiasing> {
    self.smoothing.get_mode()
  }

  /// Adds the smoothing of the view's scene, answering the frame smoothed, or the scene where nothing smooths it.
  pub(crate) fn add_smoothing<'a>(
    &self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    handles: ViewTargetHandles,
  ) -> GraphTexture {
    self.smoothing.add_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      passes,
      handles.scene,
      handles.size,
    )
  }

  /// Lays the stage's layers over the image the resolve shows (`chain`), in order, reading the drawn depth and motion of
  /// `handles`, and answers the chain with what they laid over it, so no history carries them: the camera's effects.
  pub(crate) fn add_over_resolved<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    handles: ViewTargetHandles,
    chain: ImageChain,
  ) -> ImageChain {
    chain.lay(
      self
        .camera
        .add_passes(GraphDeclaration::new(graph, bindings, runtime), passes, handles, chain),
    )
  }
}

view_units!(ImageStage { smoothing, camera });
