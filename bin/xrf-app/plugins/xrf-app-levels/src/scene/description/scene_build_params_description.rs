use serde::Serialize;
use xrf_editable_level::EditableBuildParams;

/// What the scene's `[build_params]` set the compilers to weld, smooth and bake with; the reserved fields left out.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneBuildParamsDescription {
  /// Degrees between faces' normals that still smooth into one.
  pub smooth_angle: f32,
  /// Metres within which the compiler welds vertices.
  pub weld_distance: f32,
  pub light_pixel_per_meter: f32,
  pub light_jitter_samples: u32,
  pub light_quality: u16,
}

impl From<&EditableBuildParams> for SceneBuildParamsDescription {
  fn from(params: &EditableBuildParams) -> Self {
    Self {
      smooth_angle: params.smooth_angle,
      weld_distance: params.weld_distance,
      light_pixel_per_meter: params.light_pixel_per_meter,
      light_jitter_samples: params.light_jitter_samples,
      light_quality: params.light_quality,
    }
  }
}
