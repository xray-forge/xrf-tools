use xrf_renderer_core::{GraphDeclaration, GraphTexture};

use crate::camera::camera_view::CameraView;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::scene::ai_map::ai_map_level::AiMapLevel;
use crate::feature::scene::ai_map::ai_map_view::AiMapView;
use crate::feature::scene::calibration::calibration_feature::CalibrationFeature;
use crate::feature::view::view_units::view_units;
use crate::scene::level::view::record::view_record::ViewRecord;

/// The units of a view's scene that draw in it alone: the calibration props drawn into the G-buffer, and the AI map's
/// debug views written over the light. The statics, the grass, the reflections' depth and the sky's motion record
/// around them.
#[derive(Default)]
pub(crate) struct SceneStage {
  calibration: CalibrationFeature,
  ai_map: AiMapView,
}

impl SceneStage {
  /// Takes this frame's units as `options` ask, looking through `camera`.
  pub(crate) fn prepare(&mut self, options: &RenderViewOptions, camera: &CameraView) {
    self.calibration.prepare(options, camera);
    self.ai_map.prepare(options);
  }

  /// Draws into the G-buffer over what the static scene drew: the calibration props, while they are shown.
  pub(crate) fn add_passes<'a>(
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

  /// Writes the AI map's debug view over `light` where the view shows one: a field of each drawn pixel's node of
  /// `level`.
  pub(crate) fn add_ai_map<'a>(
    &self,
    declaration: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    level: Option<&'a AiMapLevel>,
    light: GraphTexture,
  ) {
    self.ai_map.add_node_field(declaration, record, level, light);
  }
}

view_units!(SceneStage { calibration, ai_map });
