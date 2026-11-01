use xrf_renderer_core::{GraphDeclaration, GraphTexture};

use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::level_passes::LevelPasses;
use crate::scene::level::feature::camera_feature::CameraFeature;
use crate::scene::level::feature::image_chain::ImageChain;
use crate::scene::level::feature::smoothing_feature::SmoothingFeature;
use crate::scene::level::feature::view_units::view_units;
use crate::scene::level::view_prepare::ViewPrepare;

/// The units turning a view's frame into the image it shows, each alone in that part of it: the smoothing before the
/// resolve, and the camera laid over the resolved image. The present's distortion, the resolve, the enhanced bloom,
/// the selection, the streaks and the exposure record between them.
#[derive(Default)]
pub struct ImageStage {
  smoothing: SmoothingFeature,
  camera: CameraFeature,
}

impl ImageStage {
  /// Takes this frame's smoothing and camera as the view's options ask.
  pub fn prepare(&mut self, frame: &ViewPrepare<'_>) {
    self.smoothing.prepare(frame);
    self.camera.prepare(frame.passes, frame.options, frame.view);
  }

  /// The pass smoothing the frame, where one does.
  pub fn get_smoothing(&self) -> Option<RenderAntialiasing> {
    self.smoothing.get_mode()
  }

  /// Adds the smoothing of the view's scene, answering the frame smoothed, or the scene where nothing smooths it.
  pub fn add_smoothing<'a>(
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

  /// Adds the camera over the image the resolve shows (`chain`), reading the drawn depth and motion of `handles`, and
  /// answers the chain with what it laid over it, so no history carries its blurs.
  pub fn add_camera<'a>(
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
