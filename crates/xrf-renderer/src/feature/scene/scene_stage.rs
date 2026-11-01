use xrf_renderer_core::GraphDeclaration;

use crate::camera::camera_view::CameraView;
use crate::contract::render_view_options::RenderViewOptions;
use crate::feature::scene::calibration::calibration_feature::CalibrationFeature;
use crate::feature::view::view_units::view_units;
use crate::scene::level::view_record::ViewRecord;

/// The units of a view's scene that draw in it alone: the calibration props drawn into the G-buffer. The statics, the
/// grass, the reflections' depth and the sky's motion record around them.
#[derive(Default)]
pub struct SceneStage {
  calibration: CalibrationFeature,
}

impl SceneStage {
  /// Takes this frame's units as `options` ask, looking through `camera`.
  pub fn prepare(&mut self, options: &RenderViewOptions, camera: &CameraView) {
    self.calibration.prepare(options, camera);
  }

  /// Draws into the G-buffer over what the static scene drew: the calibration props, while they are shown.
  pub fn add_passes<'a>(
    &self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
  ) {
    self.calibration.add_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      record.passes,
      record.view,
      record.handles,
    );
  }
}

view_units!(SceneStage { calibration });
