use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphDepthAttachment};

use crate::camera::camera_view::CameraView;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::calibration_parameters::CalibrationParameters;
use crate::pass::calibration_uniform::CalibrationUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;

/// A view's calibration props while it shows them: stood ahead of the camera in the frame they are first shown, and
/// kept there, so the camera can move about them.
#[derive(Default)]
pub struct CalibrationFeature {
  props: Option<CalibrationUniform>,
}

impl CalibrationFeature {
  /// Stands the props ahead of `camera` when the view first shows them; forgets them once it does not.
  pub fn prepare(&mut self, options: &RenderViewOptions, camera: &CameraView) {
    if !options.show.is_calibration_visible {
      self.props = None;
    } else if self.props.is_none() {
      self.props = Some(CalibrationUniform::new(camera.position, camera.get_look()));
    }
  }

  /// Draws the props into the G-buffer over what the scene drew; nothing while they are not shown.
  pub fn add_passes<'a>(
    &self,
    GraphDeclaration { graph, runtime, .. }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    targets: ViewTargetHandles,
  ) {
    let Some(props) = &self.props else {
      return;
    };
    let parameters: CalibrationParameters = CalibrationParameters {
      calibration: runtime.push_uniform(props),
    };

    targets
      .get_gbuffer()
      .into_iter()
      .fold(graph.add_raster_pass("calibration"), |builder, texture| {
        builder.color(GraphColorAttachment::new(texture, wgpu::LoadOp::Load))
      })
      .depth(GraphDepthAttachment::new(targets.depth, wgpu::LoadOp::Load))
      .parameters(&parameters)
      .record(move |context| passes.calibration.record(context, view, &parameters));
  }
}

impl ViewFeature for CalibrationFeature {
  /// Nothing: the props are a uniform pushed each frame.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {
    self.props = None;
  }
}
